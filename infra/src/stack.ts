import {
	CfnResource,
	Duration,
	RemovalPolicy,
	Stack,
	type StackProps,
	Validations,
	aws_ec2 as ec2,
	aws_iam as iam,
	aws_lambda as lambda,
	aws_logs as logs,
	aws_rds as rds,
	aws_secretsmanager as secretsmanager,
} from 'aws-cdk-lib';
import type { Construct, IConstruct } from 'constructs';

export const STACK_NAME = 'InsightBoardExperiment';
export const FUNCTION_NAME = 'InsightBoardExperiment';
export const DATABASE_NAME = 'insightboard';
const DATABASE_USER = 'insightboard';
/** Path of the committed RDS CA bundle inside the Lambda package (M6 shared interface). */
export const RDS_CA_BUNDLE_PATH = '/var/task/certs/rds-global-bundle.pem';
/** Provider keys the owner fills into the app secret in M7; blank means unset in the backend. */
export const PROVIDER_KEY_NAMES = [
	'GITHUB_API_TOKEN',
	'OPENWEATHER_API_KEY',
	'NEWSAPI_API_KEY',
] as const;
/**
 * Space and every ASCII punctuation character. Generated database passwords
 * are alphanumeric so they need no escaping inside `DATABASE_URL` (decision 10).
 */
const URL_UNSAFE_CHARACTERS = ' !"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~';

export interface InsightBoardExperimentStackProps extends StackProps {
	/** Absolute path of `insightboard-lambda.zip` (bootstrap, static/, certs/). */
	readonly lambdaAssetPath: string;
}

/**
 * The M7 experiment shape (M6 contract decisions 7, 8, 10): one Lambda function
 * outside any VPC behind a Function URL, RDS PostgreSQL 16, two generated
 * Secrets Manager secrets, and an explicit log group. Synth only.
 */
