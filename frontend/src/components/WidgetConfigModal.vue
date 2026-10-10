<!--
	Widget Configuration Modal
	Dynamic form for editing widget settings based on the registry schema. The
	parent mounts it with v-if, so the form is filled from `initialConfig` when it
	opens: the current configuration for an existing widget, registry defaults for
	a new one.
-->
<script setup lang="ts">
import { ref } from 'vue';
import type { WidgetConfigField } from '../stores/widgets';
import type { WidgetConfig } from '../types';
import { useDialogFocus } from '../composables/useDialogFocus';

interface Props {
	widgetTypeName: string;
	configSchema: WidgetConfigField[];
	initialConfig: WidgetConfig;
}

interface Emits {
	(e: 'close'): void;
	(e: 'save', config: WidgetConfig): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// Every schema field holds a string (text, textarea, select)
const formData = ref<Record<string, string>>(
	Object.fromEntries(
		props.configSchema.map((field) => {
			const value = props.initialConfig[field.key];
			return [field.key, typeof value === 'string' ? value : ''];
		}),
	),
);
const errors = ref<Record<string, string>>({});

const panel = ref<HTMLElement | null>(null);
const { onKeydown } = useDialogFocus(panel, () => emit('close'));

const inputId = (key: string) => `widget-config-${key}`;

const describedBy = (field: WidgetConfigField) =>
	[
		errors.value[field.key] ? `${inputId(field.key)}-error` : null,
		field.helpText ? `${inputId(field.key)}-help` : null,
	]
		.filter(Boolean)
		.join(' ') || undefined;

const validate = (values: Record<string, string>): boolean => {
	const found: Record<string, string> = {};
	for (const field of props.configSchema) {
		const value = values[field.key] ?? '';
		if (!value) {
			if (field.required) found[field.key] = `${field.label} is required`;
			continue;
		}
		const message = field.type === 'select' ? null : field.validation?.(value);
		if (message) found[field.key] = message;
	}
	errors.value = found;
	return Object.keys(found).length === 0;
};

const handleSave = () => {
	const values = Object.fromEntries(
		Object.entries(formData.value).map(([key, value]) => [key, value.trim()]),
	);
	if (!validate(values)) {
		const firstInvalid = props.configSchema.find((f) => errors.value[f.key]);
		if (firstInvalid)
			document.getElementById(inputId(firstInvalid.key))?.focus();
		return;
	}
	// Keys the schema does not render are kept unchanged
	emit('save', { ...props.initialConfig, ...values });
	emit('close');
};
</script>

<template>
	<div class="modal modal-open" @click.self="emit('close')">
		<div
			ref="panel"
			class="modal-box w-full max-w-md"
			role="dialog"
			aria-modal="true"
			aria-labelledby="widget-config-title"
			@keydown="onKeydown"
		>
			<div class="flex items-center justify-between gap-2 pb-4">
				<h2 id="widget-config-title" class="text-lg font-bold">
					Configure {{ widgetTypeName }}
				</h2>
				<button
					type="button"
					class="btn btn-ghost btn-sm btn-circle"
					aria-label="Close"
					@click="emit('close')"
				>
					<svg
						aria-hidden="true"
						class="h-5 w-5"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
				</button>
			</div>

			<form class="flex flex-col gap-4" novalidate @submit.prevent="handleSave">
				<div
					v-for="field in configSchema"
					:key="field.key"
					class="flex flex-col gap-1"
				>
					<label :for="inputId(field.key)" class="text-sm font-medium">
						{{ field.label }}
						<span v-if="field.required" class="text-error" aria-hidden="true"
							>*</span
						>
					</label>

					<input
						v-if="field.type === 'text'"
						:id="inputId(field.key)"
						v-model="formData[field.key]"
						type="text"
						:placeholder="field.placeholder"
						:aria-required="field.required"
						:aria-invalid="!!errors[field.key]"
						:aria-describedby="describedBy(field)"
						class="input w-full"
						:class="{ 'input-error': errors[field.key] }"
					/>

					<textarea
						v-else-if="field.type === 'textarea'"
						:id="inputId(field.key)"
						v-model="formData[field.key]"
						:placeholder="field.placeholder"
						:aria-required="field.required"
						:aria-invalid="!!errors[field.key]"
						:aria-describedby="describedBy(field)"
						rows="3"
						class="textarea w-full"
						:class="{ 'textarea-error': errors[field.key] }"
					></textarea>

					<select
						v-else-if="field.type === 'select'"
						:id="inputId(field.key)"
						v-model="formData[field.key]"
						:aria-required="field.required"
						:aria-invalid="!!errors[field.key]"
						:aria-describedby="describedBy(field)"
						class="select w-full"
						:class="{ 'select-error': errors[field.key] }"
					>
						<option value="" disabled>Select {{ field.label }}</option>
						<option
							v-for="option in field.options"
							:key="option.value"
							:value="option.value"
						>
							{{ option.label }}
						</option>
					</select>

					<p
						v-if="field.helpText"
						:id="`${inputId(field.key)}-help`"
						class="text-xs text-base-content/70"
					>
						{{ field.helpText }}
					</p>
					<p
						v-if="errors[field.key]"
						:id="`${inputId(field.key)}-error`"
						class="text-sm text-error"
						role="alert"
					>
						{{ errors[field.key] }}
					</p>
				</div>

				<div class="modal-action mt-2">
					<button
						type="button"
						class="btn btn-ghost btn-sm"
						@click="emit('close')"
					>
						Cancel
					</button>
					<button type="submit" class="btn btn-primary btn-sm">
						Save Changes
					</button>
				</div>
			</form>
		</div>
	</div>
</template>
