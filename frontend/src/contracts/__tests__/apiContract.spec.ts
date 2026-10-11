import Ajv2020 from 'ajv/dist/2020';
import contractJson from '../../../../contracts/api-contract.v1.json';
import { beforeAll, describe, expect, it } from 'vitest';
import type {
	ContractAuthResponse,
	ContractCreateDashboardRequest,
	ContractCryptoPrice,
	ContractDashboard,
	ContractErrorResponse,
	ContractGitHubEvent,
	ContractHealthResponse,
	ContractLoginRequest,
	ContractNewsArticle,
	ContractReadyResponse,
	ContractRegisterRequest,
	ContractStatusCheck,
	ContractUpdateDashboardRequest,
	ContractUser,
	ContractValidationErrorResponse,
	ContractWeatherData,
} from '../apiContract';

const schema = contractJson as { $id: string };

// Minimal validators for the `format` keywords the contract uses; strict Ajv
// rejects unknown formats, and a full formats package is not an approved dependency.
const FORMATS: Record<string, (value: string) => boolean> = {
	// local@domain with a dotted domain; not a full RFC 5322 grammar
	email: (v) => /^[^\s@]+@[^\s@.]+(\.[^\s@.]+)+$/.test(v),
	uuid: (v) =>
		/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(v),
	// RFC 3339 date-time with a mandatory offset, and a real calendar date
	'date-time': (v) =>
		/^\d{4}-\d{2}-\d{2}[Tt]\d{2}:\d{2}:\d{2}(\.\d+)?([Zz]|[+-]\d{2}:\d{2})$/.test(
			v,
		) && !Number.isNaN(Date.parse(v)),
	// absolute URI: must have a scheme and parse
	uri: (v) => /^[a-z][a-z0-9+.-]*:/i.test(v) && URL.canParse(v),
};

let ajv: Ajv2020;

function validate(def: string, payload: unknown): string | null {
	const check = ajv.getSchema(`${schema.$id}#/$defs/${def}`);
	if (!check) throw new Error(`unknown $defs entry ${def}`);
	return check(payload) ? null : ajv.errorsText(check.errors);
}

function expectValid(def: string, payload: unknown): void {
	expect(validate(def, payload), `${def} should validate`).toBeNull();
}

// Payloads annotated with the contract types: type drift fails vue-tsc,
// schema drift fails these assertions.
const user: ContractUser = {
	id: '6f1f3f0e-6a52-4a9e-8d57-0e3f4d7a9b11',
	email: 'user@example.com',
	created_at: '2026-10-10T10:00:00Z',
};

const dashboard: ContractDashboard = {
	id: '0b8f2c4e-5d7a-4a54-9b0e-2f9e8f6a1c33',
	user_id: user.id,
	name: 'Ops',
	layout_json: {
		widgets: [
			{
				id: 'w1',
				type: 'status',
				x: 0,
				y: 0,
				w: 4,
				h: 3,
				config: { urls: 'http://127.0.0.1:9999' },
			},
		],
	},
	settings_json: { theme: 'dark', title: 'Ops', refresh_interval: 60 },
	created_at: user.created_at,
	updated_at: user.created_at,
};

const authResponse: ContractAuthResponse = { token: 'a.b.c', user };
const registerRequest: ContractRegisterRequest = {
	email: 'user@example.com',
	password: 'correct-horse',
};
const loginRequest: ContractLoginRequest = registerRequest;
const createRequest: ContractCreateDashboardRequest = { name: 'Ops' };
const updateRequest: ContractUpdateDashboardRequest = {
	name: 'Renamed',
	layout_json: { widgets: [] },
	settings_json: { theme: 'light' },
};

const githubEvents: ContractGitHubEvent[] = [
	{
		id: '1',
		type: 'PushEvent',
		repo: { name: 'octocat/hello' },
		created_at: user.created_at,
		payload: { size: 1 },
	},
	{
		id: '2',
		type: 'WatchEvent',
		repo: { name: 'octocat/hello' },
		created_at: user.created_at,
		payload: null,
	},
];

const weather: ContractWeatherData = {
	city: 'Berlin',
	country: 'DE',
	temperature: 12.5,
	feels_like: 11.2,
	humidity: 70,
	description: 'light rain',
	wind_speed: 3.4,
	icon: '10d',
};

const news: ContractNewsArticle[] = [
	{
		title: 'Headline',
		description: 'Body',
		url: 'https://news.example.com/a',
		source: { name: 'Example News' },
		publishedAt: '2026-10-10T09:00:00Z',
		urlToImage: null,
	},
];

