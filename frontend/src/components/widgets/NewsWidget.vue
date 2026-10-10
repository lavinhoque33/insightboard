<!-- News Widget: news articles filtered by topic -->
<script setup lang="ts">
import { computed, ref } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import AppIcon from '../ui/AppIcon.vue';
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
const articles = computed(() => (data.value ?? []).slice(0, 5));

// Images that fail to load fall back to the source initial
const brokenImages = ref(new Set<string>());

const formatTime = (dateString: string): string => {
	const diff = Math.max(
		0,
		Math.floor((Date.now() - new Date(dateString).getTime()) / 1000),
	);

	if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
	return `${Math.floor(diff / 86400)}d ago`;
};
</script>

<template>
	<BaseWidget
		type="news"
		:title="`News: ${config.topic || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="articles.length === 0 && !loading && !error"
			class="flex h-full flex-col items-center justify-center gap-2 py-6 text-center"
		>
			<AppIcon name="newspaper" class="size-7 text-base-content/40" />
			<p class="text-sm text-base-content/70">No news articles found</p>
		</div>

		<ul v-else class="-mx-1.5 flex flex-col">
			<li v-for="article in articles" :key="article.url">
				<a
					:href="article.url"
					target="_blank"
					rel="noopener noreferrer"
					class="group flex gap-3 rounded-lg px-1.5 py-2 transition-colors hover:bg-base-200/80"
				>
					<span
						class="relative grid size-14 shrink-0 place-items-center overflow-hidden rounded-lg bg-linear-to-br from-widget-news/25 to-widget-news/5 text-lg font-semibold text-widget-news ring-1 ring-inset ring-base-content/5"
						aria-hidden="true"
					>
						{{ article.source.name.charAt(0).toUpperCase() }}
						<img
							v-if="article.urlToImage && !brokenImages.has(article.url)"
							:src="article.urlToImage"
							alt=""
							loading="lazy"
							class="absolute inset-0 size-full object-cover"
							@error="brokenImages.add(article.url)"
						/>
					</span>
					<span class="flex min-w-0 flex-1 flex-col gap-1">
						<span
							class="line-clamp-2 text-sm font-medium leading-snug transition-colors group-hover:text-primary"
						>
							{{ article.title }}
							<span class="sr-only">(opens in a new tab)</span>
						</span>
						<span
							v-if="article.description"
							class="line-clamp-1 text-xs text-base-content/70"
						>
							{{ article.description }}
						</span>
						<span
							class="flex items-center gap-1.5 text-[11px] text-base-content/70"
						>
							<span class="truncate font-medium">{{
								article.source.name
							}}</span>
							<span aria-hidden="true">·</span>
							<time :datetime="article.publishedAt" class="tabular-nums">{{
								formatTime(article.publishedAt)
							}}</time>
							<AppIcon
								name="external"
								class="ml-auto size-3 opacity-0 transition-opacity group-hover:opacity-100"
							/>
						</span>
					</span>
				</a>
			</li>
		</ul>
	</BaseWidget>
</template>
