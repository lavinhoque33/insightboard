import { App, Tags, Validations } from 'aws-cdk-lib';
import { AwsSolutionsChecks } from 'cdk-nag';
import { InsightBoardExperimentStack, STACK_NAME } from './stack';

export const PROJECT_TAGS = {
	project: 'insightboard',
	purpose: 'm7-experiment',
} as const;

export interface BuildAppOptions {
	/** Absolute path of the Lambda zip package. */
	readonly lambdaAssetPath: string;
	/** Cloud assembly output directory. */
	readonly outdir: string;
}

/**
 * The CDK app: one environment-agnostic stack (no account, region, or lookups),
 * project tags on every taggable resource, analytics off, and the cdk-nag
 * AwsSolutions pack registered as a synthesis validation plugin. The app does
 * not throw on violations itself: the synth script reads the validation report
 * and fails on any unacknowledged error or warning (contract section 5).
 */
export function buildApp(options: BuildAppOptions): {
	app: App;
	stack: InsightBoardExperimentStack;
} {
	const app = new App({
		outdir: options.outdir,
		analyticsReporting: false,
		context: { '@aws-cdk/core:failSynthOnValidationErrors': false },
	});
	const stack = new InsightBoardExperimentStack(app, STACK_NAME, {
		lambdaAssetPath: options.lambdaAssetPath,
		description: 'InsightBoard M7 experiment (synthesised in M6; not deployed)',
	});
	for (const [key, value] of Object.entries(PROJECT_TAGS)) {
		Tags.of(app).add(key, value);
	}
	// writeSuppressionsToCloudFormation records every acknowledgement and its
	// reason in the template's resource Metadata (`cdk_nag.rules_to_suppress`).
	Validations.of(app).addPlugins(
		new AwsSolutionsChecks(app, {
			verbose: true,
			writeSuppressionsToCloudFormation: true,
		}),
	);
	return { app, stack };
}
