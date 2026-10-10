/**
 * Session storage keys and client-side reading of the JWT `exp` claim (seconds
 * since the epoch). The signature is not checked here; the server stays the
 * authority. The client only uses the expiry to stop sending a token it knows
 * the server will reject.
 */

export const TOKEN_KEY = 'auth_token';
export const USER_KEY = 'user';

/** Expiry of `token` in milliseconds since the epoch, or null when unreadable. */
function tokenExpiresAt(token: string): number | null {
	const payload = token.split('.')[1];
	if (!payload) return null;
	try {
		const claims: unknown = JSON.parse(
			atob(payload.replace(/-/g, '+').replace(/_/g, '/')),
		);
		if (
			typeof claims === 'object' &&
			claims !== null &&
			'exp' in claims &&
			typeof claims.exp === 'number'
		) {
			return claims.exp * 1000;
		}
	} catch {
		// Malformed payload: unreadable
	}
	return null;
}

/** A token without a readable `exp` counts as expired: it cannot be trusted. */
export function isTokenExpired(token: string, now = Date.now()): boolean {
	const expiresAt = tokenExpiresAt(token);
	return expiresAt === null || expiresAt <= now;
}
