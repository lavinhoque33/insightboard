import assert from 'node:assert/strict';
import { after, before, describe, it } from 'node:test';
import { aws_sqs as sqs, type cx_api } from 'aws-cdk-lib';
import { Match, Template } from 'aws-cdk-lib/assertions';
import { buildApp, PROJECT_TAGS } from '../src/app';
import { RDS_CA_BUNDLE_PATH, STACK_NAME } from '../src/stack';
import { findSynthProblems } from '../src/synth';
import { makeFixture } from './fixture';

/** One template resource; `Properties` stays loosely typed like the template itself. */
interface CfnResource {
	Type: string;
	Properties?: Record<string, any>;
	Metadata?: {
		cdk_nag?: { rules_to_suppress: { id: string; reason: string }[] };
		'cfn-lint'?: { config: { ignore_checks: string[] } };
	};
}

/** Renders a string or `Fn::Join`, showing intrinsics as `<Ref:Id>` / `<GetAtt:Id.Attr>`. */
function render(value: unknown): string {
	if (typeof value === 'string') return value;
	if (typeof value === 'object' && value !== null) {
		if ('Fn::Join' in value && Array.isArray(value['Fn::Join'])) {
			const [separator, parts] = value['Fn::Join'];
			if (typeof separator === 'string' && Array.isArray(parts)) {
				return parts.map(render).join(separator);
			}
		}
		if ('Ref' in value) return `<Ref:${String(value.Ref)}>`;
		if ('Fn::GetAtt' in value && Array.isArray(value['Fn::GetAtt'])) {
			return `<GetAtt:${value['Fn::GetAtt'].join('.')}>`;
		}
	}
	throw new Error(`unexpected template value ${JSON.stringify(value)}`);
}

