import { createHash } from 'node:crypto';
import { existsSync, readFileSync, rmSync, statSync } from 'node:fs';
import path from 'node:path';
import type { cx_api } from 'aws-cdk-lib';
import { VALIDATION_REPORT_FILE } from 'aws-cdk-lib/cx-api';
import { buildApp } from './app';
import { findAwsCredentialVariables } from './guard';
import { STACK_NAME } from './stack';

/** The `infra/` package root (this file runs from `dist/src/`). */
const INFRA_DIR = path.resolve(__dirname, '..', '..');
const DEFAULT_LAMBDA_ASSET = '../backend/target/lambda/insightboard-lambda.zip';

/** The parts of CDK's `validation-report.json` the gate reads. */
interface ValidationReport {
	pluginReports: {
		pluginName: string;
		conclusion: string;
		violations: {
			ruleName: string;
			severity: string;
			description: string;
			violatingConstructs: { constructPath: string }[];
		}[];
	}[];
}

/**
 * Everything that fails the synth (contract section 5): unacknowledged
 * violations of any severity from the validation plugins (cdk-nag AwsSolutions
 * and CDK's built-in CloudFormation validation), a failed plugin, and error or
 * warning annotations on the stack.
 */
export function findSynthProblems(
	assembly: cx_api.CloudAssembly,
	stackName: string,
): string[] {
	const reportPath = path.join(assembly.directory, VALIDATION_REPORT_FILE);
	if (!existsSync(reportPath)) {
		return [`no ${VALIDATION_REPORT_FILE} in the cloud assembly: validation did not run`];
	}
	const report = JSON.parse(readFileSync(reportPath, 'utf8')) as ValidationReport;
	const problems = report.pluginReports.flatMap((plugin) => [
		...(plugin.conclusion === 'success'
			? []
			: [`[${plugin.pluginName}] conclusion: ${plugin.conclusion}`]),
		...plugin.violations.map(
			(v) =>
				`[${plugin.pluginName}] ${v.severity} ${v.ruleName} at ` +
				`${v.violatingConstructs.map((c) => c.constructPath).join(', ')}: ${v.description}`,
		),
	]);
	for (const message of assembly.getStackByName(stackName).messages) {
		if (message.level === 'error' || message.level === 'warning') {
			problems.push(
				`[annotation] ${message.level} at ${message.id}: ${String(message.entry.data)}`,
			);
		}
	}
	return problems;
}

function main(): number {
	const credentialVariables = findAwsCredentialVariables(process.env);
	if (credentialVariables.length > 0) {
		console.error(
			`synth refused: ${credentialVariables.join(', ')} set. ` +
				'M6 synthesis runs without AWS credentials or profiles; unset them and retry.',
		);
		return 1;
	}

	const lambdaAssetPath = path.resolve(
		INFRA_DIR,
		process.env['LAMBDA_ASSET'] || DEFAULT_LAMBDA_ASSET,
	);
	if (!existsSync(lambdaAssetPath) || !statSync(lambdaAssetPath).isFile()) {
		console.error(
			`synth failed: Lambda package not found at ${lambdaAssetPath}. ` +
				'Run `make lambda-build` or set LAMBDA_ASSET to an existing .zip file.',
		);
		return 1;
	}

	const outdir = path.join(INFRA_DIR, 'cdk.out');
	rmSync(outdir, { recursive: true, force: true });
	const { app } = buildApp({ lambdaAssetPath, outdir });
	const assembly = app.synth();

	const problems = findSynthProblems(assembly, STACK_NAME);
	if (problems.length > 0) {
		console.error(
			`synth failed: ${problems.length} unacknowledged finding(s) or annotation(s):\n${problems.join('\n')}`,
		);
		return 1;
	}

	const templatePath = path.join(outdir, `${STACK_NAME}.template.json`);
	const templateBytes = readFileSync(templatePath);
	const template = JSON.parse(templateBytes.toString('utf8')) as {
		Resources: Record<
			string,
			{
				Metadata?: {
					cdk_nag?: { rules_to_suppress: { id: string; reason: string }[] };
				};
			}
		>;
	};
	// Acknowledgements as written into the template, grouped by rule and reason.
	const acknowledged = new Map<string, string[]>();
	for (const [logicalId, resource] of Object.entries(template.Resources)) {
		for (const rule of resource.Metadata?.cdk_nag?.rules_to_suppress ?? []) {
			const key = `${rule.id}: ${rule.reason}`;
			acknowledged.set(key, [...(acknowledged.get(key) ?? []), logicalId]);
		}
	}
	console.log(`template: ${path.relative(process.cwd(), templatePath)}`);
	console.log(`sha256:   ${createHash('sha256').update(templateBytes).digest('hex')}`);
	console.log(
		'validation: 0 unacknowledged findings (cdk-nag AwsSolutions, CloudFormation Validate)',
	);
	console.log('acknowledged (template Metadata cdk_nag.rules_to_suppress):');
	const resourceCount = Object.keys(template.Resources).length;
	for (const [rule, logicalIds] of acknowledged) {
		const scope =
			logicalIds.length === resourceCount
				? `all ${resourceCount} resources (stack-level)`
				: logicalIds.join(', ');
		console.log(`  ${rule}\n    on ${scope}`);
	}
	return 0;
}

if (require.main === module) {
	try {
		process.exitCode = main();
	} catch (error) {
		console.error('synth failed:', error);
		process.exitCode = 1;
	}
}
