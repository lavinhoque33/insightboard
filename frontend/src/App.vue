<script setup lang="ts">
import { useRoute } from 'vue-router';
import { useAuthStore } from './stores/auth';
import { useDisclosure } from './composables/useDisclosure';
import AppIcon from './components/ui/AppIcon.vue';
import BrandMark from './components/ui/BrandMark.vue';
import ThemeSwitcher from './components/ThemeSwitcher.vue';

const route = useRoute();
const authStore = useAuthStore();

const userMenu = useDisclosure();
const { root: userMenuRoot, trigger: userMenuTrigger } = userMenu;
</script>

<template>
	<div class="relative isolate flex min-h-screen flex-col text-base-content">
		<div class="app-aurora" aria-hidden="true"></div>

		<nav
			class="sticky top-0 z-50 border-b border-base-content/10 bg-base-100/70 backdrop-blur-xl backdrop-saturate-150"
			aria-label="Main"
		>
			<div class="flex h-14 items-center gap-2 px-3 sm:px-6">
				<div class="min-w-0 flex-1">
					<router-link
						to="/"
						class="inline-flex items-center gap-2.5 rounded-lg px-1 py-1 text-base font-semibold tracking-tight transition-opacity hover:opacity-80"
					>
						<BrandMark class="size-7" />
						<span :class="{ 'max-[419px]:sr-only': !authStore.isAuthenticated }"
							>InsightBoard</span
						>
					</router-link>
				</div>

				<div class="flex flex-none items-center gap-1.5 sm:gap-2">
					<template v-if="authStore.isAuthenticated">
						<router-link
							to="/dashboards"
							class="btn btn-ghost btn-sm gap-2 font-medium"
							active-class="bg-base-content/8"
						>
							<AppIcon name="dashboards" class="size-4" />
							<span class="sr-only sm:not-sr-only">Dashboards</span>
						</router-link>

						<ThemeSwitcher />

						<div ref="userMenuRoot" class="relative">
							<button
								ref="userMenuTrigger"
								type="button"
								class="bg-brand grid size-8 place-items-center rounded-full text-sm font-semibold text-white shadow-sm ring-2 ring-base-100 transition-transform duration-150 hover:scale-105 active:scale-95"
								aria-label="Account menu"
								aria-controls="user-menu"
								:aria-expanded="userMenu.open.value"
								@click="userMenu.toggle"
							>
								<span aria-hidden="true">
									{{ authStore.user?.email?.charAt(0).toUpperCase() }}
								</span>
							</button>
							<ul
								v-if="userMenu.open.value"
								id="user-menu"
								data-menu-panel
								class="menu absolute right-0 top-full z-50 mt-2 w-64 max-w-[calc(100vw-1rem)] origin-top-right animate-pop rounded-box border border-base-300 bg-base-100 p-1.5 shadow-float"
							>
								<li class="menu-title flex flex-col gap-0.5 px-3 pb-2 pt-1.5">
									<span class="text-xs font-normal text-base-content/70"
										>Signed in as</span
									>
									<span
										class="break-all text-sm font-medium text-base-content"
										>{{ authStore.user?.email }}</span
									>
								</li>
								<li>
									<router-link
										to="/logout"
										class="text-error"
										@click="userMenu.close()"
									>
										<AppIcon name="logout" class="size-4" />
										Logout
									</router-link>
								</li>
							</ul>
						</div>
					</template>

					<template v-else>
						<ThemeSwitcher />
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
			</div>
		</nav>

		<main class="flex flex-1 flex-col">
			<router-view />
		</main>

		<footer class="border-t border-base-content/10 px-4 py-5 sm:px-6">
			<div
				class="flex flex-wrap items-center justify-between gap-2 text-xs text-base-content/70"
			>
				<p class="flex items-center gap-2">
					<BrandMark class="size-4" />
					<span>InsightBoard</span>
				</p>
				<p>Built with Rust, Vue 3 and daisyUI</p>
			</div>
		</footer>
	</div>
</template>
