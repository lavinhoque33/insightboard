<!-- Accent-tinted icon chip for a widget type (decorative). -->
<script setup lang="ts">
import type { WidgetType as WidgetKind } from '../../types';
import AppIcon from './AppIcon.vue';
import { WIDGET_VISUALS } from './widgetVisuals';

withDefaults(defineProps<{ type: WidgetKind; size?: 'sm' | 'md' | 'lg' }>(), {
	size: 'md',
});

const SIZES = {
	sm: 'size-6 rounded-md text-sm',
	md: 'size-8 rounded-lg text-base',
	lg: 'size-10 rounded-xl text-xl',
} as const;
</script>

<template>
	<span
		aria-hidden="true"
		class="inline-flex shrink-0 items-center justify-center ring-1 ring-inset"
		:class="[WIDGET_VISUALS[type].chip, SIZES[size]]"
	>
		<AppIcon :name="WIDGET_VISUALS[type].icon" />
	</span>
</template>
