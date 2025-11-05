import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './style.css';
import App from './App.vue';
import router from './router';
import { initializeWidgets } from './composables/widgetRegistry';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);

// Initialize widget registry after Pinia is installed
console.log('[main] Initializing widget registry...');
initializeWidgets();
console.log('[main] Widget registry initialized');

app.mount('#app');
