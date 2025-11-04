<script setup lang="ts">
import { computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuthStore } from './stores/auth';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();

const handleLogout = () => {
	authStore.logout();
	router.push('/login');
};

// Dynamic classes for auth buttons (daisyUI)
const loginBtnClass = computed(() =>
    `btn btn-sm gap-2 transition-colors duration-200 ${route.path.startsWith('/login') ? 'btn-primary' : 'btn-ghost'}`,
);
const registerBtnClass = computed(() =>
    `btn btn-sm gap-2 transition-colors duration-200 ${route.path.startsWith('/register') ? 'btn-primary' : 'btn-ghost'}`,
);
</script>

<template>
	<div class="min-h-screen bg-base-100">
		<!-- daisyUI Navbar -->
		<nav class="navbar bg-base-200 shadow-lg sticky top-0 z-50">
			<div class="flex-1">
				<router-link
					to="/"
					class="btn btn-ghost normal-case text-2xl font-bold text-transparent bg-gradient-to-r from-primary to-secondary bg-clip-text"
				>
					📊 InsightBoard
				</router-link>
			</div>

			<!-- Navigation Items -->
			<div class="flex-none gap-2">
				<template v-if="authStore.isAuthenticated">
					<!-- Dashboards Link -->
					<router-link
						to="/dashboards"
						class="btn btn-ghost btn-sm gap-2"
						active-class="btn-primary"
					>
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
								d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
							/>
						</svg>
						Dashboards
					</router-link>

					<!-- User Menu Dropdown -->
					<div class="dropdown dropdown-end">
						<div
							tabindex="0"
							class="btn btn-ghost btn-circle avatar placeholder"
						>
							<div
								class="bg-gradient-to-br from-primary to-secondary text-white w-10 rounded-full flex items-center justify-center"
							>
								<span class="text-lg font-bold">
									{{
										authStore.user?.email
											?.charAt(0)
											.toUpperCase()
									}}
								</span>
							</div>
						</div>
						<ul
							tabindex="0"
							class="dropdown-content z-[1] menu p-2 shadow-lg bg-base-100 rounded-box w-52 border border-base-300"
						>
							<li class="menu-title">
								<span>{{ authStore.user?.email }}</span>
							</li>
							<li>
								<a @click="handleLogout" class="text-error">
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
											d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
										/>
									</svg>
									Logout
								</a>
							</li>
						</ul>
					</div>
				</template>

				<!-- Authentication Links -->
				<template v-else>
					<router-link to="/login" :class="loginBtnClass">
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
						Login
					</router-link>
					<router-link to="/register" :class="registerBtnClass">
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
						Sign Up
					</router-link>
				</template>
			</div>
		</nav>

		<!-- Main Content -->
		<main class="min-h-[calc(100vh-70px)]">
			<router-view />
		</main>

		<!-- Footer -->
		<footer
			class="footer footer-center bg-base-200 text-base-content border-t border-base-300 mt-16"
		>
			<aside>
				<p class="text-sm">
					© 2025 InsightBoard. Built with
					<span class="text-error">♥</span> using Rust, Vue 3 &
					daisyUI.
				</p>
			</aside>
		</footer>
	</div>
</template>

<style scoped>
:deep(.navbar) {
	background: linear-gradient(135deg, hsl(var(--b2)) 0%, hsl(var(--b1)) 100%);
}
</style>
