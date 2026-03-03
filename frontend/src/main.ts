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
initializeWidgets();

app.mount('#app');
