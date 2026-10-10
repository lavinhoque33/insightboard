import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// The /api proxy target: the dev backend by default; the Playwright harness points it at
// its own loopback backend (API_PROXY_TARGET=http://127.0.0.1:8090).
const apiProxyTarget = process.env.API_PROXY_TARGET || 'http://127.0.0.1:8080';

// https://vite.dev/config/
export default defineConfig({
	plugins: [vue()],
	server: {
		host: '127.0.0.1',
		proxy: {
			'/api': {
				target: apiProxyTarget,
				changeOrigin: true,
			},
		},
	},
});
