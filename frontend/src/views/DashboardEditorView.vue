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
import AppIcon from '../components/ui/AppIcon.vue';
import EmptyIllustration from '../components/ui/EmptyIllustration.vue';
import WidgetTypeIcon from '../components/ui/WidgetTypeIcon.vue';

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

const STATUS_DOT: Record<string, string> = {
	saving: 'bg-primary animate-pulse',
	pending: 'bg-warning',
	saved: 'bg-success',
};
</script>

<template>
	<div class="flex flex-1 flex-col">
		<!-- Loading -->
		<div
			v-if="dashboardStore.currentLoading"
			class="flex flex-1 flex-col"
			role="status"
		>
			<span class="sr-only">Loading dashboard…</span>
			<div
				class="flex items-center gap-3 border-b border-base-content/10 px-3 py-3 sm:px-6"
				aria-hidden="true"
			>
				<div class="skeleton size-8 rounded-lg"></div>
				<div class="skeleton h-6 w-48"></div>
				<div class="skeleton ml-auto h-8 w-28 rounded-field"></div>
			</div>
			<div
				class="grid grid-cols-1 gap-4 p-4 sm:p-6 md:grid-cols-12"
				aria-hidden="true"
			>
				<div class="skeleton h-56 md:col-span-4"></div>
				<div class="skeleton h-56 md:col-span-3"></div>
				<div class="skeleton h-56 md:col-span-5"></div>
			</div>
		</div>

		<!-- Unknown or foreign id -->
		<div
			v-else-if="dashboardStore.currentNotFound"
			class="flex flex-1 items-center justify-center px-4 py-16"
		>
			<div
				class="surface flex w-full max-w-md animate-rise flex-col items-center gap-4 px-6 py-10 text-center"
			>
				<span
					class="grid size-14 place-items-center rounded-2xl bg-base-200 text-base-content/70 ring-1 ring-inset ring-base-300"
					aria-hidden="true"
				>
					<AppIcon name="search" class="size-6" />
				</span>
				<h1 class="text-2xl font-semibold tracking-tight">
					Dashboard not found
				</h1>
				<p class="text-base-content/70">
					This dashboard does not exist or belongs to another account.
				</p>
				<router-link to="/dashboards" class="btn btn-primary mt-2">
					<AppIcon name="arrow-left" class="size-4" />
					Back to dashboards
				</router-link>
			</div>
		</div>

		<!-- Load failure -->
		<div
			v-else-if="dashboardStore.currentError"
			class="flex flex-1 items-center justify-center px-4 py-16"
		>
			<div
				class="surface flex w-full max-w-md animate-rise flex-col items-center gap-4 px-6 py-10 text-center"
			>
				<span
					class="grid size-14 place-items-center rounded-2xl bg-error/10 text-error"
					aria-hidden="true"
				>
					<AppIcon name="alert-triangle" class="size-6" />
				</span>
				<div role="alert" class="flex flex-col gap-1.5">
					<h1 class="text-xl font-semibold tracking-tight">
						Could not load the dashboard
					</h1>
					<p class="text-sm text-base-content/70">
						{{ dashboardStore.currentError }}
					</p>
				</div>
				<div class="mt-2 flex flex-wrap justify-center gap-2">
					<button
						type="button"
						class="btn btn-primary btn-sm"
						@click="load(String(route.params.id))"
					>
						<AppIcon name="refresh" class="size-4" />
						Retry
					</button>
					<router-link to="/dashboards" class="btn btn-ghost btn-sm"
						>Back to dashboards</router-link
					>
				</div>
			</div>
		</div>

		<template v-else-if="ready">
			<!-- Toolbar -->
			<div
				class="sticky top-14 z-40 flex flex-wrap items-center gap-x-2 gap-y-2 border-b border-base-content/10 bg-base-100/60 px-3 py-2.5 backdrop-blur-xl sm:px-6"
			>
				<router-link
					to="/dashboards"
					class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-base-content"
					aria-label="Back to dashboards"
				>
					<AppIcon name="arrow-left" class="size-4" />
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
						class="input input-sm min-w-0 flex-1 basis-40 font-medium"
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
				<div v-else class="flex min-w-0 flex-1 basis-48 items-center gap-1">
					<h1 class="truncate text-lg font-semibold tracking-tight sm:text-xl">
						{{ name }}
					</h1>
					<button
						ref="renameButton"
						type="button"
						class="btn btn-ghost btn-xs btn-square shrink-0 text-base-content/70 hover:text-base-content"
						aria-label="Rename dashboard"
						@click="startRename"
					>
						<AppIcon name="pencil" class="size-3.5" />
					</button>
				</div>

				<div class="ml-auto flex items-center gap-2">
					<p
						class="inline-flex items-center gap-1.5 text-xs font-medium text-base-content/70"
						:class="{
							'rounded-full border border-base-300 bg-base-100/70 px-2.5 py-1':
								saveStatusText,
						}"
						role="status"
						data-testid="save-status"
					>
						<span
							v-if="saveStatusText"
							class="size-1.5 rounded-full"
							:class="STATUS_DOT[saver.status.value]"
							aria-hidden="true"
						></span>
						{{ saveStatusText }}
					</p>

					<div ref="pickerRoot" class="relative">
						<button
							ref="pickerTrigger"
							type="button"
							class="btn btn-primary btn-sm shadow-sm"
							aria-controls="widget-picker"
							:aria-expanded="picker.open.value"
							@click="picker.toggle"
						>
							<AppIcon name="plus" class="size-4" />
							Add Widget
						</button>
						<ul
							v-if="picker.open.value"
							id="widget-picker"
							data-menu-panel
							class="menu absolute right-0 top-full z-50 mt-2 w-80 max-w-[calc(100vw-1.5rem)] origin-top-right animate-pop rounded-box border border-base-300 bg-base-100 p-1.5 shadow-float"
						>
							<li v-for="type in widgetStore.availableWidgets" :key="type.id">
								<button
									type="button"
									class="flex items-center gap-3 px-2.5 py-2"
									@click="addWidget(type.id)"
								>
									<WidgetTypeIcon :type="type.id" />
									<span class="flex min-w-0 flex-col items-start">
										<span class="text-sm font-medium">{{ type.name }}</span>
										<span class="text-xs text-base-content/70">{{
											type.description
										}}</span>
									</span>
								</button>
							</li>
						</ul>
					</div>
				</div>
			</div>

			<!-- Save failure: edits stay in memory until a save succeeds -->
			<div
				v-if="saver.status.value === 'error'"
				class="alert alert-error alert-soft mx-3 mt-4 animate-rise sm:mx-6"
				role="alert"
			>
				<AppIcon name="alert-triangle" class="size-5" />
				<div>
					<h2 class="font-semibold">Your changes are not saved</h2>
					<p class="text-sm">{{ saver.error.value }}</p>
				</div>
				<button type="button" class="btn btn-sm" @click="saver.flush">
					<AppIcon name="refresh" class="size-4" />
					Retry
				</button>
			</div>

			<p
				v-if="compact"
				class="mx-3 mt-3 flex items-center gap-2 text-xs text-base-content/70 sm:mx-6"
			>
				<AppIcon name="move" class="size-3.5" />
				Moving and resizing widgets is available on wider screens.
			</p>

			<div class="flex-1 p-1.5 sm:p-4">
				<div
					v-if="widgets.length === 0"
					class="surface mx-auto mt-6 flex max-w-xl animate-rise flex-col items-center gap-4 border-dashed px-6 py-12 text-center sm:mt-10"
				>
					<EmptyIllustration class="max-w-60" />
					<h2 class="text-2xl font-semibold tracking-tight">
						Build your dashboard
					</h2>
					<p class="max-w-sm text-base-content/70">
						Add widgets to bring your data together in one place.
					</p>
					<button
						type="button"
						class="btn btn-primary mt-2"
						@click="openPicker"
					>
						<AppIcon name="plus" class="size-4" />
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
			:widget-type="configTarget.instance.type"
			:config-schema="
				widgetType(configTarget.instance.type)?.configSchema ?? []
			"
			:initial-config="configTarget.instance.config"
			@close="configTarget = null"
			@save="saveWidgetConfig"
		/>
	</div>
</template>