export class InsightBoardExperimentStack extends Stack {
	constructor(
		scope: Construct,
		id: string,
		props: InsightBoardExperimentStackProps,
	) {
		super(scope, id, props);

		// RDS requires a VPC. Public subnets only: no NAT gateway, no endpoints,
		// and no default-security-group custom resource (decision 7).
		const vpc = new ec2.Vpc(this, 'DatabaseVpc', {
			maxAzs: 2,
			natGateways: 0,
			subnetConfiguration: [
				{ name: 'public', subnetType: ec2.SubnetType.PUBLIC, cidrMask: 24 },
			],
			restrictDefaultSecurityGroup: false,
		});

		// The function runs outside any VPC and has no fixed source address, so
		// 5432 is open to any IPv4 address; TLS and a generated password protect
		// access (decision 8, stated risk).
		const databaseSecurityGroup = new ec2.SecurityGroup(
			this,
			'DatabaseSecurityGroup',
			{
				vpc,
				description: 'InsightBoard experiment Postgres: TLS-only 5432',
				allowAllOutbound: false,
			},
		);
		databaseSecurityGroup.addIngressRule(
			ec2.Peer.anyIpv4(),
			ec2.Port.tcp(5432),
			'Postgres from the Lambda function (no fixed address, decision 8)',
		);

		const engine = rds.DatabaseInstanceEngine.postgres({
			version: rds.PostgresEngineVersion.VER_16,
		});
		const parameterGroup = new rds.ParameterGroup(
			this,
			'DatabaseParameters',
			{
				engine,
				description: 'InsightBoard experiment: reject non-TLS connections',
				parameters: { 'rds.force_ssl': '1' },
			},
		);

		const databaseSecret = new rds.DatabaseSecret(this, 'DatabaseSecret', {
			username: DATABASE_USER,
			excludeCharacters: URL_UNSAFE_CHARACTERS,
		});
		databaseSecret.applyRemovalPolicy(RemovalPolicy.DESTROY);

		const database = new rds.DatabaseInstance(this, 'Database', {
			engine,
			instanceType: ec2.InstanceType.of(
				ec2.InstanceClass.BURSTABLE4_GRAVITON,
				ec2.InstanceSize.MICRO,
			),
			vpc,
			vpcSubnets: { subnetType: ec2.SubnetType.PUBLIC },
			publiclyAccessible: true,
			securityGroups: [databaseSecurityGroup],
			parameterGroup,
			credentials: rds.Credentials.fromSecret(databaseSecret),
			databaseName: DATABASE_NAME,
			allocatedStorage: 20,
			storageType: rds.StorageType.GP3,
			storageEncrypted: true,
			multiAz: false,
			backupRetention: Duration.days(1),
			deleteAutomatedBackups: true,
			deletionProtection: false,
			removalPolicy: RemovalPolicy.DESTROY,
		});

		const appSecret = new secretsmanager.Secret(this, 'AppSecret', {
			description:
				'InsightBoard experiment: JWT_SECRET (generated) and provider keys (owner fills in M7)',
			generateSecretString: {
				secretStringTemplate: JSON.stringify(
					Object.fromEntries(PROVIDER_KEY_NAMES.map((name) => [name, ''])),
				),
				generateStringKey: 'JWT_SECRET',
				passwordLength: 64,
				excludePunctuation: true,
				includeSpace: false,
			},
			removalPolicy: RemovalPolicy.DESTROY,
		});

		const logGroup = new logs.LogGroup(this, 'FunctionLogGroup', {
			logGroupName: `/aws/lambda/${FUNCTION_NAME}`,
			retention: logs.RetentionDays.TWO_WEEKS,
			removalPolicy: RemovalPolicy.DESTROY,
		});

		// An explicit role scoped to the stack's log group instead of the
		// AWSLambdaBasicExecutionRole managed policy. Secrets reach the function as
		// CloudFormation dynamic references, so it needs no Secrets Manager access.
		const role = new iam.Role(this, 'FunctionRole', {
			assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
			description: 'InsightBoard experiment function: write its own logs only',
		});
		logGroup.grantWrite(role);

		const dbUser = databaseSecret
			.secretValueFromJson('username')
			.unsafeUnwrap();
		const dbPassword = databaseSecret
			.secretValueFromJson('password')
			.unsafeUnwrap();
		const fromAppSecret = (key: string) =>
			appSecret.secretValueFromJson(key).unsafeUnwrap();

		const fn = new lambda.Function(this, 'Function', {
			functionName: FUNCTION_NAME,
			description: 'InsightBoard backend and single-page app (M6 lambda entry point)',
			runtime: lambda.Runtime.PROVIDED_AL2023,
			architecture: lambda.Architecture.X86_64,
			handler: 'bootstrap',
			code: lambda.Code.fromAsset(props.lambdaAssetPath),
			memorySize: 512,
			timeout: Duration.seconds(15),
			role,
			logGroup,
			environment: {
				STATIC_DIR: '/var/task/static',
				DB_MAX_CONNECTIONS: '2',
				RUST_LOG: 'insightboard_backend=info,tower_http=info',
				DATABASE_URL:
					`postgres://${dbUser}:${dbPassword}` +
					`@${database.dbInstanceEndpointAddress}:${database.dbInstanceEndpointPort}` +
					`/${DATABASE_NAME}?sslmode=verify-full&sslrootcert=${RDS_CA_BUNDLE_PATH}`,
				JWT_SECRET: fromAppSecret('JWT_SECRET'),
				...Object.fromEntries(
					PROVIDER_KEY_NAMES.map((name) => [name, fromAppSecret(name)]),
				),
			},
		});

		fn.addFunctionUrl({
			authType: lambda.FunctionUrlAuthType.NONE,
			invokeMode: lambda.InvokeMode.BUFFERED,
		});

		// Findings the contract accepts on purpose, from cdk-nag (AwsSolutions-*)
		// and from CDK's built-in offline template validation
		// (CloudFormation-Validate::*). Each is acknowledged on the one
		// CloudFormation resource it concerns, by specific rule ID, with a reason
		// citing the decision behind it.
		const openPostgresReason =
			'Decision 8: the Lambda function runs outside any VPC and has no fixed source address, so Postgres 5432 is open to any IPv4 address; rds.force_ssl=1, sslmode=verify-full, and a generated password protect access (stated risk).';
		Validations.of(cfnResourceOf(vpc)).acknowledge({
			id: 'AwsSolutions-VPC7',
			reason:
				'Decision 7: the VPC exists only to host the experiment database; VPC flow logs add cost and are not part of the accepted experiment shape.',
		});
		Validations.of(cfnResourceOf(databaseSecurityGroup)).acknowledge(
			{ id: 'AwsSolutions-EC23', reason: openPostgresReason },
			{ id: 'CloudFormation-Validate::W2508', reason: openPostgresReason },
		);
		Validations.of(cfnResourceOf(database)).acknowledge(
			{
				id: 'CloudFormation-Validate::W9011',
				reason:
					'Decision 8: the instance is publicly accessible because the Lambda function runs outside any VPC; rds.force_ssl=1, sslmode=verify-full, and a generated password protect access (stated risk).',
			},
			{
				id: 'AwsSolutions-RDS3',
				reason:
					'Decision 8: single-AZ db.t4g.micro keeps the short M7 experiment at minimum cost.',
			},
			{
				id: 'AwsSolutions-RDS10',
				reason:
					'Decision 8 and section 8: deletion protection is off so deleting the stack removes the experiment database with no leftovers.',
			},
			{
				id: 'AwsSolutions-RDS11',
				reason:
					'Decision 8: the default port 5432 is kept; a non-default port adds no protection to an endpoint already guarded by TLS and a generated password.',
			},
		);
		for (const secret of [databaseSecret, appSecret]) {
			Validations.of(cfnResourceOf(secret)).acknowledge({
				id: 'AwsSolutions-SMG4',
				reason:
					'Decision 10: the secrets live only for the short M7 experiment and are deleted with the stack; rotation would need a rotation Lambda and is out of scope.',
			});
		}

		// cfn-lint (scripts/cfn-lint.sh) reads resource-level ignores from
		// Metadata; that format has no reason field, so the reasons live here.
		// W3691: cfn-lint's data lists every major-only EngineVersion ("16") as
		// deprecated because RDS catalogues only full minor versions; decision 8
		// pins the major version and RDS chooses the 16.x minor at creation.
		cfnResourceOf(database).addMetadata('cfn-lint', {
			config: { ignore_checks: ['W3691'] },
		});
		// W3005: CDK adds an explicit DependsOn on the function role next to the
		// Role GetAtt that already orders it; the redundancy is harmless.
		cfnResourceOf(fn).addMetadata('cfn-lint', {
			config: { ignore_checks: ['W3005'] },
		});
	}
}

/** The CloudFormation resource behind an L2 construct. */
function cfnResourceOf(construct: IConstruct): CfnResource {
	const resource = construct.node.defaultChild;
	if (!CfnResource.isCfnResource(resource)) {
		throw new Error(`${construct.node.path} has no CloudFormation resource`);
	}
	return resource;
}
