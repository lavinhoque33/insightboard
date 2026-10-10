import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './style.css';
import App from './App.vue';
import router from './router';
import { initializeWidgets } from './composables/widgetRegistry';
import { initTheme } from './composables/useTheme';
import { onUnauthorized } from './api/client';
import { useAuthStore } from './stores/auth';

// index.html already applied the theme; this keeps it in sync from now on
initTheme();

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);

// Initialize widget registry after Pinia is installed
initializeWidgets();

const authStore = useAuthStore();

// A rejected or expired session ends here: clear every store and return to
// login, keeping the page the user was on as the destination.
onUnauthorized(() => {
	authStore.logout();
	const current = router.currentRoute.value;
	if (current.meta.requiresAuth) {
		void router.push({
			name: 'login',
			query: { redirect: current.fullPath, expired: '1' },
		});
	}
});

// Confirm a restored session with the server
void authStore.fetchCurrentUser();

app.mount('#app');
