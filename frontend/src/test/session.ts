/** A JWT-shaped token whose payload carries `exp`; the signature is not checked. */
export function tokenExpiringIn(seconds: number): string {
	const exp = Math.floor(Date.now() / 1000) + seconds;
	const payload = btoa(JSON.stringify({ sub: 'user-1', exp }))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');
	return `eyJhbGciOiJIUzI1NiJ9.${payload}.signature`;
}

export const testUser = {
	id: '00000000-0000-4000-8000-000000000001',
	email: 'ada@example.com',
	created_at: '2026-10-10T00:00:00Z',
};

/** Store a session the auth store restores on creation. */
export function storeSession(token: string): void {
	localStorage.setItem('auth_token', token);
	localStorage.setItem('user', JSON.stringify(testUser));
}
