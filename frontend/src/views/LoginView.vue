<template>
	<div
		class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8"
	>
		<div class="max-w-md w-full space-y-8">
			<!-- Header -->
			<div>
				<h2
					class="mt-6 text-center text-3xl font-extrabold text-gray-900"
				>
					Sign in to InsightBoard
				</h2>
				<p class="mt-2 text-center text-sm text-gray-600">
					Or
					<router-link
						to="/register"
						class="font-medium text-primary-600 hover:text-primary-500"
					>
						create a new account
					</router-link>
				</p>
			</div>

			<!-- Login Form -->
			<form class="mt-8 space-y-6" @submit.prevent="handleSubmit">
				<div class="rounded-md shadow-sm -space-y-px">
					<div>
						<label for="email" class="sr-only">Email address</label>
						<input
							id="email"
							v-model="email"
							name="email"
							type="email"
							autocomplete="email"
							required
							class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-primary-500 focus:border-primary-500 focus:z-10 sm:text-sm"
							placeholder="Email address"
						/>
					</div>
					<div>
						<label for="password" class="sr-only">Password</label>
						<input
							id="password"
							v-model="password"
							name="password"
							type="password"
							autocomplete="current-password"
							required
							class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-primary-500 focus:border-primary-500 focus:z-10 sm:text-sm"
							placeholder="Password"
						/>
					</div>
				</div>

				<!-- Error Message -->
				<div v-if="authStore.error" class="rounded-md bg-red-50 p-4">
					<div class="flex">
						<div class="ml-3">
							<h3 class="text-sm font-medium text-red-800">
								{{ authStore.error }}
							</h3>
						</div>
					</div>
				</div>

				<!-- Submit Button -->
				<div>
					<button
						type="submit"
						:disabled="authStore.loading"
						class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed"
					>
						<span v-if="!authStore.loading">Sign in</span>
						<span v-else>Signing in...</span>
					</button>
				</div>
			</form>
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
