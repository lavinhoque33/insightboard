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
const addWidget = (typeId: string, event?: Event) => {
	if (event) {
		event.preventDefault();
		event.stopPropagation();
	}

	const newWidget = widgetStore.createWidgetInstance(typeId);
	if (!newWidget) return;

	widgets.value.push(newWidget);
	showWidgetPicker.value = false;

	// Close dropdown
	const dropdown = document.querySelector('.dropdown');
	if (dropdown instanceof HTMLElement) {
		dropdown.classList.remove('dropdown-open');
	}

	// Initialize grid if not already done (e.g., first widget being added)
	if (!gridInstance.value && gridContainer.value) {
		initializeGrid();
	}

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
	<div class="h-screen flex flex-col bg-base-100">
		<!-- Toolbar -->
		<div
			class="navbar bg-base-200 shadow-lg border-b border-base-300 sticky top-[70px] z-40"
		>
			<div class="flex-1">
				<button
					@click="router.push('/dashboards')"
					class="btn btn-ghost gap-2"
					title="Back to Dashboards"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M10 19l-7-7m0 0l7-7m-7 7h18"
						/>
					</svg>
					<h1
						class="text-2xl font-bold text-transparent bg-gradient-to-r from-primary to-secondary bg-clip-text"
					>
						{{ dashboard?.settings?.title || 'Dashboard Editor' }}
					</h1>
				</button>
			</div>

			<!-- Saving Status -->
			<div
				v-if="saving"
				class="flex items-center gap-2 text-primary font-medium mx-4"
			>
				<span class="loading loading-spinner loading-sm"></span>
				<span>Saving...</span>
			</div>

			<!-- Add Widget Dropdown -->
			<div class="dropdown dropdown-end">
				<button class="btn btn-primary gap-2 shadow-lg">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
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
				<ul
					class="dropdown-content z-50 menu p-2 shadow-xl bg-base-100 rounded-lg w-80 border border-base-300 max-h-96 overflow-y-auto"
				>
					<li
						v-for="widgetType in widgetStore.availableWidgets"
						:key="widgetType.id"
					>
						<button
							@click="addWidget(widgetType.id, $event)"
							type="button"
							class="p-3 text-left w-full hover:bg-base-200 rounded transition-colors"
						>
							<div class="flex items-start gap-3 w-full">
								<span class="text-2xl">{{
									widgetType.icon
								}}</span>
								<div class="flex-1">
									<div class="font-bold text-base-content">
										{{ widgetType.name }}
									</div>
									<div
										class="text-sm text-base-content/60 mt-1"
									>
										{{ widgetType.description }}
									</div>
								</div>
							</div>
						</button>
					</li>
				</ul>
			</div>
		</div>

		<!-- GridStack Container -->
		<div
			class="flex-1 overflow-auto p-6 bg-gradient-to-br from-base-100 via-base-100 to-base-200"
		>
			<!-- Empty State -->
			<div
				v-if="widgets.length === 0"
				class="flex flex-col items-center justify-center h-full"
			>
				<div class="text-center max-w-md">
					<div
						class="inline-flex items-center justify-center w-24 h-24 rounded-full bg-base-200 mb-6"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-12 w-12 text-base-content/40"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
							/>
						</svg>
					</div>
					<h2 class="text-2xl font-bold text-base-content mb-2">
						Build Your Dashboard
					</h2>
					<p class="text-base-content/60 mb-8">
						Add widgets to visualize your data and create a
						personalized dashboard experience.
					</p>
					<button
						@click="showWidgetPicker = true"
						class="btn btn-primary btn-lg gap-2 shadow-lg"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-6 w-6"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 4v16m8-8H4"
							/>
						</svg>
						Add Your First Widget
					</button>
				</div>
			</div>

			<!-- Widgets Grid (always rendered so ref is available) -->
			<div
				ref="gridContainer"
				class="grid-stack rounded-lg"
				:style="{ display: widgets.length === 0 ? 'none' : 'block' }"
			>
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
					<div
						class="grid-stack-item-content rounded-lg shadow-md hover:shadow-lg transition-all"
					>
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
