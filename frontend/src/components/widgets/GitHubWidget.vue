/** * GitHub Widget * Displays recent GitHub activity for a user */
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseWidget from '../BaseWidget.vue';
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

const loading = ref(false);
const error = ref<string | null>(null);
const events = ref<GitHubEvent[]>([]);
const lastUpdated = ref<Date | null>(null);

let refreshTimer: number | null = null;

const loadData = async () => {
	if (!props.config.username) {
		error.value = 'GitHub username not configured';
		return;
	}

	loading.value = true;
	error.value = null;

	try {
		events.value = await fetchGitHubData(props.config.username);
		lastUpdated.value = new Date();
	} catch (err: any) {
		error.value = err.message || 'Failed to load GitHub data';
		events.value = [];
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

	switch (event.type) {
		case 'PushEvent':
			const commitCount = event.payload?.commits?.length || 0;
			return `Pushed ${commitCount} commit${
				commitCount !== 1 ? 's' : ''
			} to ${repo}`;
		case 'CreateEvent':
			return `Created ${
				event.payload?.ref_type || 'repository'
			} in ${repo}`;
		case 'IssuesEvent':
			return `${event.payload?.action || 'Updated'} issue in ${repo}`;
		case 'PullRequestEvent':
			return `${event.payload?.action || 'Updated'} PR in ${repo}`;
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
	() => props.config.username,
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
		:title="`GitHub: ${config.username || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		:refresh-interval="refreshInterval"
		@refresh="loadData"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="events.length === 0 && !loading && !error"
			class="text-center text-gray-500 py-8"
		>
			<p class="text-sm">No recent activity found</p>
		</div>

		<div v-else class="space-y-2">
			<div
				v-for="(event, index) in events.slice(0, 10)"
				:key="index"
				class="flex items-start space-x-3 p-2 rounded hover:bg-gray-50 transition-colors"
			>
				<span class="text-2xl flex-shrink-0">{{
					getEventIcon(event.type)
				}}</span>
				<div class="flex-1 min-w-0">
					<p class="text-sm text-gray-800 truncate">
						{{ formatEventDescription(event) }}
					</p>
					<p class="text-xs text-gray-500 mt-0.5">
						{{ formatTime(event.created_at) }}
					</p>
				</div>
			</div>
		</div>
	</BaseWidget>
</template>
