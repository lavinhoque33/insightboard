<!--
	Dashboard Editor View
	Loads one dashboard by id, lets the user add, configure, move, resize, and
	remove widgets, and auto-saves through a serialised queue. Vue creates and
	removes the widget DOM nodes; GridStack only positions them (makeWidget and
	removeWidget without DOM removal).
-->
<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import type { ComponentPublicInstance } from 'vue';
import { onBeforeRouteLeave, onBeforeRouteUpdate, useRoute } from 'vue-router';
import 'gridstack/dist/gridstack.min.css';
import { GridStack } from 'gridstack';
import type { GridStackNode } from 'gridstack';
import { useDashboardStore } from '../stores/dashboard';
import { useWidgetStore } from '../stores/widgets';
import type { WidgetInstance } from '../stores/widgets';
import { useAuthStore } from '../stores/auth';
import type {
	LayoutConfig,
	WidgetConfig,
	WidgetType as WidgetKind,
} from '../types';
import WidgetConfigModal from '../components/WidgetConfigModal.vue';
import { useSaveQueue } from '../composables/useSaveQueue';
import { useDisclosure } from '../composables/useDisclosure';

const AUTO_SAVE_DELAY_MS = 2000;
const FULL_COLUMNS = 12;

const route = useRoute();
const dashboardStore = useDashboardStore();
const widgetStore = useWidgetStore();
const authStore = useAuthStore();

// Working copy of the dashboard being edited; persisted by the save queue
const name = ref('');
const widgets = ref<WidgetInstance[]>([]);
const ready = ref(false);
// The id the working copy belongs to; saves always target it
let loadedId = '';

const saver = useSaveQueue({
	snapshot: (): { name: string; layout: LayoutConfig } => ({
		name: name.value,
		layout: {
			widgets: widgets.value.map((w) => ({
				id: w.id,
				type: w.type,
				x: w.layout.x,
				y: w.layout.y,
				w: w.layout.w,
				h: w.layout.h,
				config: w.config,
			})),
		},
	}),
	save: (changes) => dashboardStore.updateDashboard(loadedId, changes),
	delay: AUTO_SAVE_DELAY_MS,
});

// --- GridStack -------------------------------------------------------------

let grid: GridStack | null = null;
const gridContainer = ref<HTMLElement | null>(null);
const itemElements = new Map<string, HTMLElement>();
// Below the breakpoint the grid shows one column; positions are then not
// editable and not saved, so the 12-column layout is never overwritten.
const compact = ref(false);

const setItemElement = (
	id: string,
	el: Element | ComponentPublicInstance | null,
) => {
	if (el instanceof HTMLElement) itemElements.set(id, el);
	else itemElements.delete(id);
};

const syncColumnMode = () => {
	if (!grid) return;
	compact.value = grid.getColumn() < FULL_COLUMNS;
	grid.setStatic(compact.value);
};

const onGridChange = (_event: Event, nodes: GridStackNode[]) => {
	if (!grid || grid.getColumn() !== FULL_COLUMNS) return;
	for (const node of nodes) {
		const widget = widgets.value.find((w) => w.id === node.id);
		if (!widget) continue;
		widget.layout = {
			x: node.x ?? 0,
			y: node.y ?? 0,
			w: node.w ?? widget.layout.w,
			h: node.h ?? widget.layout.h,
		};
	}
	saver.schedule();
};

const initGrid = () => {
	if (grid || !gridContainer.value) return;
	// Picks up the items Vue already rendered (gs-* attributes)
	grid = GridStack.init(
		{
			column: FULL_COLUMNS,
			columnOpts: { breakpoints: [{ w: 768, c: 1 }] },
			float: false,
			cellHeight: 80,
			margin: 8,
			minRow: 1,
			animate: true,
			draggable: {
				handle: '.widget-drag-handle',
				cancel: '.widget-action,button,a,input,textarea,select',
			},
			resizable: { handles: 'se, sw' },
		},
		gridContainer.value,
	);
	grid.on('change', onGridChange);
	grid.on('resizecontent', syncColumnMode);
	syncColumnMode();
};

const destroyGrid = () => {
	// Keep the DOM: Vue owns the nodes and removes them itself
	grid?.destroy(false);
	grid = null;
};

