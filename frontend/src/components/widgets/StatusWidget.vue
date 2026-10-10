/** * Status Widget * Displays URL status checks with response times */
<script setup lang="ts">
import { computed } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { useWidgetData } from '../../composables/useWidgetData';
import { fetchStatusData } from '../../api/widgets';

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

const { data, loading, error, lastUpdated, reload } = useWidgetData({
	source: () => props.config.urls,
	fetch: fetchStatusData,
	refreshInterval: () => props.refreshInterval,
	notConfigured: 'URLs not configured',
	failure: 'Failed to load status data',
});
const statuses = computed(() => data.value ?? []);

type Health = 'success' | 'warning' | 'error';

// Status 0 means the check failed (blocked, timeout, unreachable): an error
const health = (status: number): Health => {
	if (status >= 200 && status < 300) return 'success';
	if (status >= 300 && status < 400) return 'warning';
	return 'error';
};

const summary = computed(() => {
	const counts: Record<Health, number> = { success: 0, warning: 0, error: 0 };
	for (const check of statuses.value) counts[health(check.status)]++;
	return counts;
});

const BADGE: Record<Health, string> = {
	success: 'badge-success',
	warning: 'badge-warning',
	error: 'badge-error',
};

const getStatusText = (status: number): string => {
	if (status === 0) return 'Unavailable';
	if (status >= 200 && status < 300) return 'Healthy';
	if (status >= 300 && status < 400) return 'Redirect';
	if (status >= 400 && status < 500) return 'Client Error';
	if (status >= 500) return 'Server Error';
	return 'Unknown';
};

const getStatusIcon = (status: number): string => {
	if (status === 0) return '✗';
	if (status >= 200 && status < 300) return '✓';
	if (status >= 300 && status < 400) return '↻';
	if (status >= 400) return '✗';
	return '?';
};

// Generic per-URL failure classes sent by the backend (never raw error text)
const ERROR_CLASS_TEXT: Record<string, string> = {
	invalid_url: 'Invalid URL (must be http or https, without credentials)',
	blocked_address: 'Blocked: private or internal addresses cannot be checked',
	dns_error: 'Host name could not be resolved',
	timeout: 'No response before the timeout',
	connect_error: 'Could not connect to the host',
	request_error: 'The request failed',
};

const describeError = (errorClass: string): string =>
	ERROR_CLASS_TEXT[errorClass] ?? 'The check failed';

const formatUrl = (url: string): string => {
	try {
		const urlObj = new URL(url);
		return urlObj.hostname + urlObj.pathname;
	} catch {
		return url;
	}
};
</script>

<template>
	<BaseWidget
		title="Status Monitor"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<!-- Empty State -->
		<div
			v-if="statuses.length === 0"
			class="flex flex-col items-center justify-center py-8 text-center"
		>
			<div class="text-4xl mb-3" aria-hidden="true">🔍</div>
			<p class="text-sm text-base-content/70">No status results</p>
		</div>

		<div v-else class="flex flex-col gap-2">
			<!-- Summary first so it stays visible at the default size -->
			<p class="flex flex-wrap gap-2 text-xs" data-testid="status-summary">
				<span class="badge badge-sm badge-success"
					>{{ summary.success }} healthy</span
				>
				<span class="badge badge-sm badge-warning"
					>{{ summary.warning }} redirects</span
				>
				<span class="badge badge-sm badge-error"
					>{{ summary.error }} errors</span
				>
			</p>

			<ul class="flex flex-col gap-2">
				<li
					v-for="(status, index) in statuses"
					:key="index"
					class="rounded-box border border-base-300 p-2"
				>
					<div class="flex items-center justify-between gap-2">
						<div class="flex items-center gap-2 min-w-0">
							<span
								class="badge badge-sm"
								:class="BADGE[health(status.status)]"
								aria-hidden="true"
								>{{ getStatusIcon(status.status) }}</span
							>
							<span class="text-sm font-medium truncate" :title="status.url">{{
								formatUrl(status.url)
							}}</span>
						</div>
						<span
							class="badge badge-sm font-semibold shrink-0"
							:class="BADGE[health(status.status)]"
						>
							{{ status.status === 0 ? 'ERR' : status.status }}
						</span>
					</div>
					<div class="mt-1 flex items-center justify-between text-xs">
						<span>{{ getStatusText(status.status) }}</span>
						<span>{{
							status.status === 0 ? 'N/A' : `${status.response_time_ms} ms`
						}}</span>
					</div>
					<p v-if="status.error" class="mt-1 text-xs text-error">
						{{ describeError(status.error) }}
					</p>
				</li>
			</ul>
		</div>
	</BaseWidget>
</template>
