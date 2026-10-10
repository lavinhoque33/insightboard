<!--
	Base Widget Component
	Shared widget chrome: header with type icon, title, freshness and actions;
	loading, error, and content states for the body.
-->
<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useDisclosure } from '../composables/useDisclosure';
import type { WidgetType as WidgetKind } from '../types';
import AppIcon from './ui/AppIcon.vue';
import WidgetTypeIcon from './ui/WidgetTypeIcon.vue';

interface Props {
	title: string;
	type?: WidgetKind | null;
	loading?: boolean;
	error?: string | null;
	lastUpdated?: Date | null;
}

interface Emits {
	(e: 'refresh'): void;
	(e: 'configure'): void;
	(e: 'remove'): void;
}

const props = withDefaults(defineProps<Props>(), {
	type: null,
	loading: false,
	error: null,
	lastUpdated: null,
});

const emit = defineEmits<Emits>();

const menu = useDisclosure();
const { root: menuRoot, trigger: menuTrigger } = menu;
const menuId = `widget-menu-${crypto.randomUUID()}`;

// Re-render the relative "updated" badge as time passes
const now = ref(Date.now());
let clock: number | undefined;
onMounted(() => {
	clock = window.setInterval(() => (now.value = Date.now()), 30_000);
});
onBeforeUnmount(() => window.clearInterval(clock));

const formattedUpdateTime = computed(() => {
	if (!props.lastUpdated) return '';
	const diff = Math.max(
		0,
		Math.floor((now.value - props.lastUpdated.getTime()) / 1000),
	);
	if (diff < 60) return 'Just now';
	if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
	return `${Math.floor(diff / 86400)}d ago`;
});

const handleRefresh = () => {
	menu.close();
	emit('refresh');
};

// The menu closes and focus returns to its button, so the dialog that opens
// next can hand focus back there when it closes.
const handleConfigure = () => {
	menu.close({ returnFocus: true });
	emit('configure');
};

const handleRemove = () => {
	menu.close();
	emit('remove');
};
</script>

<template>
	<div class="widget-base surface flex h-full flex-col overflow-hidden">
		<div
			class="widget-drag-handle flex shrink-0 items-center justify-between gap-2 border-b border-base-300/70 py-2 pl-3 pr-2"
		>
			<div class="flex min-w-0 flex-1 items-center gap-2.5">
				<WidgetTypeIcon v-if="type" :type="type" size="sm" />
				<h3 class="truncate text-sm font-semibold tracking-tight">
					{{ title }}
				</h3>
				<span
					v-if="lastUpdated"
					class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-base-200 px-2 py-0.5 text-[11px] font-medium tabular-nums text-base-content/70 max-[359px]:hidden"
					:title="lastUpdated.toLocaleString()"
				>
					<span
						class="size-1.5 rounded-full bg-success"
						aria-hidden="true"
					></span>
					{{ formattedUpdateTime }}
				</span>
			</div>
			<div class="flex shrink-0 items-center gap-0.5">
				<button
					type="button"
					class="btn btn-ghost btn-xs btn-square widget-action text-base-content/70 hover:text-base-content"
					:aria-label="`Refresh ${title}`"
					:aria-disabled="loading"
					@click="loading || handleRefresh()"
				>
					<AppIcon
						name="refresh"
						class="size-3.5"
						:class="{ 'animate-spin': loading }"
					/>
				</button>

				<div ref="menuRoot" class="widget-action relative">
					<button
						ref="menuTrigger"
						type="button"
						data-testid="widget-options-btn"
						class="btn btn-ghost btn-xs btn-square widget-action text-base-content/70 hover:text-base-content"
						:aria-label="`Options for ${title}`"
						:aria-expanded="menu.open.value"
						:aria-controls="menuId"
						@click="menu.toggle"
					>
						<AppIcon name="more" class="size-4" />
					</button>

					<ul
						v-if="menu.open.value"
						:id="menuId"
						data-menu-panel
						class="menu absolute right-0 top-full z-50 mt-1.5 w-44 origin-top-right animate-pop rounded-box border border-base-300 bg-base-100 p-1 shadow-float"
					>
						<li>
							<button
								data-testid="widget-configure-btn"
								type="button"
								@click="handleConfigure"
							>
								<AppIcon name="settings" class="size-4" />
								Configure
							</button>
						</li>
						<li>
							<button
								data-testid="widget-remove-btn"
								type="button"
								class="text-error"
								@click="handleRemove"
							>
								<AppIcon name="trash" class="size-4" />
								Remove
							</button>
						</li>
					</ul>
				</div>
			</div>
		</div>

		<div
			class="widget-content scroll-fade @container flex-1 overflow-auto p-3.5"
		>
			<div
				v-if="loading && !error"
				class="flex h-full flex-col gap-3"
				role="status"
			>
				<span class="sr-only">Loading data…</span>
				<div class="flex items-center gap-3" aria-hidden="true">
					<div class="skeleton size-10 shrink-0 rounded-xl"></div>
					<div class="flex flex-1 flex-col gap-2">
						<div class="skeleton h-3.5 w-3/5"></div>
						<div class="skeleton h-3 w-2/5"></div>
					</div>
				</div>
				<div class="skeleton h-3 w-full" aria-hidden="true"></div>
				<div class="skeleton h-3 w-4/5" aria-hidden="true"></div>
				<div class="skeleton h-3 w-2/3" aria-hidden="true"></div>
			</div>

			<div
				v-else-if="error"
				class="flex h-full flex-col items-center justify-center gap-3 py-2 text-center"
			>
				<div class="flex flex-col items-center gap-2" role="alert">
					<span
						class="grid size-10 place-items-center rounded-full bg-error/10 text-error"
						aria-hidden="true"
					>
						<AppIcon name="alert-triangle" class="size-5" />
					</span>
					<h4 class="text-sm font-semibold">Error loading data</h4>
					<p class="max-w-xs text-sm text-base-content/70">{{ error }}</p>
				</div>
				<button
					type="button"
					class="btn btn-sm btn-soft btn-primary"
					@click="handleRefresh"
				>
					<AppIcon name="refresh" class="size-3.5" />
					Try again
				</button>
			</div>

			<div v-else class="h-full animate-fade">
				<slot />
			</div>
		</div>
	</div>
</template>
