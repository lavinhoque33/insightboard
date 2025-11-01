/** * News Widget * Displays news articles filtered by topic */
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { fetchNewsData, type NewsArticle } from '../../api/widgets';

interface Props {
	config: {
		topic: string;
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
const articles = ref<NewsArticle[]>([]);
const lastUpdated = ref<Date | null>(null);

let refreshTimer: number | null = null;

const loadData = async () => {
	if (!props.config.topic) {
		error.value = 'News topic not configured';
		return;
	}

	loading.value = true;
	error.value = null;

	try {
		articles.value = await fetchNewsData(props.config.topic);
		lastUpdated.value = new Date();
	} catch (err: any) {
		error.value = err.message || 'Failed to load news';
		articles.value = [];
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

const formatTime = (dateString: string): string => {
	const date = new Date(dateString);
	const now = new Date();
	const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

	if (diff < 3600) return `${Math.floor(diff / 60)} minutes ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)} hours ago`;
	return `${Math.floor(diff / 86400)} days ago`;
};

// Removed unused truncateText helper to satisfy noUnusedLocals

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
	() => props.config.topic,
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
		:title="`News: ${config.topic || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		:refresh-interval="refreshInterval"
		@refresh="loadData"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="articles.length === 0 && !loading && !error"
			class="text-center text-gray-500 py-8"
		>
			<p class="text-sm">No news articles found</p>
		</div>

		<div v-else class="space-y-3">
			<a
				v-for="(article, index) in articles.slice(0, 5)"
				:key="index"
				:href="article.url"
				target="_blank"
				rel="noopener noreferrer"
				class="block p-3 rounded-lg border border-gray-200 hover:border-blue-300 hover:shadow-md transition-all"
			>
				<div class="flex space-x-3">
					<!-- Article Image -->
					<div
						v-if="article.urlToImage"
						class="flex-shrink-0 w-20 h-20 rounded overflow-hidden bg-gray-100"
					>
						<img
							:src="article.urlToImage"
							:alt="article.title"
							class="w-full h-full object-cover"
							@error="(e) => ((e.target as HTMLImageElement).style.display = 'none')"
						/>
					</div>

					<!-- Article Content -->
					<div class="flex-1 min-w-0">
						<h4
							class="text-sm font-semibold text-gray-800 line-clamp-2 mb-1"
						>
							{{ article.title }}
						</h4>
						<p class="text-xs text-gray-600 line-clamp-2 mb-2">
							{{ article.description }}
						</p>
						<div
							class="flex items-center justify-between text-xs text-gray-500"
						>
							<span class="font-medium">{{
								article.source.name
							}}</span>
							<span>{{ formatTime(article.publishedAt) }}</span>
						</div>
					</div>
				</div>
			</a>
		</div>
	</BaseWidget>
</template>

<style scoped>
.line-clamp-2 {
	display: -webkit-box;
	-webkit-line-clamp: 2;
	-webkit-box-orient: vertical;
	overflow: hidden;
}
</style>
