/**
 * Read a human-readable message from a caught value. The API client rejects with an
 * `ApiError` object rather than an `Error`, so both shapes are handled.
 */
export function errorMessage(err: unknown, fallback: string): string {
	if (
		typeof err === 'object' &&
		err !== null &&
		'message' in err &&
		typeof err.message === 'string' &&
		err.message
	) {
		return err.message;
	}
	return fallback;
}
