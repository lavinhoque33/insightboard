/**
 * Synth guardrail (M6 contract section 6): synthesis must never run with AWS
 * credentials or a profile in reach, so nothing can turn into a control-plane
 * call by accident.
 */
export const FORBIDDEN_AWS_VARIABLES = [
	'AWS_ACCESS_KEY_ID',
	'AWS_SECRET_ACCESS_KEY',
	'AWS_SESSION_TOKEN',
	'AWS_PROFILE',
	'AWS_WEB_IDENTITY_TOKEN_FILE',
] as const;

/** Names of forbidden variables that are set, even to an empty value. */
export function findAwsCredentialVariables(env: NodeJS.ProcessEnv): string[] {
	return FORBIDDEN_AWS_VARIABLES.filter((name) => env[name] !== undefined);
}
