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
		class="widget-base bg-white rounded-lg shadow-md border border-gray-200 flex flex-col h-full overflow-hidden"
	>
		<!-- Widget Header -->
		<div
			class="widget-header bg-gradient-to-r from-blue-500 to-blue-600 text-white px-4 py-3 flex items-center justify-between"
		>
			<div class="flex items-center space-x-2">
				<h3 class="font-semibold text-sm">{{ title }}</h3>
				<span v-if="lastUpdated" class="text-xs opacity-75">{{
					formattedUpdateTime
				}}</span>
			</div>
			<div class="flex items-center space-x-2 relative">
				<button
					@click="handleRefresh"
					:disabled="loading"
					class="text-white hover:bg-white/20 rounded p-1 transition-colors disabled:opacity-50"
					title="Refresh"
				>
					<svg
						class="w-4 h-4"
						:class="{ 'animate-spin': loading }"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
						/>
					</svg>
				</button>
				<button
					@click="showMenu = !showMenu"
					class="text-white hover:bg-white/20 rounded p-1 transition-colors"
					title="Options"
				>
					<svg
						class="w-4 h-4"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
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
				<div
					v-if="showMenu"
					@click.stop
					class="absolute right-0 top-full mt-2 w-48 bg-white rounded-md shadow-lg border border-gray-200 py-1 z-50"
				>
					<button
						@click="handleConfigure"
						class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2"
					>
						<svg
							class="w-4 h-4"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
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
						<span>Configure</span>
					</button>
					<button
						@click="handleRemove"
						class="w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50 flex items-center space-x-2"
					>
						<svg
							class="w-4 h-4"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
							/>
						</svg>
						<span>Remove</span>
					</button>
				</div>
			</div>
		</div>

		<!-- Widget Content -->
		<div class="widget-content flex-1 p-4 overflow-auto">
			<!-- Loading State -->
			<div
				v-if="loading && !error"
				class="flex items-center justify-center h-full"
			>
				<div class="text-center">
					<svg
						class="animate-spin h-8 w-8 text-blue-500 mx-auto mb-2"
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
					<p class="text-sm text-gray-500">Loading...</p>
				</div>
			</div>

			<!-- Error State -->
			<div
				v-else-if="error"
				class="flex items-center justify-center h-full"
			>
				<div class="text-center max-w-sm">
					<svg
						class="h-12 w-12 text-red-500 mx-auto mb-3"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
					<p class="text-sm text-red-600 font-medium mb-2">
						Error Loading Data
					</p>
					<p class="text-xs text-gray-600">{{ error }}</p>
					<button
						@click="handleRefresh"
						class="mt-4 px-4 py-2 bg-blue-500 text-white text-sm rounded hover:bg-blue-600 transition-colors"
					>
						Try Again
					</button>
				</div>
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
