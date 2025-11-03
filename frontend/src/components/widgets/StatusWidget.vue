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
		<!-- Empty State -->
		<div
			v-if="statuses.length === 0 && !loading && !error"
			class="flex flex-col items-center justify-center py-8 text-center"
		>
			<div class="text-4xl mb-3">🔍</div>
			<p class="text-sm text-base-content/60">No URLs configured</p>
		</div>

		<!-- Status List -->
		<div v-else class="space-y-3">
			<div
				v-for="(status, index) in statuses"
				:key="index"
				class="card card-bordered border-base-300 hover:shadow-md transition-all duration-300"
			>
				<div class="card-body p-3">
					<!-- Status Header -->
					<div class="flex items-center justify-between gap-2">
						<!-- Status Indicator and URL -->
						<div class="flex items-center gap-2 flex-1 min-w-0">
							<!-- Status Badge -->
							<div
								class="badge badge-lg"
								:class="{
									'badge-success':
										status.status >= 200 &&
										status.status < 300,
									'badge-warning':
										status.status >= 300 &&
										status.status < 400,
									'badge-error': status.status >= 400,
								}"
							>
								{{ getStatusIcon(status.status) }}
							</div>

							<!-- URL -->
							<span
								class="text-sm font-medium text-base-content truncate"
								:title="status.url"
							>
								{{ formatUrl(status.url) }}
							</span>
						</div>

						<!-- Status Code -->
						<span
							class="badge badge-lg font-semibold flex-shrink-0"
							:class="{
								'badge-success':
									status.status >= 200 && status.status < 300,
								'badge-warning':
									status.status >= 300 && status.status < 400,
								'badge-error': status.status >= 400,
							}"
						>
							{{ status.status }}
						</span>
					</div>

					<!-- Status Details -->
					<div class="divider my-1"></div>

					<div class="flex items-center justify-between text-sm">
						<!-- Status Text -->
						<div
							class="font-medium"
							:class="{
								'text-success':
									status.status >= 200 && status.status < 300,
								'text-warning':
									status.status >= 300 && status.status < 400,
								'text-error': status.status >= 400,
							}"
						>
							{{ getStatusText(status.status) }}
						</div>

						<!-- Response Time -->
						<div
							class="font-semibold"
							:class="{
								'text-success': status.response_time_ms < 200,
								'text-warning': status.response_time_ms < 500,
								'text-error': status.response_time_ms >= 1000,
							}"
						>
							{{ status.response_time_ms }}ms
						</div>
					</div>

					<!-- Error Message -->
					<div
						v-if="status.error"
						class="alert alert-error shadow-md mt-2 py-2"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="stroke-current flex-shrink-0 h-5 w-5"
							fill="none"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M10 14l-2-2m0 0l-2-2m2 2l2-2m-2 2l-2 2m8-8l-2 2m0 0l-2 2m2-2l2 2m-2-2l-2-2"
							/>
						</svg>
						<span class="text-xs">{{ status.error }}</span>
					</div>
				</div>
			</div>

			<!-- Summary Stats -->
			<div
				v-if="statuses.length > 0"
				class="card card-bordered border-base-300 bg-base-200"
			>
				<div class="card-body p-4">
					<div class="grid grid-cols-3 gap-2 text-center">
						<!-- Healthy Count -->
						<div class="stat bg-base-100 rounded-lg p-2">
							<div class="stat-value text-success text-2xl">
								{{
									statuses.filter(
										(s) =>
											s.status >= 200 && s.status < 300,
									).length
								}}
							</div>
							<div class="stat-title text-xs">Healthy</div>
						</div>

						<!-- Redirects Count -->
						<div class="stat bg-base-100 rounded-lg p-2">
							<div class="stat-value text-warning text-2xl">
								{{
									statuses.filter(
										(s) =>
											s.status >= 300 && s.status < 400,
									).length
								}}
							</div>
							<div class="stat-title text-xs">Redirects</div>
						</div>

						<!-- Errors Count -->
						<div class="stat bg-base-100 rounded-lg p-2">
							<div class="stat-value text-error text-2xl">
								{{
									statuses.filter((s) => s.status >= 400)
										.length
								}}
							</div>
							<div class="stat-title text-xs">Errors</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</BaseWidget>
</template>
