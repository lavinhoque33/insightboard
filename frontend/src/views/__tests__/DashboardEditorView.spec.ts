import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { nextTick } from 'vue';
import DashboardEditorView from '../DashboardEditorView.vue';
import type { Dashboard } from '../../types';
import type { WidgetType } from '../../stores/widgets';

const mockCtx = vi.hoisted(() => {
	const dashboard: Dashboard = {
		id: 'dashboard-test-1',
		user_id: 'user-test-1',
		layout: { widgets: [] },
		settings: { title: 'Test Dashboard', theme: 'light' },
		created_at: '2026-03-03T00:00:00Z',
		updated_at: '2026-03-03T00:00:00Z',
	};

	const widgetType: WidgetType = {
		id: 'status',
		name: 'Status Monitor',
		description: 'Monitor website uptime and response times',
		icon: '🔍',
		component: {
			template: '<div class="test-widget">Widget</div>',
		},
		defaultConfig: {},
		configSchema: [
			{
				key: 'urls',
				label: 'URLs to Monitor',
				type: 'text',
				required: true,
				placeholder: 'https://example.com',
			},
		],
		defaultSize: { w: 4, h: 3 },
		refreshInterval: 60,
	};

	return {
		dashboard,
		dashboardStore: {
			dashboards: [dashboard],
			fetchDashboards: vi.fn(async () => {}),
			updateDashboard: vi.fn(async () => dashboard),
		},
		widgetStore: {
			availableWidgets: [
				{
					id: widgetType.id,
					name: widgetType.name,
					description: widgetType.description,
					icon: widgetType.icon,
				},
			],
			createWidgetInstance: vi.fn(() => ({
				id: 'widget-test-1',
				type: 'status',
				config: {},
				layout: { x: 0, y: 0, w: 4, h: 3 },
			})),
			getWidgetType: vi.fn((id: string) =>
				id === 'status' ? widgetType : undefined,
			),
		},
		router: {
			push: vi.fn(),
		},
	};
});

vi.mock('vue-router', () => ({
	useRoute: () => ({ params: { id: mockCtx.dashboard.id } }),
	useRouter: () => mockCtx.router,
}));

vi.mock('../../stores/dashboard', () => ({
	useDashboardStore: () => mockCtx.dashboardStore,
}));

vi.mock('../../stores/widgets', () => ({
	useWidgetStore: () => mockCtx.widgetStore,
}));

vi.mock('gridstack', () => ({
	GridStack: {
		init: vi.fn(() => ({
			on: vi.fn(),
			addWidget: vi.fn(),
			getGridItems: vi.fn(() => []),
			removeWidget: vi.fn(),
			destroy: vi.fn(),
		})),
	},
}));

const findButtonByText = (root: ReturnType<typeof mount>, text: string) => {
	return root.findAll('button').find((button) => button.text().includes(text));
};

describe('DashboardEditorView', () => {
	it('does not create a widget until configuration is saved', async () => {
		vi.useFakeTimers();
		const wrapper = mount(DashboardEditorView, {
			attachTo: document.body,
		});

		await nextTick();
		await Promise.resolve();
		vi.advanceTimersByTime(150);
		await nextTick();

		const addWidgetButton = findButtonByText(wrapper, 'Add Widget');
		expect(addWidgetButton).toBeTruthy();
		await addWidgetButton!.trigger('click');

		const statusWidgetButton = findButtonByText(wrapper, 'Status Monitor');
		expect(statusWidgetButton).toBeTruthy();
		await statusWidgetButton!.trigger('click');
		await nextTick();

		expect(wrapper.text()).toContain('Configure Status Monitor');
		expect(wrapper.findAll('.grid-stack-item')).toHaveLength(0);

		const cancelButton = findButtonByText(wrapper, 'Cancel');
		expect(cancelButton).toBeTruthy();
		await cancelButton!.trigger('click');
		await nextTick();

		expect(wrapper.findAll('.grid-stack-item')).toHaveLength(0);
		expect(wrapper.text()).toContain('Build Your Dashboard');

		wrapper.unmount();
		vi.useRealTimers();
	});

	it('creates the widget only after saving valid configuration', async () => {
		vi.useFakeTimers();
		const wrapper = mount(DashboardEditorView, {
			attachTo: document.body,
		});

		await nextTick();
		await Promise.resolve();
		vi.advanceTimersByTime(150);
		await nextTick();

		const addWidgetButton = findButtonByText(wrapper, 'Add Widget');
		expect(addWidgetButton).toBeTruthy();
		await addWidgetButton!.trigger('click');

		const statusWidgetButton = findButtonByText(wrapper, 'Status Monitor');
		expect(statusWidgetButton).toBeTruthy();
		await statusWidgetButton!.trigger('click');
		await nextTick();

		expect(wrapper.findAll('.grid-stack-item')).toHaveLength(0);

		const configInput = wrapper.get('input#urls');
		await configInput.setValue('https://example.com');

		const saveButton = findButtonByText(wrapper, 'Save Changes');
		expect(saveButton).toBeTruthy();
		await saveButton!.trigger('click');
		await nextTick();

		expect(wrapper.findAll('.grid-stack-item')).toHaveLength(1);

		wrapper.unmount();
		vi.useRealTimers();
	});
});
