/**
 * Read a human-readable message from a caught value. The API client rejects with an
 * `ApiError` object rather than an `Error`, so both shapes are handled.
 */
export function apiErrorMessage(err: unknown, fallback: string): string {
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

/** HTTP status of an `ApiError`, or undefined for anything else. */
export function apiErrorStatus(err: unknown): number | undefined {
	return typeof err === 'object' &&
		err !== null &&
		'status' in err &&
		typeof err.status === 'number'
		? err.status
		: undefined;
}

/**
 * Widget data failures: upstream statuses get fixed user-facing text because the
 * backend body is generic.
 */
const STATUS_MESSAGES: Record<number, string> = {
	429: 'Too many requests. Please wait a moment and try again.',
	502: 'The data provider failed to respond. Please try again later.',
	503: 'This widget is unavailable: the provider key is not configured on the server.',
};

export function errorMessage(err: unknown, fallback: string): string {
	const status = apiErrorStatus(err);
	return (
		(status !== undefined ? STATUS_MESSAGES[status] : undefined) ??
		apiErrorMessage(err, fallback)
	);
}
