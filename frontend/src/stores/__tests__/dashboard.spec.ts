import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useDashboardStore } from '../dashboard';
import { dashboardApi } from '../../api/dashboard';
import type { Dashboard } from '../../types';

vi.mock('../../api/dashboard', () => ({
	dashboardApi: {
		list: vi.fn(),
		get: vi.fn(),
		create: vi.fn(),
		update: vi.fn(),
		delete: vi.fn(),
	},
}));

const ID = '6f1c2a8e-3b4d-4e5f-8a9b-0c1d2e3f4a5b';

const dashboard = (overrides: Partial<Dashboard> = {}): Dashboard => ({
	id: ID,
	user_id: 'u1',
	name: 'Operations',
	layout: { widgets: [] },
	created_at: '2026-10-10T00:00:00Z',
	updated_at: '2026-10-10T00:00:00Z',
	...overrides,
});

describe('dashboard store', () => {
	beforeEach(() => {
		setActivePinia(createPinia());
		vi.mocked(dashboardApi.get).mockReset();
		vi.mocked(dashboardApi.list).mockReset();
		vi.mocked(dashboardApi.update).mockReset();
	});

	it('loads one dashboard by id', async () => {
		vi.mocked(dashboardApi.get).mockResolvedValue(dashboard());
		const store = useDashboardStore();

		await store.fetchDashboard(ID);

		expect(dashboardApi.get).toHaveBeenCalledWith(ID);
		expect(dashboardApi.list).not.toHaveBeenCalled();
		expect(store.current?.name).toBe('Operations');
		expect(store.currentLoading).toBe(false);
	});

	it('reports an unknown or foreign id (404) as not found', async () => {
		vi.mocked(dashboardApi.get).mockRejectedValue({
			status: 404,
			message: 'Dashboard not found',
		});
		const store = useDashboardStore();

		await store.fetchDashboard(ID);

		expect(store.currentNotFound).toBe(true);
		expect(store.currentError).toBeNull();
		expect(store.current).toBeNull();
	});

	it('treats a value that is not an id as not found without a request', async () => {
		const store = useDashboardStore();

		await store.fetchDashboard('not-a-dashboard');

		expect(store.currentNotFound).toBe(true);
		expect(dashboardApi.get).not.toHaveBeenCalled();
	});

	it('keeps other load failures as a retryable error', async () => {
		vi.mocked(dashboardApi.get).mockRejectedValueOnce({
			status: 0,
			message: 'No response from server. Please check your connection.',
		});
		const store = useDashboardStore();

		await store.fetchDashboard(ID);
		expect(store.currentNotFound).toBe(false);
		expect(store.currentError).toContain('No response from server');

		vi.mocked(dashboardApi.get).mockResolvedValueOnce(dashboard());
		await store.fetchDashboard(ID);
		expect(store.currentError).toBeNull();
		expect(store.current?.id).toBe(ID);
	});

	it('ignores a response for a dashboard that is no longer requested', async () => {
		let resolveFirst: (d: Dashboard) => void = () => {};
		vi.mocked(dashboardApi.get)
			.mockReturnValueOnce(new Promise((r) => (resolveFirst = r)))
			.mockResolvedValueOnce(dashboard({ name: 'Second' }));
		const store = useDashboardStore();

		const first = store.fetchDashboard(ID);
		await store.fetchDashboard(ID);
		resolveFirst(dashboard({ name: 'First (stale)' }));
		await first;

		expect(store.current?.name).toBe('Second');
	});

	it('shows a list load failure', async () => {
		vi.mocked(dashboardApi.list).mockRejectedValue({
			status: 500,
			message: 'An error occurred',
		});
		const store = useDashboardStore();

		await store.fetchDashboards();

		expect(store.listError).toBe('An error occurred');
		expect(store.listLoading).toBe(false);
	});

	it('updates the list and the open dashboard after a save', async () => {
		const store = useDashboardStore();
		store.dashboards = [dashboard()];
		store.current = dashboard();
		vi.mocked(dashboardApi.update).mockResolvedValue(
			dashboard({ name: 'Renamed' }),
		);

		await store.updateDashboard(ID, { name: 'Renamed' });

		expect(dashboardApi.update).toHaveBeenCalledWith(ID, { name: 'Renamed' });
		expect(store.dashboards[0]?.name).toBe('Renamed');
		expect(store.current?.name).toBe('Renamed');
	});
});
