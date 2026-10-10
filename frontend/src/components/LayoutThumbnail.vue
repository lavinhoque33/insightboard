<!--
	Miniature of a dashboard's real 12-column layout, one tinted block per
	widget in its type's accent. Decorative: the card states the widget count.
-->
<script setup lang="ts">
import { computed } from 'vue';
import type { LayoutConfig } from '../types';

const props = defineProps<{ layout: LayoutConfig }>();

// One column is 10 units wide; a row (80 px cells) is about 0.7 of a column
const COL = 10;
const ROW = 7;
const GAP = 1.2;
const MIN_ROWS = 4;

const rows = computed(() =>
	Math.max(MIN_ROWS, ...props.layout.widgets.map((w) => w.y + w.h)),
);

const FILLS = {
	github: 'fill-widget-github/20 stroke-widget-github/60',
	weather: 'fill-widget-weather/20 stroke-widget-weather/60',
	news: 'fill-widget-news/20 stroke-widget-news/60',
	crypto: 'fill-widget-crypto/20 stroke-widget-crypto/60',
	status: 'fill-widget-status/20 stroke-widget-status/60',
} as const;
</script>

<template>
	<svg
		aria-hidden="true"
		focusable="false"
		xmlns="http://www.w3.org/2000/svg"
		:viewBox="`0 0 ${12 * COL} ${rows * ROW}`"
		preserveAspectRatio="xMidYMid meet"
		class="absolute inset-0 size-full"
	>
		<rect
			v-for="widget in layout.widgets"
			:key="widget.id"
			:x="widget.x * COL + GAP / 2"
			:y="widget.y * ROW + GAP / 2"
			:width="Math.max(widget.w * COL - GAP, 1)"
			:height="Math.max(widget.h * ROW - GAP, 1)"
			rx="2.5"
			stroke-width="0.6"
			:class="FILLS[widget.type]"
		/>
	</svg>
</template>
