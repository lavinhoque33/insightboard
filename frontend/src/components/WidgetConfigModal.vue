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
		<!-- daisyUI Modal Dialog -->
		<div v-if="show" class="modal modal-open" @click="handleBackdropClick">
			<div
				class="modal-box w-full max-w-md bg-base-100 shadow-2xl"
				@click.stop
			>
				<!-- Modal Header with Gradient -->
				<div class="pb-4 border-b border-base-300">
					<div class="flex items-center justify-between">
						<h3
							class="font-bold text-lg text-transparent bg-gradient-to-r from-primary to-secondary bg-clip-text"
						>
							⚙️ Configure {{ widgetTypeName }}
						</h3>
						<button
							@click="handleClose"
							class="btn btn-ghost btn-sm btn-circle"
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
				</div>

				<!-- Modal Body with Form -->
				<form
					@submit.prevent="handleSave"
					class="py-4 space-y-5 max-h-96 overflow-y-auto"
				>
					<!-- Dynamic Form Fields -->
					<div
						v-for="field in configSchema"
						:key="field.key"
						class="form-control w-full px-4"
					>
						<!-- Label with Required Indicator -->
						<label :for="field.key" class="label">
							<span class="label-text font-medium">
								{{ field.label }}
								<span
									v-if="field.required"
									class="text-error ml-1"
									>*</span
								>
							</span>
						</label>

						<!-- Text Input -->
						<input
							v-if="field.type === 'text' || field.type === 'url'"
							:id="field.key"
							v-model="formData[field.key]"
							:type="field.type"
							:placeholder="field.placeholder"
							:required="field.required"
							class="input input-bordered input-primary w-full transition-all duration-300"
							:class="{ 'input-error': errors[field.key] }"
						/>

						<!-- Number Input -->
						<input
							v-else-if="field.type === 'number'"
							:id="field.key"
							v-model.number="formData[field.key]"
							type="number"
							:placeholder="field.placeholder"
							:required="field.required"
							class="input input-bordered input-primary w-full transition-all duration-300"
							:class="{ 'input-error': errors[field.key] }"
						/>

						<!-- Textarea -->
						<textarea
							v-else-if="field.type === 'textarea'"
							:id="field.key"
							v-model="formData[field.key]"
							:placeholder="field.placeholder"
							:required="field.required"
							rows="3"
							class="textarea textarea-bordered textarea-primary w-full resize-none transition-all duration-300"
							:class="{ 'textarea-error': errors[field.key] }"
						></textarea>

						<!-- Select Dropdown -->
						<select
							v-else-if="field.type === 'select'"
							:id="field.key"
							v-model="formData[field.key]"
							:required="field.required"
							class="select select-bordered select-primary w-full transition-all duration-300"
							:class="{ 'select-error': errors[field.key] }"
						>
							<option value="" disabled selected>
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
							class="flex items-center gap-3 mt-2"
						>
							<input
								:id="field.key"
								v-model="formData[field.key]"
								type="checkbox"
								class="checkbox checkbox-primary"
							/>
							<label
								:for="field.key"
								class="label cursor-pointer flex-1"
							>
								<span class="label-text">
									{{ field.helpText || 'Enable' }}
								</span>
							</label>
						</div>

						<!-- Help Text -->
						<label
							v-if="field.helpText && field.type !== 'checkbox'"
							:for="field.key"
							class="label"
						>
							<span
								class="label-text-alt text-base-content/60 italic"
							>
								{{ field.helpText }}
							</span>
						</label>

						<!-- Error Message Alert -->
						<div
							v-if="errors[field.key]"
							class="alert alert-error shadow-md mt-2 py-2"
						>
							<div class="flex gap-2 items-start">
								<!-- Error Icon -->
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="stroke-current flex-shrink-0 h-5 w-5 mt-0.5"
									fill="none"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M10 14l-2-2m0 0l-2-2m2 2l2-2m-2 2l-2 2m8-8l-2 2m0 0l-2 2m2-2l2 2m-2-2l-2-2"
									/>
								</svg>
								<div class="flex-1">
									<h3
										class="font-semibold text-sm leading-tight"
									>
										Validation Error
									</h3>
									<p class="text-xs mt-0.5">
										{{ errors[field.key] }}
									</p>
								</div>
							</div>
						</div>
					</div>
				</form>

				<!-- Modal Footer with Action Buttons -->
				<div
					class="modal-action pt-6 border-t border-base-300 gap-3 px-4"
				>
					<button
						@click="handleClose"
						type="button"
						class="btn btn-ghost btn-sm"
					>
						Cancel
					</button>
					<button
						@click="handleSave"
						type="button"
						class="btn btn-primary btn-sm gap-2"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M5 13l4 4L19 7"
							/>
						</svg>
						Save Changes
					</button>
				</div>
			</div>
		</div>
	</Transition>
</template>

<style scoped>
@reference "tailwindcss";
/* Modal transition animations with daisyUI integration */
.modal-enter-active,
.modal-leave-active {
	transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
	opacity: 0;
}

.modal-enter-active .modal-box,
.modal-leave-active .modal-box {
	transition: transform 0.3s ease;
}

.modal-enter-from .modal-box,
.modal-leave-to .modal-box {
	transform: scale(0.95);
}

/* Smooth input transitions */
:deep(.input),
:deep(.textarea),
:deep(.select),
:deep(.checkbox) {
	@apply transition-all duration-300;
}

/* Focus states for better UX */
:deep(.input:focus),
:deep(.textarea:focus),
:deep(.select:focus) {
	@apply shadow-md;
}
</style>
