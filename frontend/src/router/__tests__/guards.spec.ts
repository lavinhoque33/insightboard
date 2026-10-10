import { createPinia, setActivePinia } from 'pinia';
import { createMemoryHistory } from 'vue-router';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createAppRouter, safeRedirect } from '..';
import { useAuthStore } from '../../stores/auth';
import { useDashboardStore } from '../../stores/dashboard';
import { storeSession, tokenExpiringIn } from '../../test/session';

// The guards never reach the network; views are not mounted here
vi.mock('../../api', () => ({ authApi: {}, dashboardApi: {} }));

const EDITOR = '/dashboards/6f1c2a8e-3b4d-4e5f-8a9b-0c1d2e3f4a5b';

describe('router guards', () => {
	beforeEach(() => {
		localStorage.clear();
		setActivePinia(createPinia());
	});

	it('sends a signed-out visitor to login and keeps the destination', async () => {
		const router = createAppRouter(createMemoryHistory());

		await router.push(EDITOR);

		expect(router.currentRoute.value.name).toBe('login');
		expect(router.currentRoute.value.query.redirect).toBe(EDITOR);
	});

	it('lets a signed-in user open a protected page', async () => {
		storeSession(tokenExpiringIn(3600));
		const router = createAppRouter(createMemoryHistory());

		await router.push('/dashboards');

		expect(router.currentRoute.value.name).toBe('dashboards');
	});

	it('treats an expired token as signed out and clears it', async () => {
		vi.useFakeTimers();
		storeSession(tokenExpiringIn(60));
		const router = createAppRouter(createMemoryHistory());
		await router.push('/dashboards');
		expect(router.currentRoute.value.name).toBe('dashboards');

		vi.advanceTimersByTime(61_000);
		await router.push(EDITOR);

		expect(router.currentRoute.value.name).toBe('login');
		expect(router.currentRoute.value.query.redirect).toBe(EDITOR);
		expect(localStorage.getItem('auth_token')).toBeNull();
		vi.useRealTimers();
	});

	it('keeps signed-in users away from login and register', async () => {
		storeSession(tokenExpiringIn(3600));
		const router = createAppRouter(createMemoryHistory());

		await router.push('/login');
		expect(router.currentRoute.value.name).toBe('dashboards');

		await router.push('/register');
		expect(router.currentRoute.value.name).toBe('dashboards');
	});

	it('logout ends the session, resets the stores, and lands on login', async () => {
		storeSession(tokenExpiringIn(3600));
		const router = createAppRouter(createMemoryHistory());
		await router.push('/dashboards');
		useDashboardStore().listError = 'stale';

		await router.push('/logout');

		expect(router.currentRoute.value.name).toBe('login');
		expect(useAuthStore().isAuthenticated).toBe(false);
		expect(useDashboardStore().listError).toBeNull();
	});

	it.each([
		['/dashboards/abc', '/dashboards/abc'],
		['//evil.example', '/dashboards'],
		['/\\evil.example', '/dashboards'],
		['https://evil.example', '/dashboards'],
		[undefined, '/dashboards'],
	])('accepts only same-origin redirects (%s)', (value, expected) => {
		expect(safeRedirect(value)).toBe(expected);
	});
});
