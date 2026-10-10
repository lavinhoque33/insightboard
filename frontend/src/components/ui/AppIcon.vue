<!--
	Decorative stroke icon, 1em square unless a size class is given; the control
	around it carries the accessible name.
-->
<script setup lang="ts">
import { ICONS, type IconName } from './icons';

defineProps<{ name: IconName }>();
</script>

<template>
	<svg
		aria-hidden="true"
		focusable="false"
		xmlns="http://www.w3.org/2000/svg"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="1.75"
		stroke-linecap="round"
		stroke-linejoin="round"
		width="1em"
		height="1em"
		class="shrink-0"
	>
		<path v-for="d in ICONS[name]" :key="d" :d="d" />
	</svg>
</template>
