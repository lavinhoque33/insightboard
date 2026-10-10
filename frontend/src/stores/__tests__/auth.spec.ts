import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useAuthStore } from '../auth';
import { useDashboardStore } from '../dashboard';
import { authApi } from '../../api/auth';
import { storeSession, testUser, tokenExpiringIn } from '../../test/session';

vi.mock('../../api/auth', () => ({
	authApi: { login: vi.fn(), register: vi.fn(), getCurrentUser: vi.fn() },
}));

describe('auth store', () => {
	beforeEach(() => {
		localStorage.clear();
		setActivePinia(createPinia());
	});

	afterEach(() => {
		vi.mocked(authApi.login).mockReset();
		vi.mocked(authApi.getCurrentUser).mockReset();
	});

	it('restores an unexpired stored session', () => {
		storeSession(tokenExpiringIn(3600));

		const store = useAuthStore();

		expect(store.isAuthenticated).toBe(true);
		expect(store.user?.email).toBe(testUser.email);
	});

	it('drops a stored session whose token has expired', () => {
		storeSession(tokenExpiringIn(-1));

		const store = useAuthStore();

		expect(store.isAuthenticated).toBe(false);
		expect(localStorage.getItem('auth_token')).toBeNull();
		expect(localStorage.getItem('user')).toBeNull();
	});

	it('ends the session when the token expires while signed in', () => {
		vi.useFakeTimers();
		storeSession(tokenExpiringIn(60));
		const store = useAuthStore();
		expect(store.checkSession()).toBe(true);

		vi.advanceTimersByTime(61_000);

		expect(store.checkSession()).toBe(false);
		expect(localStorage.getItem('auth_token')).toBeNull();
		vi.useRealTimers();
	});

	it('persists the session after login', async () => {
		const token = tokenExpiringIn(3600);
		vi.mocked(authApi.login).mockResolvedValue({ token, user: testUser });
		const store = useAuthStore();

		await store.login(testUser.email, 'password123');

		expect(store.isAuthenticated).toBe(true);
		expect(localStorage.getItem('auth_token')).toBe(token);
	});

	it('keeps field errors from a 422 login for the form', async () => {
		vi.mocked(authApi.login).mockRejectedValue({
			status: 422,
			message: 'Validation failed',
			fieldErrors: [{ field: 'email', message: 'must be a valid email' }],
		});
		const store = useAuthStore();

		await expect(store.login('x', 'y')).rejects.toBeTruthy();

		expect(store.isAuthenticated).toBe(false);
		expect(store.loading).toBe(false);
		expect(store.fieldErrors).toEqual([
			{ field: 'email', message: 'must be a valid email' },
		]);
	});

	it('logout clears the session and every user-data store', () => {
		storeSession(tokenExpiringIn(3600));
		const auth = useAuthStore();
		const dashboards = useDashboardStore();
		dashboards.dashboards = [
			{
				id: 'd1',
				user_id: testUser.id,
				name: 'Ops',
				layout: { widgets: [] },
				created_at: '',
				updated_at: '',
			},
		];
		dashboards.current = dashboards.dashboards[0] ?? null;
		dashboards.listError = 'stale';

		auth.logout();

		expect(auth.isAuthenticated).toBe(false);
		expect(localStorage.getItem('auth_token')).toBeNull();
		expect(dashboards.dashboards).toEqual([]);
		expect(dashboards.current).toBeNull();
		expect(dashboards.listError).toBeNull();
	});

	it('keeps a restored session when the server cannot be reached', async () => {
		storeSession(tokenExpiringIn(3600));
		vi.mocked(authApi.getCurrentUser).mockRejectedValue({
			status: 0,
			message: 'No response from server.',
		});
		const store = useAuthStore();

		await store.fetchCurrentUser();

		expect(store.isAuthenticated).toBe(true);
	});
});