describe('InsightBoardExperiment template', () => {
	const fixture = makeFixture();
	let template: Template;
	let assembly: cx_api.CloudAssembly;
	let resources: Record<string, CfnResource>;

	const logicalIdOf = (type: string, prefix: string): string => {
		const ids = Object.keys(template.findResources(type)).filter((id) =>
			id.startsWith(prefix),
		);
		assert.equal(ids.length, 1, `one ${type} named ${prefix}*`);
		return ids[0]!;
	};

	before(() => {
		const { app, stack } = buildApp(fixture);
		template = Template.fromStack(stack);
		assembly = app.synth();
		resources = template.toJSON()['Resources'];
	});
	after(() => fixture.dispose());

	it('contains exactly the decision 7 resources', () => {
		const counts: Record<string, number> = {};
		for (const { Type } of Object.values(resources)) {
			counts[Type] = (counts[Type] ?? 0) + 1;
		}
		assert.deepEqual(counts, {
			'AWS::EC2::VPC': 1,
			'AWS::EC2::Subnet': 2,
			'AWS::EC2::RouteTable': 2,
			'AWS::EC2::SubnetRouteTableAssociation': 2,
			'AWS::EC2::Route': 2,
			'AWS::EC2::InternetGateway': 1,
			'AWS::EC2::VPCGatewayAttachment': 1,
			'AWS::EC2::SecurityGroup': 1,
			'AWS::RDS::DBParameterGroup': 1,
			'AWS::RDS::DBSubnetGroup': 1,
			'AWS::RDS::DBInstance': 1,
			'AWS::SecretsManager::Secret': 2,
			'AWS::SecretsManager::SecretTargetAttachment': 1,
			'AWS::Logs::LogGroup': 1,
			'AWS::IAM::Role': 1,
			'AWS::IAM::Policy': 1,
			'AWS::Lambda::Function': 1,
			'AWS::Lambda::Url': 1,
			'AWS::Lambda::Permission': 2,
		});
	});

	it('has no NAT gateway, Elastic IP, VPC endpoint, or custom resource', () => {
		const forbidden = Object.values(resources)
			.map((r) => r.Type)
			.filter((type) =>
				/^(AWS::EC2::(NatGateway|EIP|VPCEndpoint)|AWS::CloudFormation::CustomResource|Custom::.*)$/.test(
					type,
				),
			);
		assert.deepEqual(forbidden, []);
	});

	it('runs the function on provided.al2023 x86_64 outside any VPC', () => {
		template.hasResourceProperties('AWS::Lambda::Function', {
			Runtime: 'provided.al2023',
			Architectures: ['x86_64'],
			Handler: 'bootstrap',
			MemorySize: 512,
			Timeout: 15,
			VpcConfig: Match.absent(),
		});
	});

	it('gives the function exactly the contract environment, secrets by dynamic reference', () => {
		const fnId = logicalIdOf('AWS::Lambda::Function', 'Function');
		const env: Record<string, unknown> =
			resources[fnId]!.Properties!['Environment'].Variables;
		assert.deepEqual(Object.keys(env).sort(), [
			'DATABASE_URL',
			'DB_MAX_CONNECTIONS',
			'GITHUB_API_TOKEN',
			'JWT_SECRET',
			'NEWSAPI_API_KEY',
			'OPENWEATHER_API_KEY',
			'RUST_LOG',
			'STATIC_DIR',
		]);
		assert.equal(env['STATIC_DIR'], '/var/task/static');
		assert.equal(env['DB_MAX_CONNECTIONS'], '2');
		assert.equal(env['RUST_LOG'], 'insightboard_backend=info,tower_http=info');

		const dbSecretId = logicalIdOf('AWS::SecretsManager::Secret', 'DatabaseSecret');
		const dbId = logicalIdOf('AWS::RDS::DBInstance', 'Database');
		assert.equal(
			render(env['DATABASE_URL']),
			`postgres://{{resolve:secretsmanager:<Ref:${dbSecretId}>:SecretString:username::}}` +
				`:{{resolve:secretsmanager:<Ref:${dbSecretId}>:SecretString:password::}}` +
				`@<GetAtt:${dbId}.Endpoint.Address>:<GetAtt:${dbId}.Endpoint.Port>` +
				`/insightboard?sslmode=verify-full&sslrootcert=${RDS_CA_BUNDLE_PATH}`,
		);

		const appSecretId = logicalIdOf('AWS::SecretsManager::Secret', 'AppSecret');
		for (const key of [
			'JWT_SECRET',
			'GITHUB_API_TOKEN',
			'OPENWEATHER_API_KEY',
			'NEWSAPI_API_KEY',
		]) {
			assert.equal(
				render(env[key]),
				`{{resolve:secretsmanager:<Ref:${appSecretId}>:SecretString:${key}::}}`,
				key,
			);
		}
	});

	it('exposes the function through a public buffered Function URL without CORS', () => {
		template.hasResourceProperties('AWS::Lambda::Url', {
			AuthType: 'NONE',
			InvokeMode: 'BUFFERED',
			Cors: Match.absent(),
		});
	});

	it('defines RDS PostgreSQL 16 per decision 8, deleted with the stack', () => {
		const parameterGroupId = logicalIdOf('AWS::RDS::DBParameterGroup', 'DatabaseParameters');
		template.hasResource('AWS::RDS::DBInstance', {
			Properties: Match.objectLike({
				Engine: 'postgres',
				EngineVersion: '16',
				DBInstanceClass: 'db.t4g.micro',
				AllocatedStorage: '20',
				StorageType: 'gp3',
				MultiAZ: false,
				PubliclyAccessible: true,
				StorageEncrypted: true,
				DeletionProtection: false,
				BackupRetentionPeriod: 1,
				DeleteAutomatedBackups: true,
				DBName: 'insightboard',
				DBParameterGroupName: { Ref: parameterGroupId },
			}),
			DeletionPolicy: 'Delete',
			UpdateReplacePolicy: 'Delete',
		});
		template.hasResourceProperties('AWS::RDS::DBParameterGroup', {
			Family: 'postgres16',
			Parameters: Match.objectEquals({ 'rds.force_ssl': '1' }),
		});
	});

	it('opens only TCP 5432 inbound on the database security group', () => {
		template.hasResourceProperties('AWS::EC2::SecurityGroup', {
			SecurityGroupIngress: [
				Match.objectLike({
					CidrIp: '0.0.0.0/0',
					IpProtocol: 'tcp',
					FromPort: 5432,
					ToPort: 5432,
				}),
			],
		});
	});

	it('keeps the function log group for 14 days and deletes it with the stack', () => {
		template.hasResource('AWS::Logs::LogGroup', {
			Properties: {
				LogGroupName: '/aws/lambda/InsightBoardExperiment',
				RetentionInDays: 14,
				Tags: Match.anyValue(),
			},
			DeletionPolicy: 'Delete',
			UpdateReplacePolicy: 'Delete',
		});
		template.hasResourceProperties('AWS::Lambda::Function', {
			FunctionName: 'InsightBoardExperiment',
			LoggingConfig: {
				LogGroup: { Ref: logicalIdOf('AWS::Logs::LogGroup', 'FunctionLogGroup') },
			},
		});
	});

	it('generates both secrets in Secrets Manager, URL-safe, deleted with the stack', () => {
		template.allResources('AWS::SecretsManager::Secret', {
			Properties: Match.objectLike({ SecretString: Match.absent() }),
			DeletionPolicy: 'Delete',
			UpdateReplacePolicy: 'Delete',
		});

		const dbSecretId = logicalIdOf('AWS::SecretsManager::Secret', 'DatabaseSecret');
		const dbGenerate = resources[dbSecretId]!.Properties!['GenerateSecretString'];
		assert.equal(dbGenerate.GenerateStringKey, 'password');
		const excluded: string = dbGenerate.ExcludeCharacters;
		for (const reserved of ":/?#[]@!$&'()*+,;=% \"<>\\^`{|}") {
			assert.ok(excluded.includes(reserved), `password excludes ${JSON.stringify(reserved)}`);
		}

		const appSecretId = logicalIdOf('AWS::SecretsManager::Secret', 'AppSecret');
		const appGenerate = resources[appSecretId]!.Properties!['GenerateSecretString'];
		assert.equal(appGenerate.GenerateStringKey, 'JWT_SECRET');
		assert.equal(appGenerate.ExcludePunctuation, true);
		assert.ok(appGenerate.PasswordLength >= 32);
		assert.deepEqual(JSON.parse(appGenerate.SecretStringTemplate), {
			GITHUB_API_TOKEN: '',
			OPENWEATHER_API_KEY: '',
			NEWSAPI_API_KEY: '',
		});

		const dbId = logicalIdOf('AWS::RDS::DBInstance', 'Database');
		assert.equal(
			render(resources[dbId]!.Properties!['MasterUserPassword']),
			`{{resolve:secretsmanager:<Ref:${dbSecretId}>:SecretString:password::}}`,
		);
	});

	it('tags every taggable resource with the project tags', () => {
		const untaggable = new Set([
			'AWS::EC2::Route',
			'AWS::EC2::SubnetRouteTableAssociation',
			'AWS::EC2::VPCGatewayAttachment',
			'AWS::IAM::Policy',
			'AWS::Lambda::Permission',
			'AWS::Lambda::Url',
			'AWS::SecretsManager::SecretTargetAttachment',
		]);
		for (const [id, resource] of Object.entries(resources)) {
			if (untaggable.has(resource.Type)) continue;
			const tags: { Key: string; Value: string }[] = resource.Properties?.['Tags'] ?? [];
			for (const [key, value] of Object.entries(PROJECT_TAGS)) {
				assert.ok(
					tags.some((t) => t.Key === key && t.Value === value),
					`${id} (${resource.Type}) carries ${key}=${value}`,
				);
			}
		}
	});

	it('is environment-agnostic: no account ID, region, or context lookup', () => {
		const json = JSON.stringify(template.toJSON());
		assert.doesNotMatch(json, /(?<!\d)\d{12}(?!\d)/);
		assert.doesNotMatch(
			json,
			/\b(us|eu|ap|sa|ca|me|af|il|mx|cn)(-gov|-iso[a-z]?)?-(north|south|east|west|central|northeast|northwest|southeast|southwest)-\d\b/,
		);
		const artifact = assembly.getStackByName(STACK_NAME);
		assert.equal(artifact.environment.account, 'unknown-account');
		assert.equal(artifact.environment.region, 'unknown-region');
		assert.equal(assembly.manifest.missing, undefined);
	});

	it('synthesises with zero unacknowledged findings, each acknowledgement citing a decision', () => {
		assert.deepEqual(findSynthProblems(assembly, STACK_NAME), []);
		const acknowledgements = Object.values(resources).flatMap(
			(r) => r.Metadata?.cdk_nag?.rules_to_suppress ?? [],
		);
		// CDK itself acknowledges W3010 (Fn::GetAZs) at stack level; ours cite a decision.
		const ours = acknowledgements.filter((a) => a.id !== 'CloudFormation-Validate::W3010');
		assert.ok(ours.length > 0);
		for (const { id, reason } of ours) {
			assert.match(reason, /^Decisions? \d+/, id);
		}
	});

	it('ignores only the two assessed cfn-lint warnings, each on its own resource', () => {
		const ignores = Object.entries(resources)
			.filter(([, r]) => r.Metadata?.['cfn-lint'] !== undefined)
			.map(([, r]) => [r.Type, r.Metadata!['cfn-lint']!.config.ignore_checks]);
		assert.deepEqual(ignores, [
			['AWS::RDS::DBInstance', ['W3691']],
			['AWS::Lambda::Function', ['W3005']],
		]);
	});
});

describe('synth gate', () => {
	it('reports an unacknowledged cdk-nag finding', () => {
		const fixture = makeFixture();
		try {
			const { app, stack } = buildApp(fixture);
			new sqs.Queue(stack, 'UncheckedQueue');
			const problems = findSynthProblems(app.synth(), STACK_NAME);
			assert.ok(
				problems.some((p) => p.includes('AwsSolutions-SQS') && p.includes('UncheckedQueue')),
				problems.join('\n'),
			);
		} finally {
			fixture.dispose();
		}
	});
});
