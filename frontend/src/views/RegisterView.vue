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
							Join InsightBoard
						</h1>
						<p class="text-base-content/70 text-sm">
							Create your free account to start building
							dashboards
						</p>
					</div>

					<!-- Divider -->
					<div class="divider my-4"></div>

					<!-- Register Form -->
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
									>This will be your unique login</span
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
								placeholder="Enter a strong password"
								class="input input-bordered input-primary w-full focus:outline-none transition-all"
								required
								minlength="8"
								:disabled="authStore.loading"
							/>
							<label class="label">
								<span
									class="label-text-alt text-base-content/60"
									>Minimum 8 characters</span
								>
								<span
									class="label-text-alt flex items-center gap-1"
									:class="
										password.length >= 8
											? 'text-success'
											: 'text-warning'
									"
								>
									<span
										v-if="password.length >= 8"
										class="badge badge-success badge-xs"
									></span>
									<span
										v-else
										class="badge badge-warning badge-xs"
									></span>
									{{ password.length }}/8
								</span>
							</label>
						</div>

						<!-- Confirm Password Field -->
						<div class="form-control w-full">
							<label class="label">
								<span
									class="label-text font-semibold text-base-content"
									>Confirm Password</span
								>
								<span class="label-text-alt text-error">*</span>
							</label>
							<input
								id="confirm-password"
								v-model="confirmPassword"
								type="password"
								placeholder="Re-enter your password"
								class="input input-bordered input-primary w-full focus:outline-none transition-all"
								required
								:disabled="authStore.loading"
							/>
							<label class="label">
								<span
									class="label-text-alt flex items-center gap-1"
									:class="
										confirmPassword &&
										password === confirmPassword
											? 'text-success'
											: confirmPassword &&
											  password !== confirmPassword
											? 'text-error'
											: ''
									"
								>
									<template
										v-if="
											confirmPassword &&
											password === confirmPassword
										"
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											class="h-4 w-4 text-success"
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
										Passwords match
									</template>
									<template
										v-else-if="
											confirmPassword &&
											password !== confirmPassword
										"
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											class="h-4 w-4 text-error"
											fill="none"
											viewBox="0 0 24 24"
											stroke="currentColor"
										>
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M6 18L18 6M6 6l12 12"
											/>
										</svg>
										Passwords don't match
									</template>
									<span v-else></span>
								</span>
							</label>
						</div>

						<!-- Validation Error -->
						<transition
							enter-active-class="transition ease-out duration-200"
							enter-from-class="opacity-0 -translate-y-2"
							enter-to-class="opacity-100 translate-y-0"
							leave-active-class="transition ease-in duration-150"
							leave-from-class="opacity-100 translate-y-0"
							leave-to-class="opacity-0 -translate-y-2"
						>
							<div
								v-if="validationError"
								class="alert alert-warning shadow-lg"
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
										d="M12 8v4m0 4v.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
									/>
								</svg>
								<span class="text-sm">{{
									validationError
								}}</span>
							</div>
						</transition>

						<!-- API Error Alert -->
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
							:disabled="authStore.loading || !!validationError"
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
										d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM9 19c-4.35 0-8 1.343-8 3v2h16v-2c0-1.657-3.65-3-8-3z"
									/>
								</svg>
								Create Account
							</template>
							<template v-else>
								<span
									class="loading loading-spinner loading-sm"
								></span>
								Creating Account...
							</template>
						</button>
					</form>

					<!-- Divider -->
					<div class="divider my-4"></div>

					<!-- Sign In Link -->
					<div class="text-center">
						<p class="text-sm text-base-content/70">
							Already have an account?
							<router-link
								to="/login"
								class="link link-primary font-semibold"
							>
								Sign in here
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
							>Your account is free and instant. No credit card
							required.</span
						>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const router = useRouter();
const authStore = useAuthStore();

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
	// Clear previous errors
	authStore.clearError();

	// Validate passwords match
	if (password.value !== confirmPassword.value) {
		return;
	}

	try {
		await authStore.register(email.value, password.value);

		// Redirect to login page after successful registration
		router.push('/login');
	} catch (error) {
		// Error is handled by the store
		console.error('Registration failed:', error);
	}
};
</script>
