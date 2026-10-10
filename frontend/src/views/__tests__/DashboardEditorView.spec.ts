import { flushPromises, mount } from '@vue/test-utils';
import type { VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createMemoryHistory, createRouter, RouterView } from 'vue-router';
import type { Router } from 'vue-router';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Mock } from 'vitest';
import { defineComponent, h } from 'vue';
import DashboardEditorView from '../DashboardEditorView.vue';
import { dashboardApi } from '../../api/dashboard';
import type { DashboardChanges } from '../../api/dashboard';
import { initializeWidgets } from '../../composables/widgetRegistry';
import { storeSession, tokenExpiringIn } from '../../test/session';
import type { Dashboard } from '../../types';
import type { ContractWidgetLayout as WidgetLayout } from '../../contracts/apiContract';

vi.mock('../../api/dashboard', () => ({
	dashboardApi: {
		list: vi.fn(),
		get: vi.fn(),
		create: vi.fn(),
		update: vi.fn(),
		delete: vi.fn(),
	},
}));

// Widgets fetch their data on mount; never resolve so nothing leaves the test
vi.mock('../../api/widgets', () => {
	const never = () => new Promise(() => {});
	return {
		fetchGitHubData: vi.fn(never),
		fetchWeatherData: vi.fn(never),
		fetchNewsData: vi.fn(never),
		fetchCryptoData: vi.fn(never),
		fetchStatusData: vi.fn(never),
	};
});

// GridStack positions the nodes Vue renders; the fake records what it is told
const gridCtx = vi.hoisted(() => ({
	instances: [] as {
		makeWidget: Mock;
		removeWidget: Mock;
		destroy: Mock;
		setStatic: Mock;
		handlers: Record<string, (event: Event, nodes: unknown[]) => void>;
		column: number;
	}[],
}));

vi.mock('gridstack', () => ({
	GridStack: {
		init: vi.fn(() => {
			const instance = {
				makeWidget: vi.fn(),
				removeWidget: vi.fn(),
				destroy: vi.fn(),
				setStatic: vi.fn(),
				handlers: {} as Record<
					string,
					(event: Event, nodes: unknown[]) => void
				>,
				column: 12,
				on(name: string, cb: (event: Event, nodes: unknown[]) => void) {
					instance.handlers[name] = cb;
					return instance;
				},
				getColumn: () => instance.column,
			};
			gridCtx.instances.push(instance);
			return instance;
		}),
	},
}));

const ID = '6f1c2a8e-3b4d-4e5f-8a9b-0c1d2e3f4a5b';

const weather: WidgetLayout = {
	id: 'widget-weather',
	type: 'weather',
	x: 0,
	y: 0,
	w: 3,
	h: 3,
	config: { city: 'London' },
};
const news: WidgetLayout = {
	id: 'widget-news',
	type: 'news',
	x: 3,
	y: 0,
	w: 4,
	h: 4,
	config: { topic: 'science' },
};

const dashboard = (widgets: WidgetLayout[]): Dashboard => ({
	id: ID,
	user_id: 'u1',
	name: 'Operations',
	layout: { widgets },
	created_at: '2026-10-10T00:00:00Z',
	updated_at: '2026-10-10T00:00:00Z',
});

// The page the editor leaves for, and the app root that renders the routes
const DashboardList = { render: () => h('p', 'Dashboard list') };
const Host = defineComponent({ render: () => h(RouterView) });

let router: Router;
let wrapper: VueWrapper;

async function mountEditor() {
	router = createRouter({
		history: createMemoryHistory(),
		routes: [
			{ path: '/dashboards', component: DashboardList },
			{ path: '/dashboards/:id', component: DashboardEditorView },
		],
	});
	await router.push(`/dashboards/${ID}`);
	wrapper = mount(Host, {
		global: { plugins: [router] },
		attachTo: document.body,
	});
	await flushPromises();
}

async function openEditor(widgets: WidgetLayout[] = [weather, news]) {
	vi.mocked(dashboardApi.get).mockResolvedValue(dashboard(widgets));
	await mountEditor();
}

const putCalls = () =>
	vi.mocked(dashboardApi.update).mock.calls as [string, DashboardChanges][];

const lastLayout = () =>
	putCalls()[putCalls().length - 1]?.[1].layout?.widgets ?? [];

const button = (text: string) => {
	const found = wrapper.findAll('button').find((b) => b.text().includes(text));
	if (!found) throw new Error(`no button "${text}"`);
	return found;
};

async function openWidgetMenu(widgetIndex: number) {
	await wrapper
		.findAll('[data-testid="widget-options-btn"]')
		[widgetIndex]!.trigger('click');
}

