<!--
	Light / dark / system theme switch: a native radio group, so arrow keys move
	between the options and screen readers announce the choice.
-->
<script setup lang="ts">
import AppIcon from './ui/AppIcon.vue';
import type { IconName } from './ui/icons';
import { useTheme, type ThemeChoice } from '../composables/useTheme';

const { choice, setChoice } = useTheme();

const OPTIONS: { value: ThemeChoice; label: string; icon: IconName }[] = [
	{ value: 'light', label: 'Light', icon: 'sun' },
	{ value: 'dark', label: 'Dark', icon: 'moon' },
	{ value: 'system', label: 'System', icon: 'monitor' },
];
</script>

<template>
	<fieldset
		class="flex items-center gap-0.5 rounded-full border border-base-300 bg-base-200/70 p-0.5"
	>
		<legend class="sr-only">Theme</legend>
		<label
			v-for="option in OPTIONS"
			:key="option.value"
			:title="`${option.label} theme`"
			class="grid size-7 cursor-pointer place-items-center rounded-full text-base-content/70 transition-colors duration-150 hover:text-base-content has-checked:bg-base-100 has-checked:text-base-content has-checked:shadow-sm has-focus-visible:outline-2 has-focus-visible:outline-offset-1 has-focus-visible:outline-primary"
		>
			<input
				type="radio"
				name="theme"
				class="sr-only"
				:value="option.value"
				:checked="choice === option.value"
				@change="setChoice(option.value)"
			/>
			<AppIcon :name="option.icon" class="size-3.5" />
			<span class="sr-only">{{ option.label }}</span>
		</label>
	</fieldset>
</template>
