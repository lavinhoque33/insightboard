/** * Dashboard Editor View * Allows users to add, configure, move, and remove
widgets on their dashboard */
<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useDashboardStore } from '../stores/dashboard';
import { useWidgetStore } from '../stores/widgets';
import type { WidgetInstance } from '../stores/widgets';
import type { WidgetType as WidgetKind } from '../types';
import 'gridstack/dist/gridstack.min.css';
import { GridStack } from 'gridstack';
import WidgetConfigModal from '../components/WidgetConfigModal.vue';

const route = useRoute();
const router = useRouter();
const dashboardStore = useDashboardStore();
const widgetStore = useWidgetStore();

const dashboardId = computed(() => route.params.id as string);
const dashboard = computed(() =>
	dashboardStore.dashboards.find((d) => d.id === dashboardId.value),
);

const widgets = ref<WidgetInstance[]>([]);
const gridInstance = ref<GridStack | null>(null);
const gridContainer = ref<HTMLElement | null>(null);
const saving = ref(false);
const showWidgetPicker = ref(false);
const showConfigModal = ref(false);
const configModalWidget = ref<{
	instance: WidgetInstance;
	typeName: string;
} | null>(null);
const autoSaveTimer = ref<number | null>(null);

// Load dashboard data
const loadDashboard = async () => {
	await dashboardStore.fetchDashboards();
	if (dashboard.value && dashboard.value.layout?.widgets) {
		widgets.value = dashboard.value.layout.widgets.map((w) => ({
			id: w.id,
			type: w.type,
			config: w.config || {},
			layout: { x: w.x, y: w.y, w: w.w, h: w.h },
		}));

		// Wait for next tick then initialize grid
		setTimeout(() => {
			initializeGrid();
		}, 100);
	}
};

// Initialize GridStack
const initializeGrid = () => {
	if (!gridContainer.value || gridInstance.value) return;

	gridInstance.value = GridStack.init(
		{
			float: false,
			cellHeight: 80,
			margin: 10,
			minRow: 1,
			column: 12,
			animate: true,
			resizable: {
				handles: 'se, sw',
			},
		},
		gridContainer.value,
	);

	// Listen to change events for auto-save
	gridInstance.value.on('change', () => {
		scheduleAutoSave();
	});
};

// Add widget to dashboard
const addWidget = (typeId: string) => {
	const newWidget = widgetStore.createWidgetInstance(typeId);
	if (!newWidget) return;

	widgets.value.push(newWidget);
	showWidgetPicker.value = false;

	// Add to grid
	if (gridInstance.value) {
		gridInstance.value.addWidget({
			id: newWidget.id,
			x: newWidget.layout.x,
			y: newWidget.layout.y,
			w: newWidget.layout.w,
			h: newWidget.layout.h,
		});
	}

	scheduleAutoSave();
};

// Configure widget
const configureWidget = (widgetId: string) => {
	const widget = widgets.value.find((w) => w.id === widgetId);
	if (!widget) return;

	const widgetType = widgetStore.getWidgetType(widget.type);
	if (!widgetType) return;

	configModalWidget.value = {
		instance: widget,
		typeName: widgetType.name,
	};
	showConfigModal.value = true;
};

// Save widget configuration
const saveWidgetConfig = (config: Record<string, any>) => {
	if (!configModalWidget.value) return;

	const widget = widgets.value.find(
		(w) => w.id === configModalWidget.value!.instance.id,
	);
	if (widget) {
		widget.config = config;
		scheduleAutoSave();
	}

	configModalWidget.value = null;
};

// Remove widget
const removeWidget = (widgetId: string) => {
	widgets.value = widgets.value.filter((w) => w.id !== widgetId);

	if (gridInstance.value) {
		const element = document.getElementById(widgetId);
		if (element) {
			gridInstance.value.removeWidget(element);
		}
	}

	scheduleAutoSave();
};

// Schedule auto-save (debounced)
const scheduleAutoSave = () => {
	if (autoSaveTimer.value) {
		clearTimeout(autoSaveTimer.value);
	}

	autoSaveTimer.value = window.setTimeout(() => {
		saveDashboard();
	}, 2000);
};

// Save dashboard
const saveDashboard = async () => {
	if (!dashboard.value) return;

	saving.value = true;

	try {
		// Update widget positions from GridStack
		if (gridInstance.value) {
			const gridWidgets = gridInstance.value.getGridItems();
			gridWidgets.forEach((el) => {
				const widget = widgets.value.find(
					(w) => w.id === el.getAttribute('id'),
				);
				if (widget) {
					const node = el.gridstackNode;
					if (node) {
						widget.layout.x = node.x || 0;
						widget.layout.y = node.y || 0;
						widget.layout.w = node.w || 4;
						widget.layout.h = node.h || 3;
					}
				}
			});
		}

		await dashboardStore.updateDashboard(dashboard.value.id, {
			widgets: widgets.value.map((w) => ({
				id: w.id,
				type: w.type as WidgetKind,
				x: w.layout.x,
				y: w.layout.y,
				w: w.layout.w,
				h: w.layout.h,
				config: w.config,
			})),
		});
	} catch (error: any) {
		console.error('Failed to save dashboard:', error);
	} finally {
		saving.value = false;
	}
};

