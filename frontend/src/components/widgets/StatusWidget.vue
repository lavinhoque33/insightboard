/** * Status Widget * Displays URL status checks with response times */
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { fetchStatusData, type StatusCheck } from '../../api/widgets';

interface Props {
	config: {
		urls: string; // Comma-separated URLs to check
	};
	refreshInterval?: number;
}

interface Emits {
	(e: 'configure'): void;
	(e: 'remove'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const loading = ref(false);
const error = ref<string | null>(null);
const statuses = ref<StatusCheck[]>([]);
const lastUpdated = ref<Date | null>(null);

let refreshTimer: number | null = null;

const loadData = async () => {
	if (!props.config.urls) {
		error.value = 'URLs not configured';
		return;
	}

	loading.value = true;
	error.value = null;

	try {
		statuses.value = await fetchStatusData(props.config.urls);
		lastUpdated.value = new Date();
	} catch (err: any) {
		error.value = err.message || 'Failed to load status data';
		statuses.value = [];
	} finally {
		loading.value = false;
	}
};

const setupAutoRefresh = () => {
	if (refreshTimer) {
		clearInterval(refreshTimer);
		refreshTimer = null;
	}

	if (props.refreshInterval && props.refreshInterval > 0) {
		refreshTimer = window.setInterval(() => {
			loadData();
		}, props.refreshInterval * 1000);
	}
};

const getStatusColor = (status: number): string => {
	if (status >= 200 && status < 300) return 'bg-green-500';
	if (status >= 300 && status < 400) return 'bg-yellow-500';
	if (status >= 400 && status < 500) return 'bg-orange-500';
	if (status >= 500) return 'bg-red-500';
	return 'bg-gray-500';
};

const getStatusText = (status: number): string => {
	if (status >= 200 && status < 300) return 'Healthy';
	if (status >= 300 && status < 400) return 'Redirect';
	if (status >= 400 && status < 500) return 'Client Error';
	if (status >= 500) return 'Server Error';
	return 'Unknown';
};

const getStatusIcon = (status: number): string => {
	if (status >= 200 && status < 300) return '✓';
	if (status >= 300 && status < 400) return '↻';
	if (status >= 400) return '✗';
	return '?';
};

const getResponseTimeColor = (responseTime: number): string => {
	if (responseTime < 200) return 'text-green-600';
	if (responseTime < 500) return 'text-yellow-600';
	if (responseTime < 1000) return 'text-orange-600';
	return 'text-red-600';
};

const formatUrl = (url: string): string => {
	try {
		const urlObj = new URL(url);
		return urlObj.hostname + urlObj.pathname;
	} catch {
		return url;
	}
};

onMounted(() => {
	loadData();
	setupAutoRefresh();
});

watch(
	() => props.refreshInterval,
	() => {
		setupAutoRefresh();
	},
);

watch(
	() => props.config.urls,
	() => {
		loadData();
	},
);

// Cleanup on unmount
onMounted(() => {
	return () => {
		if (refreshTimer) {
			clearInterval(refreshTimer);
		}
	};
});
</script>

<template>
	<BaseWidget
		title="Status Monitor"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		:refresh-interval="refreshInterval"
		@refresh="loadData"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="statuses.length === 0 && !loading && !error"
			class="text-center text-gray-500 py-8"
		>
			<p class="text-sm">No URLs configured</p>
		</div>

		<div v-else class="space-y-3">
			<div
				v-for="(status, index) in statuses"
				:key="index"
				class="p-3 rounded-lg border border-gray-200 hover:shadow-md transition-shadow"
			>
				<!-- Status Header -->
				<div class="flex items-center justify-between mb-2">
					<div class="flex items-center space-x-2 flex-1 min-w-0">
						<!-- Status Indicator -->
						<div
							:class="getStatusColor(status.status)"
							class="w-3 h-3 rounded-full flex-shrink-0 flex items-center justify-center"
						>
							<span class="text-white text-xs font-bold">{{
								getStatusIcon(status.status)
							}}</span>
						</div>

						<!-- URL -->
						<span
							class="text-sm font-medium text-gray-800 truncate"
							:title="status.url"
						>
							{{ formatUrl(status.url) }}
						</span>
					</div>

					<!-- Status Code Badge -->
					<span
						:class="getStatusColor(status.status)"
						class="px-2 py-1 rounded text-xs font-semibold text-white flex-shrink-0 ml-2"
					>
						{{ status.status }}
					</span>
				</div>

				<!-- Status Details -->
				<div class="flex items-center justify-between text-xs">
					<div class="text-gray-600">
						<span
							:class="
								getStatusColor(status.status).replace(
									'bg-',
									'text-',
								)
							"
							class="font-semibold"
						>
							{{ getStatusText(status.status) }}
						</span>
					</div>
					<div
						:class="getResponseTimeColor(status.response_time_ms)"
						class="font-semibold"
					>
						{{ status.response_time_ms }}ms
					</div>
				</div>

				<!-- Error Message -->
				<div
					v-if="status.error"
					class="mt-2 pt-2 border-t border-gray-100"
				>
					<p class="text-xs text-red-600">{{ status.error }}</p>
				</div>
			</div>

			<!-- Summary -->
			<div class="mt-4 pt-3 border-t border-gray-200">
				<div class="grid grid-cols-3 gap-2 text-center text-xs">
					<div>
						<div class="text-green-600 font-bold text-lg">
							{{
								statuses.filter(
									(s) => s.status >= 200 && s.status < 300,
								).length
							}}
						</div>
						<div class="text-gray-500">Healthy</div>
					</div>
					<div>
						<div class="text-yellow-600 font-bold text-lg">
							{{
								statuses.filter(
									(s) => s.status >= 300 && s.status < 400,
								).length
							}}
						</div>
						<div class="text-gray-500">Redirects</div>
					</div>
					<div>
						<div class="text-red-600 font-bold text-lg">
							{{ statuses.filter((s) => s.status >= 400).length }}
						</div>
						<div class="text-gray-500">Errors</div>
					</div>
				</div>
			</div>
		</div>
	</BaseWidget>
</template>
