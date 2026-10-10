<!-- GitHub Widget: recent GitHub activity for a user -->
<script setup lang="ts">
import { computed } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import AppIcon from '../ui/AppIcon.vue';
import type { IconName } from '../ui/icons';
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
const events = computed(() => (data.value ?? []).slice(0, 10));

const EVENT_ICONS: Record<string, IconName> = {
	PushEvent: 'git-commit',
	CreateEvent: 'plus-circle',
	DeleteEvent: 'trash',
	IssuesEvent: 'circle-dot',
	PullRequestEvent: 'git-pull-request',
	WatchEvent: 'star',
	ForkEvent: 'git-fork',
	IssueCommentEvent: 'message',
	PullRequestReviewEvent: 'eye',
	ReleaseEvent: 'tag',
};

/** Action text before the repository name, e.g. "Pushed 3 commits to" */
const describeAction = (event: GitHubEvent): string => {
	const payload = (event.payload || {}) as {
		commits?: unknown[];
		ref_type?: string;
		action?: string;
	};

	switch (event.type) {
		case 'PushEvent': {
			const commitCount = Array.isArray(payload.commits)
				? payload.commits.length
				: 0;
			return `Pushed ${commitCount} commit${commitCount !== 1 ? 's' : ''} to`;
		}
		case 'CreateEvent':
			return `Created ${payload.ref_type || 'repository'} in`;
		case 'IssuesEvent':
			return `${payload.action || 'updated'} issue in`;
		case 'PullRequestEvent':
			return `${payload.action || 'updated'} PR in`;
		case 'WatchEvent':
			return 'Starred';
		case 'ForkEvent':
			return 'Forked';
		default:
			return `${event.type.replace('Event', '')} in`;
	}
};

const formatTime = (dateString: string): string => {
	const diff = Math.floor((Date.now() - new Date(dateString).getTime()) / 1000);

	if (diff < 60) return 'just now';
	if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
	return `${Math.floor(diff / 86400)}d ago`;
};
</script>

<template>
	<BaseWidget
		type="github"
		:title="`GitHub: ${config.username || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="events.length === 0 && !loading && !error"
			class="flex h-full flex-col items-center justify-center gap-2 py-6 text-center"
		>
			<AppIcon name="git-branch" class="size-7 text-base-content/40" />
			<p class="text-sm text-base-content/70">No recent activity found</p>
		</div>

		<ol v-else class="relative flex flex-col">
			<li
				v-for="(event, index) in events"
				:key="event.id || index"
				class="group relative flex gap-3 pb-3 last:pb-0"
			>
				<span
					v-if="index < events.length - 1"
					class="absolute left-[13px] top-7 bottom-0 w-px bg-base-300"
					aria-hidden="true"
				></span>
				<span
					class="relative z-[1] grid size-7 shrink-0 place-items-center rounded-full border border-base-300 bg-base-100 text-widget-github"
					aria-hidden="true"
				>
					<AppIcon
						:name="EVENT_ICONS[event.type] ?? 'activity'"
						class="size-3.5"
					/>
				</span>
				<div class="min-w-0 flex-1 pt-0.5">
					<p class="text-sm leading-snug">
						<span
							class="mr-1 inline-block text-base-content/70 first-letter:uppercase"
							>{{ `${describeAction(event)} ` }}</span
						>
						<span class="break-all font-medium">{{ event.repo.name }}</span>
					</p>
					<p class="mt-0.5 text-xs tabular-nums text-base-content/70">
						<time :datetime="event.created_at">{{
							formatTime(event.created_at)
						}}</time>
					</p>
				</div>
			</li>
		</ol>
	</BaseWidget>
</template>