// Get widget component
const getWidgetComponent = (typeId: string) => {
	const widgetType = widgetStore.getWidgetType(typeId);
	return widgetType?.component;
};

// Get widget refresh interval
const getRefreshInterval = (typeId: string): number => {
	const widgetType = widgetStore.getWidgetType(typeId);
	return widgetType?.refreshInterval || 0;
};

onMounted(() => {
	loadDashboard();
});

onBeforeUnmount(() => {
	if (gridInstance.value) {
		gridInstance.value.destroy(false);
	}
	if (autoSaveTimer.value) {
		clearTimeout(autoSaveTimer.value);
		saveDashboard(); // Final save on unmount
	}
});
</script>

<template>
	<div class="dashboard-editor h-screen flex flex-col bg-gray-50">
		<!-- Top Bar -->
		<div
			class="bg-white shadow-sm border-b border-gray-200 px-6 py-4 flex items-center justify-between"
		>
			<div class="flex items-center space-x-4">
				<button
					@click="router.push('/dashboards')"
					class="text-gray-600 hover:text-gray-800 transition-colors"
					title="Back to Dashboards"
				>
					<svg
						class="w-6 h-6"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M10 19l-7-7m0 0l7-7m-7 7h18"
						/>
					</svg>
				</button>
				<h1 class="text-2xl font-bold text-gray-800">
					{{ dashboard?.settings?.title || 'Dashboard Editor' }}
				</h1>
				<span
					v-if="saving"
					class="text-sm text-blue-600 flex items-center space-x-1"
				>
					<svg
						class="animate-spin h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
					>
						<circle
							class="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							stroke-width="4"
						></circle>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						></path>
					</svg>
					<span>Saving...</span>
				</span>
			</div>

			<button
				@click="showWidgetPicker = !showWidgetPicker"
				class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors flex items-center space-x-2"
			>
				<svg
					class="w-5 h-5"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 4v16m8-8H4"
					/>
				</svg>
				<span>Add Widget</span>
			</button>
		</div>

		<!-- Widget Picker Dropdown -->
		<div
			v-if="showWidgetPicker"
			class="absolute right-6 top-20 z-50 w-80 bg-white rounded-lg shadow-xl border border-gray-200"
		>
			<div class="p-4 border-b border-gray-200">
				<h3 class="font-semibold text-gray-800">Add Widget</h3>
			</div>
			<div class="max-h-96 overflow-y-auto">
				<button
					v-for="widgetType in widgetStore.availableWidgets"
					:key="widgetType.id"
					@click="addWidget(widgetType.id)"
					class="w-full text-left px-4 py-3 hover:bg-blue-50 transition-colors border-b border-gray-100 last:border-0"
				>
					<div class="flex items-start space-x-3">
						<span class="text-2xl">{{ widgetType.icon }}</span>
						<div class="flex-1">
							<div class="font-medium text-gray-800">
								{{ widgetType.name }}
							</div>
							<div class="text-sm text-gray-600 mt-0.5">
								{{ widgetType.description }}
							</div>
						</div>
					</div>
				</button>
			</div>
		</div>

		<!-- GridStack Container -->
		<div class="flex-1 overflow-auto p-6">
			<div ref="gridContainer" class="grid-stack">
				<div
					v-for="widget in widgets"
					:key="widget.id"
					:id="widget.id"
					class="grid-stack-item"
					:gs-id="widget.id"
					:gs-x="widget.layout.x"
					:gs-y="widget.layout.y"
					:gs-w="widget.layout.w"
					:gs-h="widget.layout.h"
				>
					<div class="grid-stack-item-content">
						<component
							:is="getWidgetComponent(widget.type)"
							:config="widget.config"
							:refresh-interval="getRefreshInterval(widget.type)"
							@configure="configureWidget(widget.id)"
							@remove="removeWidget(widget.id)"
						/>
					</div>
				</div>
			</div>

			<!-- Empty State -->
			<div
				v-if="widgets.length === 0"
				class="flex flex-col items-center justify-center h-full text-center text-gray-500 py-20"
			>
				<svg
					class="w-20 h-20 text-gray-300 mb-4"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1.5"
						d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
					/>
				</svg>
				<h3 class="text-xl font-semibold text-gray-700 mb-2">
					No widgets yet
				</h3>
				<p class="text-sm text-gray-500 mb-6">
					Click "Add Widget" to start building your dashboard
				</p>
				<button
					@click="showWidgetPicker = true"
					class="px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
				>
					Add Your First Widget
				</button>
			</div>
		</div>

		<!-- Widget Configuration Modal -->
		<WidgetConfigModal
			v-if="configModalWidget"
			:show="showConfigModal"
			:widget-type-name="configModalWidget.typeName"
			:config-schema="
				widgetStore.getWidgetType(configModalWidget.instance.type)
					?.configSchema || []
			"
			:initial-config="configModalWidget.instance.config"
			@close="
				showConfigModal = false;
				configModalWidget = null;
			"
			@save="saveWidgetConfig"
		/>
	</div>
</template>

<style>
/* GridStack styles */
.grid-stack {
	background: transparent;
}

.grid-stack-item {
	overflow: visible !important;
}

.grid-stack-item-content {
	overflow: visible !important;
	inset: 0 !important;
}

/* Ensure widgets display properly */
.grid-stack-item .widget-base {
	height: 100%;
}
</style>
