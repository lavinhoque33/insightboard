<template>
	<div class="hero flex-1 bg-linear-to-br from-base-100 to-base-200 px-4 py-8">
		<div class="hero-content w-full max-w-md flex-col p-0">
			<div class="card card-border w-full bg-base-100 shadow-xl">
				<div class="card-body">
					<div class="mb-2 text-center">
						<h1
							class="mb-2 bg-linear-to-r from-primary to-secondary bg-clip-text text-4xl font-bold text-transparent"
						>
							Welcome Back
						</h1>
						<p class="text-sm text-base-content/70">
							Sign in to access your personalized dashboards
						</p>
					</div>

					<div
						v-if="sessionExpired"
						class="alert alert-info alert-soft"
						role="status"
					>
						<span class="text-sm"
							>Your session ended. Sign in again to continue.</span
						>
					</div>

					<div class="divider my-2"></div>

					<form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
						<div class="flex flex-col gap-1">
							<label for="email" class="text-sm font-semibold">
								Email Address
								<span class="text-error" aria-hidden="true">*</span>
							</label>
							<!-- Read-only (not disabled) while signing in, so focus stays put -->
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
									fieldError('password') ? 'password-error' : undefined
								"
								type="password"
								autocomplete="current-password"
								placeholder="Enter your password"
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
							:aria-disabled="authStore.loading"
							class="btn btn-primary w-full"
						>
							<span
								v-if="authStore.loading"
								class="loading loading-spinner loading-sm"
								aria-hidden="true"
							></span>
							{{ authStore.loading ? 'Signing in…' : 'Sign In' }}
						</button>
					</form>

					<div class="divider my-2"></div>

					<p class="text-center text-sm text-base-content/70">
						Don't have an account?
						<router-link
							:to="{ name: 'register', query: route.query }"
							class="link link-primary font-semibold"
						>
							Create one now
						</router-link>
					</p>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { safeRedirect } from '../router';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();

const fieldError = (field: string): string | undefined =>
	authStore.fieldErrors.find((e) => e.field === field)?.message;

const sessionExpired = computed(() => route.query.expired === '1');

const email = ref('');
const password = ref('');

// Clear error as user edits fields for smoother UX
watch([email, password], () => {
	if (authStore.error) {
		authStore.clearError();
	}
});

const handleSubmit = async () => {
	if (authStore.loading) return;
	try {
		await authStore.login(email.value, password.value);
	} catch {
		// The store keeps the message the form shows
		return;
	}
	// Back to the page that sent the user here, if any
	await router.push(safeRedirect(route.query.redirect));
};
</script>
