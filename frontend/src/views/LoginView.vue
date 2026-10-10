<template>
	<AuthLayout>
		<div class="surface p-6 shadow-float sm:p-8">
			<div class="mb-6">
				<BrandMark class="mb-5 size-10 lg:hidden" />
				<h1 class="text-2xl font-semibold tracking-tight sm:text-3xl">
					Welcome Back
				</h1>
				<p class="mt-1.5 text-sm text-base-content/70">
					Sign in to access your personalized dashboards
				</p>
			</div>

			<div
				v-if="sessionExpired"
				class="alert alert-info alert-soft mb-5"
				role="status"
			>
				<AppIcon name="clock" class="size-4" />
				<span class="text-sm"
					>Your session ended. Sign in again to continue.</span
				>
			</div>

			<form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
				<div class="flex flex-col gap-1.5">
					<label for="email" class="text-sm font-medium">
						Email Address
						<span class="text-error" aria-hidden="true">*</span>
					</label>
					<!-- Read-only (not disabled) while signing in, so focus stays put -->
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
							fieldError('password') ? 'password-error' : undefined
						"
						type="password"
						autocomplete="current-password"
						placeholder="Enter your password"
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
					:aria-disabled="authStore.loading"
					class="btn btn-primary mt-1 w-full"
				>
					<span
						v-if="authStore.loading"
						class="loading loading-spinner loading-sm"
						aria-hidden="true"
					></span>
					{{ authStore.loading ? 'Signing in…' : 'Sign In' }}
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
				Don't have an account?
				<router-link
					:to="{ name: 'register', query: route.query }"
					class="link link-primary font-medium"
				>
					Create one now
				</router-link>
			</p>
		</div>
	</AuthLayout>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
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
