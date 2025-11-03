<template>
	<div
		class="hero min-h-[calc(100vh-70px)] bg-gradient-to-br from-base-100 to-base-200"
	>
		<div class="hero-content flex-col w-full max-w-md">
			<!-- Card Container -->
			<div class="card bg-base-100 shadow-2xl border border-base-300">
				<!-- Header -->
				<div class="card-body">
					<div class="text-center mb-4">
						<h1
							class="text-4xl font-bold text-transparent bg-gradient-to-r from-primary to-secondary bg-clip-text mb-2"
						>
							Welcome Back
						</h1>
						<p class="text-base-content/70 text-sm">
							Sign in to access your personalized dashboards
						</p>
					</div>

					<!-- Divider -->
					<div class="divider my-4"></div>

					<!-- Login Form -->
					<form @submit.prevent="handleSubmit" class="space-y-5">
						<!-- Email Field -->
						<div class="form-control w-full">
							<label class="label">
								<span
									class="label-text font-semibold text-base-content"
									>Email Address</span
								>
								<span class="label-text-alt text-error">*</span>
							</label>
							<input
								id="email"
								v-model="email"
								type="email"
								placeholder="you@example.com"
								class="input input-bordered input-primary w-full focus:outline-none transition-all"
								required
								:disabled="authStore.loading"
							/>
							<label class="label">
								<span
									class="label-text-alt text-base-content/60"
									>We'll never share your email</span
								>
							</label>
						</div>

						<!-- Password Field -->
						<div class="form-control w-full">
							<label class="label">
								<span
									class="label-text font-semibold text-base-content"
									>Password</span
								>
								<span class="label-text-alt text-error">*</span>
							</label>
							<input
								id="password"
								v-model="password"
								type="password"
								placeholder="Enter your password"
								class="input input-bordered input-primary w-full focus:outline-none transition-all"
								required
								:disabled="authStore.loading"
							/>
						</div>

						<!-- Error Alert -->
						<transition
							enter-active-class="transition ease-out duration-200"
							enter-from-class="opacity-0 -translate-y-2"
							enter-to-class="opacity-100 translate-y-0"
							leave-active-class="transition ease-in duration-150"
							leave-from-class="opacity-100 translate-y-0"
							leave-to-class="opacity-0 -translate-y-2"
						>
							<div
								v-if="authStore.error"
								class="alert alert-error shadow-lg"
								role="alert"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="stroke-current shrink-0 h-6 w-6"
									fill="none"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M10 14l-2-2m0 0l-2-2m2 2l2-2m-2 2l-2 2m2-2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
									/>
								</svg>
								<span class="text-sm">{{
									authStore.error
								}}</span>
							</div>
						</transition>

						<!-- Submit Button -->
						<button
							type="submit"
							:disabled="authStore.loading"
							class="btn btn-primary w-full gap-2 font-semibold text-base"
						>
							<template v-if="!authStore.loading">
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-5 w-5"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
									/>
								</svg>
								Sign In
							</template>
							<template v-else>
								<span
									class="loading loading-spinner loading-sm"
								></span>
								Signing in...
							</template>
						</button>
					</form>

					<!-- Divider -->
					<div class="divider my-4"></div>

					<!-- Sign Up Link -->
					<div class="text-center">
						<p class="text-sm text-base-content/70">
							Don't have an account?
							<router-link
								to="/register"
								class="link link-primary font-semibold"
							>
								Create one now
							</router-link>
						</p>
					</div>

					<!-- Info Box -->
					<div class="alert alert-info shadow-md">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
							class="stroke-current shrink-0 w-6 h-6"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
							></path>
						</svg>
						<span class="text-sm"
							>Test with any email and password. Development mode
							enabled!</span
						>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();

const email = ref('');
const password = ref('');

const handleSubmit = async () => {
	try {
		await authStore.login(email.value, password.value);

		// Redirect to intended destination or dashboards
		const redirect = route.query.redirect as string;
		router.push(redirect || '/dashboards');
	} catch (error) {
		// Error is handled by the store
		console.error('Login failed:', error);
	}
};
</script>
