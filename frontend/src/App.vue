<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useAuthStore } from './stores/auth';

const router = useRouter();
const authStore = useAuthStore();

const handleLogout = () => {
	authStore.logout();
	router.push('/login');
};
</script>

<template>
	<div class="min-h-screen bg-gray-50">
		<!-- Navigation Header -->
		<nav class="bg-white shadow-sm">
			<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
				<div class="flex justify-between h-16">
					<div class="flex items-center">
						<router-link
							to="/"
							class="flex items-center text-xl font-bold text-gray-900"
						>
							<svg
								class="h-8 w-8 text-primary-600 mr-2"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
								/>
							</svg>
							InsightBoard
						</router-link>
					</div>

					<!-- Navigation Links -->
					<div class="flex items-center space-x-4">
						<template v-if="authStore.isAuthenticated">
							<router-link
								to="/dashboards"
								class="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
							>
								Dashboards
							</router-link>
							<span class="text-gray-500 text-sm">
								{{ authStore.user?.email }}
							</span>
							<button
								@click="handleLogout"
								class="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
							>
								Logout
							</button>
						</template>
						<template v-else>
							<router-link
								to="/login"
								class="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
							>
								Login
							</router-link>
							<router-link
								to="/register"
								class="bg-primary-600 text-white hover:bg-primary-700 px-4 py-2 rounded-md text-sm font-medium"
							>
								Sign Up
							</router-link>
						</template>
					</div>
				</div>
			</div>
		</nav>

		<!-- Main Content -->
		<router-view />
	</div>
</template>

<style>
/* Global styles can be added here */
</style>
