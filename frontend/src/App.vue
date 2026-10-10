<script setup lang="ts">
import { useRoute } from 'vue-router';
import { useAuthStore } from './stores/auth';
import { useDisclosure } from './composables/useDisclosure';

const route = useRoute();
const authStore = useAuthStore();

const userMenu = useDisclosure();
const { root: userMenuRoot, trigger: userMenuTrigger } = userMenu;
</script>

<template>
	<div class="flex min-h-screen flex-col bg-base-100">
		<nav
			class="navbar sticky top-0 z-50 gap-2 border-b border-base-300 bg-base-200 px-2 shadow-sm sm:px-4"
			aria-label="Main"
		>
			<div class="min-w-0 flex-1">
				<router-link
					to="/"
					class="btn btn-ghost px-2 text-lg font-bold sm:text-2xl"
				>
					<span aria-hidden="true">📊</span> InsightBoard
				</router-link>
			</div>

			<div class="flex flex-none items-center gap-1 sm:gap-2">
				<template v-if="authStore.isAuthenticated">
					<router-link
						to="/dashboards"
						class="btn btn-ghost btn-sm"
						active-class="btn-active"
					>
						<svg
							aria-hidden="true"
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
						<span class="sr-only sm:not-sr-only">Dashboards</span>
					</router-link>

					<div ref="userMenuRoot" class="relative">
						<button
							ref="userMenuTrigger"
							type="button"
							class="btn btn-circle btn-primary"
							aria-label="Account menu"
							aria-controls="user-menu"
							:aria-expanded="userMenu.open.value"
							@click="userMenu.toggle"
						>
							<span class="text-lg font-bold" aria-hidden="true">
								{{ authStore.user?.email?.charAt(0).toUpperCase() }}
							</span>
						</button>
						<ul
							v-if="userMenu.open.value"
							id="user-menu"
							data-menu-panel
							class="menu absolute right-0 top-full z-50 mt-2 w-64 max-w-[calc(100vw-1rem)] rounded-box border border-base-300 bg-base-100 p-2 shadow-lg"
						>
							<li class="menu-title">
								<span class="break-all">{{ authStore.user?.email }}</span>
							</li>
							<li>
								<router-link
									to="/logout"
									class="text-error"
									@click="userMenu.close()"
								>
									Logout
								</router-link>
							</li>
						</ul>
					</div>
				</template>

				<template v-else>
					<router-link
						to="/login"
						class="btn btn-sm"
						:class="route.name === 'login' ? 'btn-primary' : 'btn-ghost'"
					>
						Login
					</router-link>
					<router-link
						to="/register"
						class="btn btn-sm"
						:class="route.name === 'register' ? 'btn-primary' : 'btn-ghost'"
					>
						Sign Up
					</router-link>
				</template>
			</div>
		</nav>

		<main class="flex flex-1 flex-col">
			<router-view />
		</main>

		<footer
			class="footer footer-center border-t border-base-300 bg-base-200 p-4 text-base-content"
		>
			<aside>
				<p class="text-sm">
					© 2025 InsightBoard. Built with
					<span class="text-error" aria-label="love">♥</span> using Rust, Vue 3
					&amp; daisyUI.
				</p>
			</aside>
		</footer>
	</div>
</template>