const crypto: ContractCryptoPrice[] = [
	{
		id: 'bitcoin',
		symbol: 'btc',
		name: 'Bitcoin',
		current_price: 65000.12,
		price_change_percentage_24h: -1.5,
		market_cap: 1.3e12,
		total_volume: 3.1e10,
	},
	{
		id: 'obscure',
		symbol: 'obs',
		name: 'Obscure',
		current_price: null,
		price_change_percentage_24h: null,
		market_cap: null,
		total_volume: null,
	},
];

const statusChecks: ContractStatusCheck[] = [
	{ url: 'http://127.0.0.1:9', status: 200, response_time_ms: 12, error: null },
	{ url: 'http://127.0.0.1:9/r', status: 302, response_time_ms: 5 },
	{
		url: 'http://10.0.0.1',
		status: 0,
		response_time_ms: 0,
		error: 'blocked_address',
	},
	{ url: 'ftp://x', status: 0, response_time_ms: 0, error: 'invalid_url' },
	{
		url: 'http://127.0.0.1:1',
		status: 0,
		response_time_ms: 0,
		error: 'timeout',
	},
	{
		url: 'http://127.0.0.1:2',
		status: 0,
		response_time_ms: 0,
		error: 'connect_error',
	},
];

const errorResponse: ContractErrorResponse = { error: 'Too many requests' };
const validationError: ContractValidationErrorResponse = {
	error: 'Validation failed',
	details: [
		{ field: 'email', message: 'must be a valid email address' },
		{ field: 'layout_json.widgets.0.w', message: 'must be between 1 and 12' },
	],
};
const health: ContractHealthResponse = {
	status: 'ok',
	service: 'insightboard-backend',
	version: '0.1.0',
};
const ready: ContractReadyResponse = { status: 'ready' };
const readyCacheDisabled: ContractReadyResponse = {
	status: 'ready',
	cache: 'disabled',
};

describe('api-contract.v1.json', () => {
	beforeAll(() => {
		ajv = new Ajv2020({ strict: true, allErrors: true });
		for (const [name, check] of Object.entries(FORMATS)) {
			ajv.addFormat(name, check);
		}
		ajv.addSchema(schema);
	});

	it('compiles', () => {
		expect(ajv.getSchema(schema.$id)).toBeDefined();
	});

	it('accepts typed auth payloads', () => {
		expectValid('RegisterRequest', registerRequest);
		expectValid('LoginRequest', loginRequest);
		expectValid('AuthResponse', authResponse);
		expectValid('User', user);
	});

	it('accepts typed dashboard payloads', () => {
		expectValid('Dashboard', dashboard);
		expectValid('CreateDashboardRequest', createRequest);
		expectValid('UpdateDashboardRequest', updateRequest);
	});

	it('accepts typed widget payloads', () => {
		for (const event of githubEvents) expectValid('GitHubEvent', event);
		expectValid('WeatherData', weather);
		for (const article of news) expectValid('NewsArticle', article);
		for (const coin of crypto) expectValid('CryptoPrice', coin);
		for (const check of statusChecks) expectValid('StatusCheck', check);
	});

	it('accepts typed error and health bodies', () => {
		expectValid('ErrorResponse', errorResponse);
		expectValid('ValidationErrorResponse', validationError);
		expectValid('HealthResponse', health);
		expectValid('ReadyResponse', ready);
		expectValid('ReadyResponse', readyCacheDisabled);
	});

	it('accepts a whole-contract document built from the fixtures', () => {
		const check = ajv.getSchema(schema.$id)!;
		const document = {
			auth: {
				register_request: registerRequest,
				login_request: loginRequest,
				auth_response: authResponse,
				current_user_response: user,
			},
			dashboards: {
				dashboard,
				list_response: [dashboard],
				create_request: createRequest,
				update_request: updateRequest,
			},
			widgets: {
				github: githubEvents,
				weather,
				news,
				crypto,
				status: statusChecks,
			},
			errors: {
				error_response: errorResponse,
				validation_error_response: validationError,
			},
			health: { healthz_response: health, readyz_response: ready },
		};
		expect(check(document), ajv.errorsText(check.errors)).toBe(true);
	});

	it('rejects deliberately broken fixtures', () => {
		// A 422 body without details, a misspelled status error class, a
		// non-numeric crypto price, an unknown error key, and a bad dashboard
		expect(
			validate('ValidationErrorResponse', { error: 'bad' }),
		).not.toBeNull();
		expect(
			validate('StatusCheck', {
				url: 'http://127.0.0.1:9',
				status: 0,
				response_time_ms: 0,
				error: 'Connection refused by 127.0.0.1',
			}),
		).not.toBeNull();
		expect(
			validate('CryptoPrice', { ...crypto[0], current_price: 'high' }),
		).not.toBeNull();
		expect(
			validate('ErrorResponse', { error: 'x', message: 'extra' }),
		).not.toBeNull();
		expect(
			validate('Dashboard', { ...dashboard, name: undefined }),
		).not.toBeNull();
	});
});
