/** * News Widget * Displays news articles filtered by topic */
<script setup lang="ts">
import { computed } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { useWidgetData } from '../../composables/useWidgetData';
import { fetchNewsData } from '../../api/widgets';

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

const { data, loading, error, lastUpdated, reload } = useWidgetData({
	source: () => props.config.topic,
	fetch: fetchNewsData,
	refreshInterval: () => props.refreshInterval,
	notConfigured: 'News topic not configured',
	failure: 'Failed to load news',
});
const articles = computed(() => data.value ?? []);

const formatTime = (dateString: string): string => {
	const date = new Date(dateString);
	const now = new Date();
	const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

	if (diff < 3600) return `${Math.floor(diff / 60)} minutes ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)} hours ago`;
	return `${Math.floor(diff / 86400)} days ago`;
};

// Removed unused truncateText helper to satisfy noUnusedLocals
</script>

<template>
	<BaseWidget
		:title="`News: ${config.topic || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<!-- Empty State -->
		<div
			v-if="articles.length === 0 && !loading && !error"
			class="flex flex-col items-center justify-center py-8 text-center"
		>
			<div class="text-4xl mb-3">📰</div>
			<p class="text-sm text-base-content/60">No news articles found</p>
		</div>

		<!-- Articles List -->
		<div v-else class="space-y-3">
			<a
				v-for="(article, index) in articles.slice(0, 5)"
				:key="index"
				:href="article.url"
				target="_blank"
				rel="noopener noreferrer"
				class="card card-border card-sm border-base-300 hover:border-primary hover:shadow-lg transition-all duration-300 cursor-pointer group bg-base-100"
			>
				<div class="card-body p-3">
					<!-- Title and Image -->
					<div class="flex gap-3">
						<!-- Article Image -->
						<div
							v-if="article.urlToImage"
							class="shrink-0 w-20 h-20 rounded-lg overflow-hidden bg-base-300"
						>
							<img
								:src="article.urlToImage"
								:alt="article.title"
								class="w-full h-full object-cover"
								@error="
									(e) => ((e.target as HTMLImageElement).style.display = 'none')
								"
							/>
						</div>

						<!-- Article Content -->
						<div class="flex-1 min-w-0 flex flex-col justify-between">
							<!-- Title -->
							<h4
								class="text-sm font-semibold text-base-content line-clamp-2 group-hover:text-primary transition-colors"
							>
								{{ article.title }}
							</h4>

							<!-- Source and Time -->
							<div class="flex items-center justify-between gap-2 mt-2">
								<span class="badge badge-sm badge-ghost text-xs">
									{{ article.source.name }}
								</span>
								<span class="text-xs text-base-content/50">
									{{ formatTime(article.publishedAt) }}
								</span>
							</div>
						</div>
					</div>

					<!-- Description (on second line) -->
					<p class="text-xs text-base-content/60 line-clamp-2 mt-2">
						{{ article.description }}
					</p>
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
