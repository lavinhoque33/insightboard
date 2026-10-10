// Hermetic fixtures for the Playwright journeys (no dependencies; node:http only).
//
// Provider listener (E2E_PROVIDER_PORT, default 18090): imitates the GitHub, OpenWeather,
// NewsAPI, and CoinGecko endpoints the backend calls, so the backend's *_API_BASE_URL
// variables can point here. Each endpoint answers only the dummy credential the e2e
// backend is started with; anything else gets 401, so a real key that slipped through
// would make the journey fail instead of passing silently.
//
// Status listener (E2E_STATUS_PORT, default 18091): the loopback target the backend's
// STATUS_ALLOWED_FIXTURES allowlist names; every GET answers 200.
//
// Both listeners bind to 127.0.0.1 only. Run: node e2e/fixtures/server.mjs

import { createServer } from 'node:http';

const HOST = '127.0.0.1';
const PROVIDER_PORT = Number(process.env.E2E_PROVIDER_PORT ?? 18090);
const STATUS_PORT = Number(process.env.E2E_STATUS_PORT ?? 18091);

/** Dummy credentials, passed in by playwright.config.ts; never real keys. */
const DUMMY_KEYS = {
	github: process.env.E2E_GITHUB_TOKEN,
	openweather: process.env.E2E_OPENWEATHER_KEY,
	newsapi: process.env.E2E_NEWSAPI_KEY,
};
if (Object.values(DUMMY_KEYS).some((key) => !key)) {
	console.error('e2e fixtures: E2E_*_KEY and E2E_GITHUB_TOKEN must be set');
	process.exit(1);
}

function send(res, status, body) {
	const text = JSON.stringify(body);
	res.writeHead(status, {
		'content-type': 'application/json',
		'content-length': Buffer.byteLength(text),
	});
	res.end(text);
}

const unauthorized = (res) => send(res, 401, { message: 'Bad credentials' });

function weather(url, res) {
	if (url.searchParams.get('appid') !== DUMMY_KEYS.openweather) {
		return unauthorized(res);
	}
	const city = url.searchParams.get('q') ?? '';
	send(res, 200, {
		name: city,
		sys: { country: 'FX' },
		main: { temp: 21.4, feels_like: 20.6, humidity: 42 },
		weather: [{ description: 'fixture clear sky', icon: '01d' }],
		wind: { speed: 3.5 },
	});
}

function crypto(url, res) {
	const ids = (url.searchParams.get('ids') ?? '').split(',').filter(Boolean);
	send(
		res,
		200,
		ids.map((id, index) => ({
			id,
			symbol: id.slice(0, 3),
			name: `Fixture ${id}`,
			current_price: 100 + index,
			price_change_percentage_24h: 1.5,
			market_cap: 1_000_000,
			total_volume: 50_000,
		})),
	);
}

function github(req, user, res) {
	if (req.headers.authorization !== `Bearer ${DUMMY_KEYS.github}`) {
		return unauthorized(res);
	}
	send(res, 200, [
		{
			id: '1',
			type: 'PushEvent',
			repo: { name: `${user}/fixture-repo` },
			created_at: '2026-10-10T00:00:00Z',
			payload: {},
		},
	]);
}

function news(req, res) {
	if (req.headers['x-api-key'] !== DUMMY_KEYS.newsapi) {
		return unauthorized(res);
	}
	send(res, 200, {
		status: 'ok',
		articles: [
			{
				title: 'Fixture headline',
				description: 'A fixture article',
				url: 'https://example.com/fixture-article',
				source: { name: 'Fixture News' },
				publishedAt: '2026-10-10T00:00:00Z',
				urlToImage: null,
			},
		],
	});
}

const provider = createServer((req, res) => {
	const url = new URL(req.url ?? '/', `http://${HOST}`);
	if (req.method !== 'GET') return send(res, 405, { message: 'GET only' });
	if (url.pathname === '/healthz') return send(res, 200, { ok: true });
	if (url.pathname === '/data/2.5/weather') return weather(url, res);
	if (url.pathname === '/api/v3/coins/markets') return crypto(url, res);
	if (url.pathname === '/v2/everything') return news(req, res);
	const events = /^\/users\/([^/]+)\/events\/public$/.exec(url.pathname);
	if (events) return github(req, decodeURIComponent(events[1]), res);
	send(res, 404, { message: 'Not Found' });
});

const status = createServer((req, res) => {
	res.writeHead(200, { 'content-type': 'text/plain' });
	res.end('ok');
});

provider.listen(PROVIDER_PORT, HOST, () =>
	console.log(`e2e provider fixture on http://${HOST}:${PROVIDER_PORT}`),
);
status.listen(STATUS_PORT, HOST, () =>
	console.log(`e2e status fixture on http://${HOST}:${STATUS_PORT}`),
);

for (const signal of ['SIGINT', 'SIGTERM']) {
	process.on(signal, () => {
		provider.close();
		status.close();
		process.exit(0);
	});
}
