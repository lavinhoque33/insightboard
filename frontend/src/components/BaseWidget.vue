/** * Base Widget Component * Provides common widget functionality: header,
loading states, error handling, settings */
<script setup lang="ts">
import { ref, computed } from 'vue';

interface Props {
	title: string;
	loading?: boolean;
	error?: string | null;
	refreshInterval?: number;
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
	refreshInterval: 0,
	lastUpdated: null,
});

const emit = defineEmits<Emits>();

const showMenu = ref(false);

const formattedUpdateTime = computed(() => {
	if (!props.lastUpdated) return '';
	const now = new Date();
	const diff = Math.floor(
		(now.getTime() - props.lastUpdated.getTime()) / 1000,
	);

	if (diff < 60) return 'Just now';
	if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
	return `${Math.floor(diff / 86400)}d ago`;
});

const handleRefresh = () => {
	emit('refresh');
};

const handleConfigure = () => {
	showMenu.value = false;
	emit('configure');
};

const handleRemove = () => {
	showMenu.value = false;
	emit('remove');
};
</script>

<template>
	<div
		class="widget-base card bg-base-100 shadow-lg border border-base-300 flex flex-col h-full overflow-hidden hover:shadow-xl transition-all duration-300"
	>
		<!-- Widget Header with daisyUI -->
		<div
			class="widget-header bg-gradient-to-r from-primary/90 to-primary text-primary-content px-4 py-3 flex items-center justify-between flex-shrink-0"
		>
			<div class="flex items-center gap-2 flex-1 min-w-0">
				<h3 class="font-bold text-base truncate">{{ title }}</h3>
				<badge
					v-if="lastUpdated"
					class="badge badge-sm badge-ghost text-xs opacity-75"
				>
					{{ formattedUpdateTime }}
				</badge>
			</div>
			<div class="flex items-center gap-1 flex-shrink-0 relative">
				<!-- Refresh Button -->
				<button
					@click="handleRefresh"
					:disabled="loading"
					class="btn btn-ghost btn-xs gap-1"
					:class="{ loading: loading }"
					title="Refresh data"
				>
					<svg
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

				<!-- Options Dropdown -->
				<div class="dropdown dropdown-end">
					<button
						class="btn btn-ghost btn-xs gap-1"
						title="Widget options"
					>
						<svg
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

					<!-- Dropdown Menu -->
					<ul
						class="dropdown-content z-50 menu p-2 shadow-xl bg-base-100 rounded-lg border border-base-300 w-48"
					>
						<li>
							<a @click="handleConfigure" class="gap-2">
								<svg
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
										d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
									/>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
									/>
								</svg>
								Configure
							</a>
						</li>
						<li>
							<a @click="handleRemove" class="text-error gap-2">
								<svg
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
										d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
									/>
								</svg>
								Remove
							</a>
						</li>
					</ul>
				</div>
			</div>
		</div>

		<!-- Widget Content -->
		<div class="widget-content flex-1 p-4 overflow-auto">
			<!-- Loading State -->
			<div
				v-if="loading && !error"
				class="flex flex-col items-center justify-center h-full gap-3"
			>
				<span
					class="loading loading-spinner loading-lg text-primary"
				></span>
				<p class="text-sm text-base-content/60 font-medium">
					Loading data...
				</p>
			</div>

			<!-- Error State -->
			<div
				v-else-if="error"
				class="flex flex-col items-center justify-center h-full"
			>
				<div class="alert alert-error shadow-md w-full" role="alert">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="stroke-current shrink-0 h-6 w-6"
						fill="none"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
					<div>
						<h3 class="font-bold">Error Loading Data</h3>
						<div class="text-sm mt-1">{{ error }}</div>
					</div>
				</div>
				<button
					@click="handleRefresh"
					class="btn btn-primary btn-sm mt-4 gap-2"
				>
					<svg
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
							d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
						/>
					</svg>
					Try Again
				</button>
			</div>

			<!-- Actual Widget Content (slot) -->
			<div v-else class="h-full">
				<slot />
			</div>
		</div>
	</div>
</template>

<style scoped>
/* Ensure dropdown menu appears above GridStack items */
.widget-base {
	position: relative;
}

/* Click outside to close menu */
@media (max-width: 640px) {
	.widget-header {
		padding: 0.5rem;
	}

	.widget-header h3 {
		font-size: 0.75rem;
	}
}
</style>