describe('DashboardEditorView', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		localStorage.clear();
		storeSession(tokenExpiringIn(3600));
		setActivePinia(createPinia());
		initializeWidgets();
		gridCtx.instances.length = 0;
		vi.mocked(dashboardApi.get).mockReset();
		vi.mocked(dashboardApi.list).mockReset();
		vi.mocked(dashboardApi.update)
			.mockReset()
			.mockImplementation(async (_id, changes) => ({
				...dashboard(changes.layout?.widgets ?? []),
				name: changes.name ?? 'Operations',
			}));
	});

	afterEach(() => {
		wrapper?.unmount();
		vi.useRealTimers();
		document.body.innerHTML = '';
	});

	it('loads the one dashboard by id and hands its nodes to GridStack with registry minimum sizes', async () => {
		await openEditor();

		expect(dashboardApi.get).toHaveBeenCalledWith(ID);
		expect(dashboardApi.list).not.toHaveBeenCalled();
		expect(wrapper.get('h1').text()).toBe('Operations');
		const items = wrapper.findAll('.grid-stack-item');
		expect(items).toHaveLength(2);
		// Weather: minSize 2x2; News: minSize 3x3
		expect(items[0]!.attributes()).toMatchObject({
			'gs-id': 'widget-weather',
			'gs-min-w': '2',
			'gs-min-h': '2',
		});
		expect(items[1]!.attributes()).toMatchObject({
			'gs-min-w': '3',
			'gs-min-h': '3',
		});
		expect(gridCtx.instances).toHaveLength(1);
	});

	it('shows a not-found state for an unknown or foreign id', async () => {
		vi.mocked(dashboardApi.get).mockRejectedValue({
			status: 404,
			message: 'Dashboard not found',
		});
		await mountEditor();

		expect(wrapper.text()).toContain('Dashboard not found');
		expect(wrapper.find('.grid-stack').exists()).toBe(false);
		expect(gridCtx.instances).toHaveLength(0);
	});

	it('shows a load failure with a retry that loads the dashboard', async () => {
		vi.mocked(dashboardApi.get).mockRejectedValueOnce({
			status: 0,
			message: 'No response from server. Please check your connection.',
		});
		await mountEditor();

		expect(wrapper.get('[role="alert"]').text()).toContain(
			'Could not load the dashboard',
		);

		vi.mocked(dashboardApi.get).mockResolvedValueOnce(dashboard([weather]));
		await button('Retry').trigger('click');
		await flushPromises();

		expect(wrapper.findAll('.grid-stack-item')).toHaveLength(1);
	});

	it('adds a widget with registry defaults only after its configuration is saved', async () => {
		await openEditor([]);

		await button('Add Widget').trigger('click');
		const choices = wrapper.findAll('#widget-picker button');
		expect(choices.map((c) => c.text())).toEqual([
			expect.stringContaining('GitHub Activity'),
			expect.stringContaining('Weather'),
			expect.stringContaining('News Feed'),
			expect.stringContaining('Cryptocurrency Prices'),
			expect.stringContaining('Status Monitor'),
		]);

		await button('Cryptocurrency Prices').trigger('click');
		// Registry default pre-filled
		expect(
			(wrapper.get('#widget-config-cryptoIds').element as HTMLInputElement)
				.value,
		).toBe('bitcoin,ethereum,cardano');
		expect(wrapper.findAll('.grid-stack-item')).toHaveLength(0);

		await wrapper.get('form').trigger('submit');
		await flushPromises();

		const items = wrapper.findAll('.grid-stack-item');
		expect(items).toHaveLength(1);
		expect(gridCtx.instances[0]!.makeWidget).toHaveBeenCalledWith(
			items[0]!.element,
		);

		await vi.advanceTimersByTimeAsync(2000);
		expect(putCalls()).toHaveLength(1);
		expect(lastLayout()).toEqual([
			expect.objectContaining({
				type: 'crypto',
				x: 0,
				y: 0,
				w: 3,
				h: 4,
				config: { cryptoIds: 'bitcoin,ethereum,cardano' },
			}),
		]);
	});

	it('cancelling a new widget adds nothing and saves nothing', async () => {
		await openEditor([]);

		await button('Add Widget').trigger('click');
		await button('Status Monitor').trigger('click');
		await button('Cancel').trigger('click');
		await vi.advanceTimersByTimeAsync(5000);

		expect(wrapper.findAll('.grid-stack-item')).toHaveLength(0);
		expect(putCalls()).toHaveLength(0);
	});

	it('configures an existing widget from its current configuration and keeps the rest', async () => {
		await openEditor();

		await openWidgetMenu(0);
		await button('Configure').trigger('click');
		const city = wrapper.get('#widget-config-city');
		expect((city.element as HTMLInputElement).value).toBe('London');

		await city.setValue('Paris');
		await wrapper.get('form').trigger('submit');
		await vi.advanceTimersByTimeAsync(2000);

		expect(putCalls()).toHaveLength(1);
		expect(lastLayout()).toEqual([
			{ ...weather, config: { city: 'Paris' } },
			news,
		]);
	});

	it('removes a widget through GridStack without letting it remove the node', async () => {
		await openEditor();
		const node = wrapper.findAll('.grid-stack-item')[1]!.element;

		await openWidgetMenu(1);
		await button('Remove').trigger('click');

		expect(gridCtx.instances[0]!.removeWidget).toHaveBeenCalledWith(
			node,
			false,
		);
		expect(wrapper.findAll('.grid-stack-item')).toHaveLength(1);
		await vi.advanceTimersByTimeAsync(2000);
		expect(lastLayout()).toEqual([weather]);
	});

	it('saves moved positions from GridStack', async () => {
		await openEditor();

		gridCtx.instances[0]!.handlers.change!(new Event('change'), [
			{ id: 'widget-weather', x: 6, y: 2, w: 4, h: 3 },
		]);
		await vi.advanceTimersByTimeAsync(2000);

		expect(lastLayout()[0]).toMatchObject({ x: 6, y: 2, w: 4, h: 3 });
	});

	it('does not save single-column positions at narrow widths', async () => {
		await openEditor();
		const grid = gridCtx.instances[0]!;
		grid.column = 1;
		grid.handlers.resizecontent!(new Event('resize'), []);
		await flushPromises();

		expect(grid.setStatic).toHaveBeenLastCalledWith(true);
		expect(wrapper.text()).toContain('available on wider screens');

		grid.handlers.change!(new Event('change'), [
			{ id: 'widget-news', x: 0, y: 3, w: 1, h: 4 },
		]);
		await vi.advanceTimersByTimeAsync(5000);
		expect(putCalls()).toHaveLength(0);
	});

	it('shows a failed save with a retry and keeps the edit until it is saved', async () => {
		await openEditor();
		vi.mocked(dashboardApi.update).mockRejectedValueOnce({
			status: 0,
			message: 'No response from server. Please check your connection.',
		});

		await openWidgetMenu(1);
		await button('Remove').trigger('click');
		await vi.advanceTimersByTimeAsync(2000);
		await flushPromises();

		const alert = wrapper.get('[role="alert"]');
		expect(alert.text()).toContain('Your changes are not saved');
		expect(alert.text()).toContain('No response from server');

		await button('Retry').trigger('click');
		await flushPromises();

		expect(putCalls()).toHaveLength(2);
		expect(lastLayout()).toEqual([weather]);
		expect(wrapper.find('[role="alert"]').exists()).toBe(false);
		expect(wrapper.get('[data-testid="save-status"]').text()).toBe(
			'All changes saved',
		);
	});

	it('renames the dashboard through the save queue and shows the new name', async () => {
		await openEditor();

		await wrapper.get('[aria-label="Rename dashboard"]').trigger('click');
		await wrapper.get('#dashboard-name').setValue('  Team metrics ');
		await wrapper.get('form').trigger('submit');
		await flushPromises();

		expect(putCalls()).toHaveLength(1);
		expect(putCalls()[0]![1].name).toBe('Team metrics');
		expect(wrapper.get('h1').text()).toBe('Team metrics');
	});

	it('saves pending edits before leaving and sends nothing afterwards', async () => {
		await openEditor();
		await openWidgetMenu(1);
		await button('Remove').trigger('click');

		await router.push('/dashboards');
		await flushPromises();

		expect(router.currentRoute.value.path).toBe('/dashboards');
		expect(putCalls()).toHaveLength(1);
		expect(lastLayout()).toEqual([weather]);

		await vi.advanceTimersByTimeAsync(10_000);
		expect(putCalls()).toHaveLength(1);
		expect(gridCtx.instances[0]!.destroy).toHaveBeenCalledWith(false);
	});

	it('sends no request when leaving without edits', async () => {
		await openEditor();

		await router.push('/dashboards');
		await vi.advanceTimersByTimeAsync(10_000);

		expect(putCalls()).toHaveLength(0);
	});

	it('sends no request after unmounting with a save still scheduled', async () => {
		await openEditor();
		await openWidgetMenu(1);
		await button('Remove').trigger('click');

		wrapper.unmount();
		await vi.advanceTimersByTimeAsync(10_000);

		expect(putCalls()).toHaveLength(0);
	});

	it('stays on the page when the save before leaving fails and the user keeps the edits', async () => {
		await openEditor();
		vi.mocked(dashboardApi.update).mockRejectedValueOnce({
			status: 500,
			message: 'An error occurred',
		});
		const confirm = vi.spyOn(window, 'confirm').mockReturnValue(false);
		await openWidgetMenu(1);
		await button('Remove').trigger('click');

		await router.push('/dashboards');
		await flushPromises();

		expect(confirm).toHaveBeenCalledOnce();
		expect(router.currentRoute.value.path).toBe(`/dashboards/${ID}`);
		expect(wrapper.get('[role="alert"]').text()).toContain(
			'Your changes are not saved',
		);
		confirm.mockRestore();
	});
});