// --- Loading -----------------------------------------------------------------

const load = async (id: string) => {
	destroyGrid();
	ready.value = false;
	loadedId = '';
	await dashboardStore.fetchDashboard(id);
	const dashboard = dashboardStore.current;
	if (!dashboard || dashboard.id !== id) return;

	loadedId = id;
	name.value = dashboard.name;
	widgets.value = dashboard.layout.widgets.map((w) => ({
		id: w.id,
		type: w.type,
		config: {
			...widgetStore.getWidgetType(w.type)?.defaultConfig,
			...w.config,
		},
		layout: { x: w.x, y: w.y, w: w.w, h: w.h },
	}));
	saver.markSaved();
	ready.value = true;
	await nextTick();
	initGrid();
};

onMounted(() => load(String(route.params.id)));

// --- Leaving -----------------------------------------------------------------

/**
 * Persist pending edits before the route changes. If the save fails the user
 * stays (with the error and Retry visible) unless they choose to discard.
 */
const settleBeforeLeaving = async (): Promise<boolean> => {
	if (!ready.value || !authStore.isAuthenticated || !saver.hasPendingWork()) {
		return true;
	}
	if (await saver.flush()) return true;
	return window.confirm(
		'Your latest changes could not be saved. Leave this page and discard them?',
	);
};

onBeforeRouteLeave(settleBeforeLeaving);
// Another dashboard in the same view instance (for example through history)
onBeforeRouteUpdate(async (to, from) => {
	if (to.params.id === from.params.id) return true;
	if (!(await settleBeforeLeaving())) return false;
	void load(String(to.params.id));
	return true;
});

const onBeforeUnload = (event: BeforeUnloadEvent) => {
	if (ready.value && saver.hasPendingWork()) event.preventDefault();
};
window.addEventListener('beforeunload', onBeforeUnload);

onBeforeUnmount(() => {
	// Nothing is sent once the editor is gone
	saver.dispose();
	window.removeEventListener('beforeunload', onBeforeUnload);
	destroyGrid();
});

// --- Widgets -----------------------------------------------------------------

const picker = useDisclosure();
const { root: pickerRoot, trigger: pickerTrigger } = picker;

const configTarget = ref<{ instance: WidgetInstance; isNew: boolean } | null>(
	null,
);

const openPicker = async () => {
	picker.open.value = true;
	await nextTick();
	pickerTrigger.value?.scrollIntoView({ block: 'nearest' });
};

// Opens the configuration for a draft; the widget is only added on Save
const addWidget = (typeId: WidgetKind) => {
	picker.close({ returnFocus: true });
	const bottom = Math.max(
		0,
		...widgets.value.map((w) => w.layout.y + w.layout.h),
	);
	const draft = widgetStore.createWidgetInstance(typeId, { x: 0, y: bottom });
	if (draft) configTarget.value = { instance: draft, isNew: true };
};

const configureWidget = (widget: WidgetInstance) => {
	configTarget.value = { instance: widget, isNew: false };
};

const saveWidgetConfig = async (config: WidgetConfig) => {
	const target = configTarget.value;
	if (!target) return;

	if (!target.isNew) {
		target.instance.config = config;
		saver.schedule();
		return;
	}

	const widget: WidgetInstance = { ...target.instance, config };
	widgets.value.push(widget);
	saver.schedule();
	await nextTick();
	const el = itemElements.get(widget.id);
	if (grid && el) grid.makeWidget(el);
};

const removeWidget = (widget: WidgetInstance) => {
	const el = itemElements.get(widget.id);
	// GridStack lets go of the node; Vue removes it from the DOM
	if (grid && el) grid.removeWidget(el, false);
	widgets.value = widgets.value.filter((w) => w.id !== widget.id);
	saver.schedule();
	pickerTrigger.value?.focus();
};

const widgetType = (typeId: WidgetKind) => widgetStore.getWidgetType(typeId);

// --- Rename ------------------------------------------------------------------

const renaming = ref(false);
const nameDraft = ref('');
const nameError = ref<string | null>(null);
const renameButton = ref<HTMLButtonElement | null>(null);
const nameInput = ref<HTMLInputElement | null>(null);

