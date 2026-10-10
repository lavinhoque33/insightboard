/**
 * Vue Router configuration
 * Defines all application routes and navigation guards
 */

import { createRouter, createWebHistory } from 'vue-router';
import type { RouteRecordRaw, RouterHistory } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import LoginView from '../views/LoginView.vue';
import RegisterView from '../views/RegisterView.vue';
import DashboardListView from '../views/DashboardListView.vue';

const routes: RouteRecordRaw[] = [
	{
		path: '/',
		name: 'home',
		redirect: () => (useAuthStore().isAuthenticated ? '/dashboards' : '/login'),
	},
	{
		path: '/login',
		name: 'login',
		component: LoginView,
		meta: { requiresGuest: true },
	},
	{
		path: '/register',
		name: 'register',
		component: RegisterView,
		meta: { requiresGuest: true },
	},
	{
		// A route rather than a click handler: route-leave guards run before
		// `beforeEnter`, so the editor saves pending changes before the session
		// ends. The component is never rendered.
		path: '/logout',
		name: 'logout',
		component: LoginView,
		beforeEnter: () => {
			useAuthStore().logout();
			return { name: 'login' };
		},
	},
	{
		path: '/dashboards',
		name: 'dashboards',
		component: DashboardListView,
		meta: { requiresAuth: true },
	},
	{
		path: '/dashboards/:id',
		name: 'dashboard-editor',
		component: () => import('../views/DashboardEditorView.vue'),
		meta: { requiresAuth: true },
	},
	{
		path: '/:pathMatch(.*)*',
		name: 'not-found',
		redirect: '/',
	},
];

/**
 * The post-login destination from a `redirect` query value: only same-origin
 * paths are accepted, so the login page cannot send users elsewhere.
 */
export function safeRedirect(value: unknown): string {
	return typeof value === 'string' &&
		value.startsWith('/') &&
		!value.startsWith('//') &&
		!value.startsWith('/\\')
		? value
		: '/dashboards';
}

export function createAppRouter(history: RouterHistory) {
	const router = createRouter({ history, routes });

	router.beforeEach((to) => {
		const authStore = useAuthStore();
		// Expired tokens end the session here, before any request is made
		const signedIn = authStore.checkSession();

		if (to.meta.requiresAuth && !signedIn) {
			return { name: 'login', query: { redirect: to.fullPath } };
		}
		if (to.meta.requiresGuest && signedIn) {
			return { name: 'dashboards' };
		}
		return true;
	});

	return router;
}

export default createAppRouter(createWebHistory(import.meta.env.BASE_URL));
