<template>
	<AuthLayout>
		<div class="surface p-6 shadow-float sm:p-8">
			<div class="mb-6">
				<BrandMark class="mb-5 size-10 lg:hidden" />
				<h1 class="text-2xl font-semibold tracking-tight sm:text-3xl">
					Join InsightBoard
				</h1>
				<p class="mt-1.5 text-sm text-base-content/70">
					Create your free account to start building dashboards
				</p>
			</div>

			<form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
				<div class="flex flex-col gap-1.5">
					<label for="email" class="text-sm font-medium">
						Email Address
						<span class="text-error" aria-hidden="true">*</span>
					</label>
					<!-- Read-only (not disabled) while submitting, so focus stays put -->
					<input
						id="email"
						v-model="email"
						:aria-invalid="!!fieldError('email')"
						:aria-describedby="fieldError('email') ? 'email-error' : undefined"
						type="email"
						autocomplete="email"
						placeholder="you@example.com"
						class="input w-full"
						:class="{ 'input-error': fieldError('email') }"
						required
						:readonly="authStore.loading"
					/>
					<p
						v-if="fieldError('email')"
						id="email-error"
						class="text-sm text-error"
						role="alert"
					>
						{{ fieldError('email') }}
					</p>
				</div>

				<div class="flex flex-col gap-1.5">
					<label for="password" class="text-sm font-medium">
						Password
						<span class="text-error" aria-hidden="true">*</span>
					</label>
					<input
						id="password"
						v-model="password"
						:aria-invalid="!!fieldError('password')"
						:aria-describedby="
							fieldError('password')
								? 'password-error password-hint'
								: 'password-hint'
						"
						type="password"
						autocomplete="new-password"
						placeholder="Create a strong password"
						class="input w-full"
						:class="{ 'input-error': fieldError('password') }"
						required
						:readonly="authStore.loading"
					/>
					<p
						v-if="fieldError('password')"
						id="password-error"
						class="text-sm text-error"
						role="alert"
					>
						{{ fieldError('password') }}
					</p>
					<div class="flex items-center gap-3">
						<div
							class="h-1 flex-1 overflow-hidden rounded-full bg-base-300"
							aria-hidden="true"
						>
							<div
								class="h-full rounded-full transition-[width] duration-300"
								:class="password.length >= 8 ? 'bg-success' : 'bg-primary'"
								:style="{
									width: `${Math.min(password.length, 8) * 12.5}%`,
								}"
							></div>
						</div>
						<p
							id="password-hint"
							class="text-xs tabular-nums text-base-content/70"
						>
							Minimum 8 characters ({{ password.length }}/8)
						</p>
					</div>
				</div>

				<div class="flex flex-col gap-1.5">
					<label for="confirm-password" class="text-sm font-medium">
						Confirm Password
						<span class="text-error" aria-hidden="true">*</span>
					</label>
					<input
						id="confirm-password"
						v-model="confirmPassword"
						type="password"
						autocomplete="new-password"
						placeholder="Re-enter your password"
						class="input w-full"
						required
						:readonly="authStore.loading"
					/>
					<p
						v-if="confirmPassword && password === confirmPassword"
						class="flex items-center gap-1.5 text-xs text-success"
					>
						<AppIcon name="check" class="size-3.5" />
						Passwords match
					</p>
				</div>

				<div
					v-if="validationError"
					class="alert alert-warning alert-soft"
					role="alert"
				>
					<AppIcon name="alert-triangle" class="size-4" />
					<span class="text-sm">{{ validationError }}</span>
				</div>

				<div
					v-if="authStore.error"
					class="alert alert-error alert-soft"
					role="alert"
				>
					<AppIcon name="alert-circle" class="size-4" />
					<span class="text-sm">{{ authStore.error }}</span>
				</div>

				<button
					type="submit"
					:aria-disabled="authStore.loading || !!validationError"
					class="btn btn-primary mt-1 w-full"
				>
					<span
						v-if="authStore.loading"
						class="loading loading-spinner loading-sm"
						aria-hidden="true"
					></span>
					{{ authStore.loading ? 'Creating account…' : 'Create Account' }}
					<AppIcon
						v-if="!authStore.loading"
						name="arrow-right"
						class="size-4"
					/>
				</button>
			</form>

			<p
				class="mt-6 border-t border-base-300 pt-5 text-center text-sm text-base-content/70"
			>
				Already have an account?
				<router-link
					:to="{ name: 'login', query: route.query }"
					class="link link-primary font-medium"
				>
					Sign in here
				</router-link>
			</p>
		</div>
	</AuthLayout>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { safeRedirect } from '../router';
import AuthLayout from '../components/AuthLayout.vue';
import AppIcon from '../components/ui/AppIcon.vue';
import BrandMark from '../components/ui/BrandMark.vue';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();

const fieldError = (field: string): string | undefined =>
	authStore.fieldErrors.find((e) => e.field === field)?.message;

const email = ref('');
const password = ref('');
const confirmPassword = ref('');

// Clear auth errors as the user types for smoother UX
watch([email, password, confirmPassword], () => {
	if (authStore.error) {
		authStore.clearError();
	}
});

const validationError = computed(() => {
	if (password.value && password.value.length < 8) {
		return 'Password must be at least 8 characters long';
	}
	if (confirmPassword.value && password.value !== confirmPassword.value) {
		return 'Passwords do not match';
	}
	return null;
});

const handleSubmit = async () => {
	if (authStore.loading || validationError.value) return;
	authStore.clearError();
	if (password.value !== confirmPassword.value) return;

	try {
		await authStore.register(email.value, password.value);
	} catch {
		// The store keeps the message the form shows
		return;
	}
	// Register returns a token: continue where the user was headed
	await router.push(safeRedirect(route.query.redirect));
};
</script>
