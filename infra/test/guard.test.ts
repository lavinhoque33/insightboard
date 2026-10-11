import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { describe, it } from 'node:test';
import { FORBIDDEN_AWS_VARIABLES, findAwsCredentialVariables } from '../src/guard';
import { makeFixture } from './fixture';

const SYNTH_SCRIPT = path.resolve(__dirname, '..', 'src', 'synth.js');

/** The current environment without any AWS credential variable or LAMBDA_ASSET. */
function cleanEnv(): NodeJS.ProcessEnv {
	const env = { ...process.env };
	for (const name of [...FORBIDDEN_AWS_VARIABLES, 'LAMBDA_ASSET']) {
		delete env[name];
	}
	return env;
}

describe('credential guard', () => {
	it('passes an environment without AWS credential variables', () => {
		assert.deepEqual(findAwsCredentialVariables(cleanEnv()), []);
	});

	it('flags each credential variable, even when empty', () => {
		for (const name of FORBIDDEN_AWS_VARIABLES) {
			assert.deepEqual(findAwsCredentialVariables({ ...cleanEnv(), [name]: '' }), [name]);
		}
	});

	it('makes the synth script refuse to run when a credential variable is set', () => {
		const fixture = makeFixture();
		try {
			const result = spawnSync(process.execPath, [SYNTH_SCRIPT], {
				env: {
					...cleanEnv(),
					AWS_ACCESS_KEY_ID: 'dummy',
					LAMBDA_ASSET: fixture.lambdaAssetPath,
				},
				encoding: 'utf8',
			});
			assert.equal(result.status, 1);
			assert.match(result.stderr, /synth refused: AWS_ACCESS_KEY_ID set/);
			assert.equal(result.stdout, '');
		} finally {
			fixture.dispose();
		}
	});

	it('makes the synth script fail clearly when the Lambda package is missing', () => {
		const result = spawnSync(process.execPath, [SYNTH_SCRIPT], {
			env: { ...cleanEnv(), LAMBDA_ASSET: '/nonexistent/insightboard-lambda.zip' },
			encoding: 'utf8',
		});
		assert.equal(result.status, 1);
		assert.match(result.stderr, /Lambda package not found at \/nonexistent\/insightboard-lambda\.zip/);
	});
});
