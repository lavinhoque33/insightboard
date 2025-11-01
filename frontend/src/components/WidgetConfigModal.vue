/** * Widget Configuration Modal * Dynamic form for editing widget settings
based on widget schema */
<script setup lang="ts">
import { ref, watch } from 'vue';
import type { WidgetConfigField } from '../stores/widgets';

interface Props {
	show: boolean;
	widgetTypeName: string;
	configSchema: WidgetConfigField[];
	initialConfig: Record<string, any>;
}

interface Emits {
	(e: 'close'): void;
	(e: 'save', config: Record<string, any>): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const formData = ref<Record<string, any>>({});
const errors = ref<Record<string, string>>({});

// Initialize form data when modal opens
watch(
	() => props.show,
	(show) => {
		if (show) {
			formData.value = { ...props.initialConfig };
			errors.value = {};
		}
	},
);

const validateForm = (): boolean => {
	errors.value = {};
	let isValid = true;

	for (const field of props.configSchema) {
		const value = formData.value[field.key];

		// Check required fields
		if (
			field.required &&
			(value === undefined || value === null || value === '')
		) {
			errors.value[field.key] = `${field.label} is required`;
			isValid = false;
			continue;
		}

		// Run custom validation
		if (
			field.validation &&
			value !== undefined &&
			value !== null &&
			value !== ''
		) {
			const error = field.validation(value);
			if (error) {
				errors.value[field.key] = error;
				isValid = false;
			}
		}
	}

	return isValid;
};

const handleSave = () => {
	if (validateForm()) {
		emit('save', { ...formData.value });
		emit('close');
	}
};

const handleClose = () => {
	emit('close');
};

const handleBackdropClick = (event: MouseEvent) => {
	if (event.target === event.currentTarget) {
		handleClose();
	}
};
</script>

<template>
	<Transition name="modal">
		<div
			v-if="show"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 p-4"
			@click="handleBackdropClick"
		>
			<div
				class="bg-white rounded-lg shadow-xl max-w-lg w-full max-h-[90vh] overflow-hidden flex flex-col"
			>
				<!-- Modal Header -->
				<div
					class="bg-gradient-to-r from-blue-500 to-blue-600 text-white px-6 py-4 flex items-center justify-between"
				>
					<h2 class="text-lg font-semibold">
						Configure {{ widgetTypeName }}
					</h2>
					<button
						@click="handleClose"
						class="text-white hover:bg-white/20 rounded p-1 transition-colors"
						title="Close"
					>
						<svg
							class="w-5 h-5"
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

				<!-- Modal Body -->
				<div class="flex-1 overflow-y-auto p-6">
					<form @submit.prevent="handleSave" class="space-y-4">
						<div
							v-for="field in configSchema"
							:key="field.key"
							class="form-group"
						>
							<label
								:for="field.key"
								class="block text-sm font-medium text-gray-700 mb-1"
							>
								{{ field.label }}
								<span v-if="field.required" class="text-red-500"
									>*</span
								>
							</label>

							<!-- Text Input -->
							<input
								v-if="
									field.type === 'text' ||
									field.type === 'url'
								"
								:id="field.key"
								v-model="formData[field.key]"
								:type="field.type"
								:placeholder="field.placeholder"
								:required="field.required"
								class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								:class="{ 'border-red-500': errors[field.key] }"
							/>

							<!-- Number Input -->
							<input
								v-else-if="field.type === 'number'"
								:id="field.key"
								v-model.number="formData[field.key]"
								type="number"
								:placeholder="field.placeholder"
								:required="field.required"
								class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								:class="{ 'border-red-500': errors[field.key] }"
							/>

							<!-- Textarea -->
							<textarea
								v-else-if="field.type === 'textarea'"
								:id="field.key"
								v-model="formData[field.key]"
								:placeholder="field.placeholder"
								:required="field.required"
								rows="3"
								class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
								:class="{ 'border-red-500': errors[field.key] }"
							></textarea>

							<!-- Select Dropdown -->
							<select
								v-else-if="field.type === 'select'"
								:id="field.key"
								v-model="formData[field.key]"
								:required="field.required"
								class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								:class="{ 'border-red-500': errors[field.key] }"
							>
								<option value="">
									Select {{ field.label }}
								</option>
								<option
									v-for="option in field.options"
									:key="option.value"
									:value="option.value"
								>
									{{ option.label }}
								</option>
							</select>

							<!-- Checkbox -->
							<div
								v-else-if="field.type === 'checkbox'"
								class="flex items-center"
							>
								<input
									:id="field.key"
									v-model="formData[field.key]"
									type="checkbox"
									class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
								/>
								<label
									:for="field.key"
									class="ml-2 text-sm text-gray-600"
								>
									{{ field.helpText || 'Enable' }}
								</label>
							</div>

							<!-- Help Text -->
							<p
								v-if="
									field.helpText && field.type !== 'checkbox'
								"
								class="mt-1 text-xs text-gray-500"
							>
								{{ field.helpText }}
							</p>

							<!-- Error Message -->
							<p
								v-if="errors[field.key]"
								class="mt-1 text-xs text-red-600"
							>
								{{ errors[field.key] }}
							</p>
						</div>
					</form>
				</div>

				<!-- Modal Footer -->
				<div
					class="bg-gray-50 px-6 py-4 flex items-center justify-end space-x-3 border-t border-gray-200"
				>
					<button
						@click="handleClose"
						type="button"
						class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
					>
						Cancel
					</button>
					<button
						@click="handleSave"
						type="button"
						class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
					>
						Save Changes
					</button>
				</div>
			</div>
		</div>
	</Transition>
</template>

<style scoped>
/* Modal transition animations */
.modal-enter-active,
.modal-leave-active {
	transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
	opacity: 0;
}

.modal-enter-active .bg-white,
.modal-leave-active .bg-white {
	transition: transform 0.3s ease;
}

.modal-enter-from .bg-white,
.modal-leave-to .bg-white {
	transform: scale(0.9);
}
</style>
