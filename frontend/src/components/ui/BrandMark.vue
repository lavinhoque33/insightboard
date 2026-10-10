<!-- InsightBoard logo mark (decorative); colours follow the theme. -->
<script setup lang="ts">
import { useId } from 'vue';

const gradientId = `brand-${useId()}`;
</script>

<template>
	<svg
		aria-hidden="true"
		focusable="false"
		xmlns="http://www.w3.org/2000/svg"
		viewBox="0 0 32 32"
		class="shrink-0 drop-shadow-sm"
	>
		<defs>
			<linearGradient :id="gradientId" x1="0" y1="0" x2="1" y2="1">
				<stop offset="0" style="stop-color: var(--color-primary)" />
				<stop offset="1" style="stop-color: var(--color-secondary)" />
			</linearGradient>
		</defs>
		<rect width="32" height="32" rx="9" :fill="`url(#${gradientId})`" />
		<rect x="7" y="16" width="4" height="9" rx="2" fill="#fff" opacity=".7" />
		<rect
			x="14"
			y="11"
			width="4"
			height="14"
			rx="2"
			fill="#fff"
			opacity=".85"
		/>
		<rect x="21" y="7" width="4" height="18" rx="2" fill="#fff" />
	</svg>
</template>
