<template>
	<div class="hero flex-1 bg-linear-to-br from-base-100 to-base-200 px-4 py-8">
		<div class="hero-content w-full max-w-md flex-col p-0">
			<div class="card card-border w-full bg-base-100 shadow-xl">
				<div class="card-body">
					<div class="mb-2 text-center">
						<h1
							class="mb-2 bg-linear-to-r from-primary to-secondary bg-clip-text text-4xl font-bold text-transparent"
						>
							Join InsightBoard
						</h1>
						<p class="text-sm text-base-content/70">
							Create your free account to start building dashboards
						</p>
					</div>

					<div class="divider my-2"></div>

					<form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
						<div class="flex flex-col gap-1">
							<label for="email" class="text-sm font-semibold">
								Email Address
								<span class="text-error" aria-hidden="true">*</span>
							</label>
							<!-- Read-only (not disabled) while submitting, so focus stays put -->
							<input
								id="email"
								v-model="email"
								:aria-invalid="!!fieldError('email')"
								:aria-describedby="
									fieldError('email') ? 'email-error' : undefined
								"
								type="email"
								autocomplete="email"
								placeholder="you@example.com"
								class="input w-full"
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

						<div class="flex flex-col gap-1">
							<label for="password" class="text-sm font-semibold">
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
							<p id="password-hint" class="text-xs text-base-content/70">
								Minimum 8 characters ({{ password.length }}/8)
							</p>
						</div>

						<div class="flex flex-col gap-1">
							<label for="confirm-password" class="text-sm font-semibold">
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
								class="text-xs text-success"
							>
								✓ Passwords match
							</p>
						</div>

						<div
							v-if="validationError"
							class="alert alert-warning alert-soft"
							role="alert"
						>
							<span class="text-sm">{{ validationError }}</span>
						</div>

						<div
							v-if="authStore.error"
							class="alert alert-error alert-soft"
							role="alert"
						>
							<span class="text-sm">{{ authStore.error }}</span>
						</div>

						<button
							type="submit"
							:aria-disabled="authStore.loading || !!validationError"
							class="btn btn-primary w-full"
						>
							<span
								v-if="authStore.loading"
								class="loading loading-spinner loading-sm"
								aria-hidden="true"
							></span>
							{{ authStore.loading ? 'Creating account…' : 'Create Account' }}
						</button>
					</form>

					<div class="divider my-2"></div>

					<p class="text-center text-sm text-base-content/70">
						Already have an account?
						<router-link
							:to="{ name: 'login', query: route.query }"
							class="link link-primary font-semibold"
						>
							Sign in here
						</router-link>
					</p>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { safeRedirect } from '../router';

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
