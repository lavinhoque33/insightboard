/** * Base Widget Component * Provides common widget functionality: header,
loading states, error handling, settings */
<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useDisclosure } from '../composables/useDisclosure';

interface Props {
	title: string;
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
	<div
		class="widget-base card card-border bg-base-100 shadow-sm flex flex-col h-full overflow-hidden"
	>
		<div
			class="widget-drag-handle bg-base-200 border-b border-base-300 px-3 py-2 flex items-center justify-between gap-2 shrink-0"
		>
			<div class="flex items-center gap-2 flex-1 min-w-0">
				<h3 class="font-bold text-sm sm:text-base truncate">{{ title }}</h3>
				<span
					v-if="lastUpdated"
					class="badge badge-sm badge-outline shrink-0"
					:title="lastUpdated.toLocaleString()"
				>
					{{ formattedUpdateTime }}
				</span>
			</div>
			<div class="flex items-center gap-1 shrink-0">
				<button
					type="button"
					class="btn btn-ghost btn-xs btn-square widget-action"
					:aria-label="`Refresh ${title}`"
					:aria-disabled="loading"
					@click="loading || handleRefresh()"
				>
					<svg
						aria-hidden="true"
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4"
						:class="{ 'animate-spin': loading }"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
						/>
					</svg>
				</button>

				<div ref="menuRoot" class="relative widget-action">
					<button
						ref="menuTrigger"
						type="button"
						data-testid="widget-options-btn"
						class="btn btn-ghost btn-xs btn-square widget-action"
						:aria-label="`Options for ${title}`"
						:aria-expanded="menu.open.value"
						:aria-controls="menuId"
						@click="menu.toggle"
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
								d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
							/>
						</svg>
					</button>

					<ul
						v-if="menu.open.value"
						:id="menuId"
						data-menu-panel
						class="menu absolute right-0 top-full mt-1 z-50 w-40 rounded-box border border-base-300 bg-base-100 p-1 shadow-lg"
					>
						<li>
							<button
								data-testid="widget-configure-btn"
								type="button"
								@click="handleConfigure"
							>
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
								Remove
							</button>
						</li>
					</ul>
				</div>
			</div>
		</div>

		<div class="widget-content flex-1 p-3 overflow-auto">
			<div
				v-if="loading && !error"
				class="flex flex-col items-center justify-center h-full gap-3"
				role="status"
			>
				<span
					class="loading loading-spinner loading-md text-primary"
					aria-hidden="true"
				></span>
				<p class="text-sm text-base-content/70">Loading data…</p>
			</div>

			<div
				v-else-if="error"
				class="flex flex-col items-center justify-center h-full gap-3"
			>
				<div class="alert alert-error alert-soft w-full" role="alert">
					<div>
						<h4 class="font-bold">Error loading data</h4>
						<p class="text-sm">{{ error }}</p>
					</div>
				</div>
				<button
					type="button"
					class="btn btn-primary btn-sm"
					@click="handleRefresh"
				>
					Try again
				</button>
			</div>

			<div v-else class="h-full">
				<slot />
			</div>
		</div>
	</div>
</template>
