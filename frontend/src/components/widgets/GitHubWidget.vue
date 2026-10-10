/** * GitHub Widget * Displays recent GitHub activity for a user */
<script setup lang="ts">
import { computed } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { useWidgetData } from '../../composables/useWidgetData';
import { fetchGitHubData, type GitHubEvent } from '../../api/widgets';

interface Props {
	config: {
		username: string;
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
	source: () => props.config.username,
	fetch: fetchGitHubData,
	refreshInterval: () => props.refreshInterval,
	notConfigured: 'GitHub username not configured',
	failure: 'Failed to load GitHub data',
});
const events = computed(() => data.value ?? []);

const getEventIcon = (eventType: string): string => {
	const icons: Record<string, string> = {
		PushEvent: '📤',
		CreateEvent: '✨',
		DeleteEvent: '🗑️',
		IssuesEvent: '🐛',
		PullRequestEvent: '🔀',
		WatchEvent: '⭐',
		ForkEvent: '🍴',
		IssueCommentEvent: '💬',
		PullRequestReviewEvent: '👀',
		ReleaseEvent: '🚀',
	};
	return icons[eventType] || '📋';
};

const formatEventDescription = (event: GitHubEvent): string => {
	const type = event.type.replace('Event', '');
	const repo = event.repo.name;
	const payload = (event.payload || {}) as {
		commits?: unknown[];
		ref_type?: string;
		action?: string;
	};

	switch (event.type) {
		case 'PushEvent':
			const commitCount = Array.isArray(payload.commits)
				? payload.commits.length
				: 0;
			return `Pushed ${commitCount} commit${
				commitCount !== 1 ? 's' : ''
			} to ${repo}`;
		case 'CreateEvent':
			return `Created ${payload.ref_type || 'repository'} in ${repo}`;
		case 'IssuesEvent':
			return `${payload.action || 'Updated'} issue in ${repo}`;
		case 'PullRequestEvent':
			return `${payload.action || 'Updated'} PR in ${repo}`;
		case 'WatchEvent':
			return `Starred ${repo}`;
		case 'ForkEvent':
			return `Forked ${repo}`;
		default:
			return `${type} in ${repo}`;
	}
};

const formatTime = (dateString: string): string => {
	const date = new Date(dateString);
	const now = new Date();
	const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

	if (diff < 60) return 'just now';
	if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
	return `${Math.floor(diff / 86400)}d ago`;
};
</script>

<template>
	<BaseWidget
		:title="`GitHub: ${config.username || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<!-- Empty State -->
		<div
			v-if="events.length === 0 && !loading && !error"
			class="flex flex-col items-center justify-center py-8 text-center"
		>
			<div class="text-4xl mb-3">📋</div>
			<p class="text-sm text-base-content/60">No recent activity found</p>
		</div>

		<!-- Activity Timeline -->
		<div v-else class="space-y-2">
			<div
				v-for="(event, index) in events.slice(0, 10)"
				:key="index"
				class="flex items-start gap-3 p-3 rounded-lg border border-base-300 hover:border-primary hover:shadow-md transition-all duration-300 group"
			>
				<!-- Event Icon Badge -->
				<div class="shrink-0">
					<div class="badge badge-lg badge-primary gap-2 text-lg">
						{{ getEventIcon(event.type) }}
					</div>
				</div>

				<!-- Event Details -->
				<div class="flex-1 min-w-0">
					<p
						class="text-sm font-medium text-base-content truncate group-hover:text-primary transition-colors"
					>
						{{ formatEventDescription(event) }}
					</p>
					<p class="text-xs text-base-content/50 mt-1">
						{{ formatTime(event.created_at) }}
					</p>
				</div>

				<!-- Event Type Badge -->
				<div class="shrink-0">
					<span class="badge badge-ghost text-xs">
						{{ event.type.replace('Event', '') }}
					</span>
				</div>
			</div>
		</div>
	</BaseWidget>
</template>