const startRename = async () => {
	nameDraft.value = name.value;
	nameError.value = null;
	renaming.value = true;
	await nextTick();
	nameInput.value?.select();
};

const endRename = async () => {
	renaming.value = false;
	await nextTick();
	renameButton.value?.focus();
};

const submitRename = () => {
	const trimmed = nameDraft.value.trim();
	if (!trimmed) {
		nameError.value = 'Enter a name for the dashboard.';
		nameInput.value?.focus();
		return;
	}
	name.value = trimmed;
	void saver.flush();
	void endRename();
};

const saveStatusText = computed(() => {
	switch (saver.status.value) {
		case 'saving':
			return 'Saving…';
		case 'pending':
			return 'Unsaved changes';
		case 'saved':
			return 'All changes saved';
		default:
			return '';
	}
});
</script>

<template>
	<div class="flex flex-1 flex-col">
		<!-- Loading -->
		<div
			v-if="dashboardStore.currentLoading"
			class="flex flex-1 flex-col items-center justify-center gap-3 py-24"
			role="status"
		>
			<span
				class="loading loading-spinner loading-lg text-primary"
				aria-hidden="true"
			></span>
			<p class="text-base-content/70">Loading dashboard…</p>
		</div>

		<!-- Unknown or foreign id -->
		<div
			v-else-if="dashboardStore.currentNotFound"
			class="mx-auto flex max-w-md flex-col items-center gap-4 px-4 py-24 text-center"
		>
			<h1 class="text-2xl font-bold">Dashboard not found</h1>
			<p class="text-base-content/70">
				This dashboard does not exist or belongs to another account.
			</p>
			<router-link to="/dashboards" class="btn btn-primary"
				>Back to dashboards</router-link
			>
		</div>

		<!-- Load failure -->
		<div
			v-else-if="dashboardStore.currentError"
			class="mx-auto flex w-full max-w-md flex-col gap-4 px-4 py-24"
		>
			<div class="alert alert-error" role="alert">
				<div>
					<h1 class="font-bold">Could not load the dashboard</h1>
					<p class="text-sm">{{ dashboardStore.currentError }}</p>
				</div>
				<button
					type="button"
					class="btn btn-sm"
					@click="load(String(route.params.id))"
				>
					Retry
				</button>
			</div>
			<router-link to="/dashboards" class="link"
				>Back to dashboards</router-link
			>
		</div>

		<template v-else-if="ready">
			<!-- Toolbar -->
			<div
				class="flex flex-wrap items-center gap-2 border-b border-base-300 bg-base-200 px-3 py-2 sm:px-6"
			>
				<router-link
					to="/dashboards"
					class="btn btn-ghost btn-sm btn-square"
					aria-label="Back to dashboards"
				>
					<svg
						aria-hidden="true"
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
				</router-link>

				<form
					v-if="renaming"
					class="flex min-w-0 flex-1 flex-wrap items-center gap-2"
					novalidate
					@submit.prevent="submitRename"
				>
					<label for="dashboard-name" class="sr-only">Dashboard name</label>
					<input
						id="dashboard-name"
						ref="nameInput"
						v-model="nameDraft"
						type="text"
						class="input input-sm min-w-0 flex-1"
						:class="{ 'input-error': nameError }"
						:aria-invalid="!!nameError"
						:aria-describedby="nameError ? 'dashboard-name-error' : undefined"
						@keydown.esc.prevent="endRename"
					/>
					<button type="submit" class="btn btn-primary btn-sm">Save</button>
					<button type="button" class="btn btn-ghost btn-sm" @click="endRename">
						Cancel
					</button>
					<p
						v-if="nameError"
						id="dashboard-name-error"
						class="w-full text-sm text-error"
						role="alert"
					>
						{{ nameError }}
					</p>
				</form>
				<div v-else class="flex min-w-0 flex-1 items-center gap-1">
					<h1 class="truncate text-lg font-bold sm:text-2xl">{{ name }}</h1>
					<button
						ref="renameButton"
						type="button"
						class="btn btn-ghost btn-xs btn-square shrink-0"
						aria-label="Rename dashboard"
						@click="startRename"
					>
						<svg
							aria-hidden="true"
							xmlns="http://www.w3.org/2000/svg"
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M15.232 5.232l3.536 3.536M9 13l6.232-6.232a2.5 2.5 0 113.536 3.536L12.536 16.536 9 17l.464-3.536z"
							/>
						</svg>
					</button>
				</div>

				<p
					class="text-sm text-base-content/70"
					role="status"
					data-testid="save-status"
				>
					{{ saveStatusText }}
				</p>

				<div ref="pickerRoot" class="relative">
					<button
						ref="pickerTrigger"
						type="button"
						class="btn btn-primary btn-sm"
						aria-controls="widget-picker"
						:aria-expanded="picker.open.value"
						@click="picker.toggle"
					>
						<svg
							aria-hidden="true"
							xmlns="http://www.w3.org/2000/svg"
							class="h-4 w-4"
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
						Add Widget
					</button>
					<ul
						v-if="picker.open.value"
						id="widget-picker"
						data-menu-panel
						class="menu absolute right-0 top-full z-50 mt-1 w-72 max-w-[calc(100vw-1.5rem)] rounded-box border border-base-300 bg-base-100 p-1 shadow-lg"
					>
						<li v-for="type in widgetStore.availableWidgets" :key="type.id">
							<button
								type="button"
								class="flex items-start gap-3"
								@click="addWidget(type.id)"
							>
								<span class="text-xl" aria-hidden="true">{{ type.icon }}</span>
								<span class="flex flex-col items-start">
									<span class="font-semibold">{{ type.name }}</span>
									<span class="text-xs text-base-content/70">{{
										type.description
									}}</span>
								</span>
							</button>
						</li>
					</ul>
				</div>
			</div>

			<!-- Save failure: edits stay in memory until a save succeeds -->
			<div
				v-if="saver.status.value === 'error'"
				class="alert alert-error mx-3 mt-3 sm:mx-6"
				role="alert"
			>
				<div>
					<h2 class="font-bold">Your changes are not saved</h2>
					<p class="text-sm">{{ saver.error.value }}</p>
				</div>
				<button type="button" class="btn btn-sm" @click="saver.flush">
					Retry
				</button>
			</div>

			<p v-if="compact" class="px-3 pt-3 text-sm text-base-content/70 sm:px-6">
				Moving and resizing widgets is available on wider screens.
			</p>

			<div class="flex-1 p-3 sm:p-6">
				<div
					v-if="widgets.length === 0"
					class="mx-auto flex max-w-md flex-col items-center gap-4 py-16 text-center"
				>
					<h2 class="text-2xl font-bold">Build your dashboard</h2>
					<p class="text-base-content/70">
						Add widgets to bring your data together in one place.
					</p>
					<button type="button" class="btn btn-primary" @click="openPicker">
						Add your first widget
					</button>
				</div>

				<div v-show="widgets.length > 0" ref="gridContainer" class="grid-stack">
					<div
						v-for="widget in widgets"
						:key="widget.id"
						:ref="(el) => setItemElement(widget.id, el)"
						class="grid-stack-item"
						:gs-id="widget.id"
						:gs-x="widget.layout.x"
						:gs-y="widget.layout.y"
						:gs-w="widget.layout.w"
						:gs-h="widget.layout.h"
						:gs-min-w="widgetType(widget.type)?.minSize.w"
						:gs-min-h="widgetType(widget.type)?.minSize.h"
					>
						<div class="grid-stack-item-content">
							<component
								:is="widgetType(widget.type)?.component"
								v-if="widgetType(widget.type)"
								:config="widget.config"
								:refresh-interval="widgetType(widget.type)?.refreshInterval"
								@configure="configureWidget(widget)"
								@remove="removeWidget(widget)"
							/>
						</div>
					</div>
				</div>
			</div>
		</template>

		<WidgetConfigModal
			v-if="configTarget"
			:key="configTarget.instance.id"
			:widget-type-name="widgetType(configTarget.instance.type)?.name ?? ''"
			:config-schema="
				widgetType(configTarget.instance.type)?.configSchema ?? []
			"
			:initial-config="configTarget.instance.config"
			@close="configTarget = null"
			@save="saveWidgetConfig"
		/>
	</div>
</template>
