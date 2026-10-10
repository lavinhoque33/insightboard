/**
 * Read a human-readable message from a caught value. The API client rejects with an
 * `ApiError` object rather than an `Error`, so both shapes are handled. Upstream
 * failure statuses get fixed user-facing text because the backend body is generic.
 */
const STATUS_MESSAGES: Record<number, string> = {
	429: 'Too many requests. Please wait a moment and try again.',
	502: 'The data provider failed to respond. Please try again later.',
	503: 'This widget is unavailable: the provider key is not configured on the server.',
};

export function errorMessage(err: unknown, fallback: string): string {
	if (typeof err === 'object' && err !== null) {
		if ('status' in err && typeof err.status === 'number') {
			const mapped = STATUS_MESSAGES[err.status];
			if (mapped) return mapped;
		}
		if ('message' in err && typeof err.message === 'string' && err.message) {
			return err.message;
		}
	}
	return fallback;
}
