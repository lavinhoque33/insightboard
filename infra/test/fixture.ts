import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

/**
 * A throwaway directory holding an empty but valid zip (end-of-central-directory
 * record only) that stands in for the Lambda package, plus a cloud assembly
 * output directory. `dispose` removes both.
 */
export function makeFixture(): {
	lambdaAssetPath: string;
	outdir: string;
	dispose: () => void;
} {
	const dir = mkdtempSync(path.join(tmpdir(), 'insightboard-infra-test-'));
	const lambdaAssetPath = path.join(dir, 'insightboard-lambda.zip');
	const emptyZip = Buffer.alloc(22);
	emptyZip.writeUInt32LE(0x06054b50, 0);
	writeFileSync(lambdaAssetPath, emptyZip);
	return {
		lambdaAssetPath,
		outdir: path.join(dir, 'cdk.out'),
		dispose: () => rmSync(dir, { recursive: true, force: true }),
	};
}
