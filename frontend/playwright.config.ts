import { tmpdir } from 'node:os';
import { defineConfig, devices } from '@playwright/test';

// Hermetic, loopback-only journey harness (M4 contract sections 9 and 14). Every listener
// binds to 127.0.0.1 on ports distinct from the dev services (backend 8080, Vite 5173):
//   provider fixture 18090, status fixture 18091, backend 8090, Vite 5175.
// The backend gets its provider base URLs pointed at the fixture and dummy keys, so no
// request can reach GitHub, OpenWeather, NewsAPI, or CoinGecko. Postgres and Redis come
// from DATABASE_URL and REDIS_URL (locally `make e2e` loads them from .env; in CI from the
// workflow's service containers).

const HOST = '127.0.0.1';
const PROVIDER_PORT = 18090;
const STATUS_PORT = 18091;
const BACKEND_PORT = 8090;
const WEB_PORT = 5175;

const providerBase = `http://${HOST}:${PROVIDER_PORT}`;
const backendBase = `http://${HOST}:${BACKEND_PORT}`;
const webBase = `http://${HOST}:${WEB_PORT}`;

/** Dummy credentials the fixture accepts; they override any real key in the environment. */
const dummyKeys = {
	E2E_GITHUB_TOKEN: 'e2e-dummy-github-token',
	E2E_OPENWEATHER_KEY: 'e2e-dummy-openweather-key',
	E2E_NEWSAPI_KEY: 'e2e-dummy-newsapi-key',
};

function required(key: 'DATABASE_URL' | 'REDIS_URL'): string {
	const value = process.env[key];
	if (!value) {
		throw new Error(
			`${key} must be set for the e2e backend (make e2e loads it from .env)`,
		);
	}
	return value;
}

const isGitHubActions = process.env.GITHUB_ACTIONS === 'true';

export default defineConfig({
	testDir: './e2e',
	// *.e2e.ts, not *.spec.ts, so Vitest's default include never picks the journeys up
	testMatch: '**/*.e2e.ts',
	outputDir: './e2e/test-results',
	fullyParallel: false,
	workers: 1,
	forbidOnly: isGitHubActions,
	retries: isGitHubActions ? 1 : 0,
	timeout: 60_000,
	expect: { timeout: 10_000 },
	reporter: [
		['list'],
		['html', { outputFolder: './e2e/playwright-report', open: 'never' }],
	],
	use: {
		baseURL: webBase,
		trace: 'retain-on-failure',
		screenshot: 'only-on-failure',
	},
	projects: [
		{
			name: 'chromium',
			use: {
				...devices['Desktop Chrome'],
				launchOptions: {
					// The pages only load 127.0.0.1: fail every other host lookup so the
					// browser cannot reach anything else, even if a page tried.
					args: ['--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1'],
				},
			},
		},
	],
	webServer: [
		{
			name: 'fixtures',
			command: 'node e2e/fixtures/server.mjs',
			url: `${providerBase}/healthz`,
			env: {
				...dummyKeys,
				E2E_PROVIDER_PORT: String(PROVIDER_PORT),
				E2E_STATUS_PORT: String(STATUS_PORT),
			},
			reuseExistingServer: false,
			timeout: 15_000,
		},
		{
			name: 'backend',
			// Built beforehand by `make e2e` (cargo build --locked). Run from the temp
			// directory so dotenvy finds no .env: the environment below is the whole config.
			command: `"${process.cwd()}/../backend/target/debug/insightboard-backend"`,
			cwd: tmpdir(),
			url: `${backendBase}/readyz`,
			env: {
				APP_HOST: HOST,
				APP_PORT: String(BACKEND_PORT),
				DATABASE_URL: required('DATABASE_URL'),
				REDIS_URL: required('REDIS_URL'),
				// Throwaway, e2e-only signing key (not a secret; tokens die with the run)
				JWT_SECRET: 'e2e-only-throwaway-jwt-secret-0123456789abcdef0123456789',
				CORS_ORIGINS: webBase,
				GITHUB_API_TOKEN: dummyKeys.E2E_GITHUB_TOKEN,
				OPENWEATHER_API_KEY: dummyKeys.E2E_OPENWEATHER_KEY,
				NEWSAPI_API_KEY: dummyKeys.E2E_NEWSAPI_KEY,
				GITHUB_API_BASE_URL: providerBase,
				OPENWEATHER_API_BASE_URL: providerBase,
				NEWSAPI_API_BASE_URL: providerBase,
				COINGECKO_API_BASE_URL: providerBase,
				STATUS_ALLOWED_FIXTURES: `${HOST}:${STATUS_PORT}`,
				PROVIDER_CONNECT_TIMEOUT_MS: '2000',
				PROVIDER_TIMEOUT_MS: '4000',
				STATUS_CONNECT_TIMEOUT_MS: '2000',
				STATUS_TIMEOUT_MS: '4000',
				RUST_LOG: 'info',
			},
			reuseExistingServer: false,
			timeout: 60_000,
		},
		{
			name: 'web',
			command: `npx vite --port ${WEB_PORT} --strictPort`,
			url: webBase,
			env: {
				API_PROXY_TARGET: backendBase,
				// The client must use the same-origin /api proxy, never a direct URL
				VITE_API_URL: '',
			},
			reuseExistingServer: false,
			timeout: 60_000,
		},
	],
});
