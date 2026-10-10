<!--
	Sign-in and sign-up page frame: product overview beside the form on wide
	screens, the form alone on narrow ones. The copy describes only what the
	app does today.
-->
<script setup lang="ts">
import WidgetTypeIcon from './ui/WidgetTypeIcon.vue';
import AppIcon from './ui/AppIcon.vue';
import type { WidgetType as WidgetKind } from '../types';

const WIDGETS: { type: WidgetKind; name: string; detail: string }[] = [
	{
		type: 'github',
		name: 'GitHub activity',
		detail: 'Recent events for a user',
	},
	{ type: 'weather', name: 'Weather', detail: 'Current conditions by city' },
	{ type: 'crypto', name: 'Crypto prices', detail: 'Price and 24h change' },
	{ type: 'news', name: 'News', detail: 'Headlines by topic' },
	{ type: 'status', name: 'Status checks', detail: 'Response codes and times' },
];

// Placeholder content lines in each tile's accent (decorative)
const BARS: Record<WidgetKind, string> = {
	github: 'bg-widget-github/20',
	weather: 'bg-widget-weather/20',
	news: 'bg-widget-news/20',
	crypto: 'bg-widget-crypto/20',
	status: 'bg-widget-status/20',
};
</script>

<template>
	<div class="grid flex-1 lg:grid-cols-[1.1fr_1fr]">
		<section
			class="relative hidden flex-col justify-center gap-10 overflow-hidden px-12 py-16 lg:flex xl:px-20"
		>
			<div class="max-w-lg animate-rise">
				<p
					class="mb-6 inline-flex items-center gap-2 rounded-full border border-base-300 bg-base-100/70 px-3 py-1 text-xs font-medium text-base-content/70 backdrop-blur"
				>
					<AppIcon name="sparkles" class="size-3.5 text-primary" />
					Personal data dashboards
				</p>
				<p
					class="text-gradient text-5xl font-semibold leading-[1.05] tracking-tight xl:text-6xl"
				>
					All your signals on one calm canvas.
				</p>
				<p class="mt-5 max-w-md text-lg leading-relaxed text-base-content/70">
					Arrange the widgets you need on a grid you shape yourself. Drag,
					resize, and configure them; layouts save automatically.
				</p>
			</div>

			<div class="relative max-w-xl animate-rise [animation-delay:120ms]">
				<div
					class="bg-brand absolute -inset-8 rounded-[3rem] opacity-15 blur-3xl"
					aria-hidden="true"
				></div>
				<div class="surface relative p-2.5 shadow-float">
					<div
						class="flex items-center gap-1.5 px-1.5 pb-2.5"
						aria-hidden="true"
					>
						<span class="size-2.5 rounded-full bg-base-300"></span>
						<span class="size-2.5 rounded-full bg-base-300"></span>
						<span class="size-2.5 rounded-full bg-base-300"></span>
					</div>
					<ul class="grid grid-cols-3 gap-2.5" aria-label="Available widgets">
						<li
							v-for="widget in WIDGETS"
							:key="widget.type"
							class="flex flex-col rounded-xl border border-base-300 bg-base-200/50 p-3"
							:class="{ 'row-span-2': widget.type === 'github' }"
						>
							<span class="flex items-center gap-2">
								<WidgetTypeIcon :type="widget.type" size="sm" />
								<span class="truncate text-sm font-medium">{{
									widget.name
								}}</span>
							</span>
							<span class="mt-1.5 text-xs text-base-content/70">{{
								widget.detail
							}}</span>
							<span class="mt-3 flex flex-col gap-1.5" aria-hidden="true">
								<span
									v-for="n in widget.type === 'github' ? 6 : 2"
									:key="n"
									class="h-1.5 rounded-full"
									:class="[BARS[widget.type], n % 2 ? 'w-full' : 'w-2/3']"
								></span>
							</span>
						</li>
					</ul>
				</div>
			</div>
		</section>

		<div
			class="flex items-start justify-center px-4 py-10 sm:items-center sm:py-16"
		>
			<div class="w-full max-w-md animate-rise [animation-delay:60ms]">
				<slot />
			</div>
		</div>
	</div>
</template>
