/**
 * Vue Router configuration
 * Defines all application routes and navigation guards
 */

import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import LoginView from '../views/LoginView.vue';
import RegisterView from '../views/RegisterView.vue';
import DashboardListView from '../views/DashboardListView.vue';

const routes: RouteRecordRaw[] = [
	{
		path: '/',
		name: 'home',
		redirect: () => {
			const authStore = useAuthStore();
			return authStore.isAuthenticated ? '/dashboards' : '/login';
		},
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

const router = createRouter({
	history: createWebHistory(import.meta.env.BASE_URL),
	routes,
});

// Navigation guard for authentication
router.beforeEach((to, from, next) => {
	const authStore = useAuthStore();

	// Check if route requires authentication
	if (to.meta.requiresAuth && !authStore.isAuthenticated) {
		// Redirect to login, save intended destination
		next({
			name: 'login',
			query: { redirect: to.fullPath },
		});
		return;
	}

	// Check if route requires guest (not authenticated)
	if (to.meta.requiresGuest && authStore.isAuthenticated) {
		// Redirect to dashboards if already logged in
		next({ name: 'dashboards' });
		return;
	}

	next();
});

export default router;
