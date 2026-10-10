<!-- Status Widget: URL status checks with response times -->
<script setup lang="ts">
import { computed } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import AppIcon from '../ui/AppIcon.vue';
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

const PILL: Record<Health, string> = {
	success: 'bg-success/12 text-success',
	warning: 'bg-warning/12 text-warning',
	error: 'bg-error/12 text-error',
};

const HEALTH_ORDER: Health[] = ['success', 'warning', 'error'];

const BAR: Record<Health, string> = {
	success: 'bg-success',
	warning: 'bg-warning',
	error: 'bg-error',
};

const ICON = {
	success: 'check',
	warning: 'arrow-right',
	error: 'x',
} as const;

const getStatusText = (status: number): string => {
	if (status === 0) return 'Unavailable';
	if (status >= 200 && status < 300) return 'Healthy';
	if (status >= 300 && status < 400) return 'Redirect';
	if (status >= 400 && status < 500) return 'Client Error';
	if (status >= 500) return 'Server Error';
	return 'Unknown';
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
		return urlObj.hostname + urlObj.pathname.replace(/\/$/, '');
	} catch {
		return url;
	}
};
</script>

<template>
	<BaseWidget
		type="status"
		title="Status Monitor"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="statuses.length === 0"
			class="flex h-full flex-col items-center justify-center gap-2 py-6 text-center"
		>
			<AppIcon name="activity" class="size-7 text-base-content/40" />
			<p class="text-sm text-base-content/70">No status results</p>
		</div>

		<div v-else class="flex flex-col gap-3">
			<!-- Summary first so it stays visible at the default size -->
			<div class="flex flex-col gap-2">
				<p
					class="flex flex-wrap gap-1.5 text-xs font-medium tabular-nums"
					data-testid="status-summary"
				>
					<span class="rounded-full px-2 py-0.5" :class="PILL.success"
						>{{ summary.success }} healthy</span
					>
					<span class="rounded-full px-2 py-0.5" :class="PILL.warning"
						>{{ summary.warning }}
						{{ summary.warning === 1 ? 'redirect' : 'redirects' }}</span
					>
					<span class="rounded-full px-2 py-0.5" :class="PILL.error"
						>{{ summary.error }}
						{{ summary.error === 1 ? 'error' : 'errors' }}</span
					>
				</p>
				<div
					class="flex h-1.5 gap-0.5 overflow-hidden rounded-full bg-base-200"
					aria-hidden="true"
				>
					<template v-for="kind in HEALTH_ORDER" :key="kind">
						<span
							v-if="summary[kind]"
							class="h-full"
							:class="BAR[kind]"
							:style="{ flexGrow: summary[kind] }"
						></span>
					</template>
				</div>
			</div>

			<ul class="flex flex-col divide-y divide-base-300/70">
				<li
					v-for="(status, index) in statuses"
					:key="index"
					class="flex flex-col gap-1 py-2 first:pt-0 last:pb-0"
				>
					<div class="flex items-center gap-2.5">
						<span
							class="grid size-5 shrink-0 place-items-center rounded-full"
							:class="PILL[health(status.status)]"
							aria-hidden="true"
						>
							<AppIcon :name="ICON[health(status.status)]" class="size-3" />
						</span>
						<span
							class="min-w-0 flex-1 truncate text-sm font-medium"
							:title="status.url"
							>{{ formatUrl(status.url) }}</span
						>
						<span
							class="shrink-0 rounded-md px-1.5 py-0.5 font-mono text-xs font-semibold tabular-nums"
							:class="PILL[health(status.status)]"
						>
							{{ status.status === 0 ? 'ERR' : status.status }}
						</span>
					</div>
					<div
						class="flex items-center justify-between gap-2 pl-7.5 text-xs text-base-content/70"
					>
						<span>{{ getStatusText(status.status) }}</span>
						<span class="tabular-nums">{{
							status.status === 0 ? 'N/A' : `${status.response_time_ms} ms`
						}}</span>
					</div>
					<p v-if="status.error" class="pl-7.5 text-xs text-error">
						{{ describeError(status.error) }}
					</p>
				</li>
			</ul>
		</div>
	</BaseWidget>
</template>
