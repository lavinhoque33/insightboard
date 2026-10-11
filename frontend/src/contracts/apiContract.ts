/**
 * InsightBoard API contract v1.
 * Source of truth: /contracts/api-contract.v1.json
 */

/** Body of every non-2xx response except 422 (400, 401, 404, 405, 409, 413, 415, 429, 500, 502, 503, 504). */
export interface ContractErrorResponse {
	error: string;
}

/** `field` is the request field, dotted for nested JSON (`layout_json.widgets.0.w`), or the query parameter name. */
export interface ContractValidationDetail {
	field: string;
	message: string;
}

/** 422 body: well-formed input that is missing, mistyped, or invalid. */
export interface ContractValidationErrorResponse {
	error: string;
	details: ContractValidationDetail[];
}

/** GET /healthz (dependency-free liveness). */
export interface ContractHealthResponse {
	status: 'ok';
	service: string;
	version: string;
}

/**
 * GET /readyz 200 body; 503 with ContractErrorResponse when Postgres or Redis is down. `cache` is
 * present only when the cache is disabled (the Lambda entry point without REDIS_URL), where only
 * Postgres is checked.
 */
export interface ContractReadyResponse {
	status: 'ready';
	cache?: 'disabled';
}

export interface ContractRegisterRequest {
	email: string;
	password: string;
}

export interface ContractLoginRequest {
	email: string;
	password: string;
}

export interface ContractUser {
	id: string;
	email: string;
	created_at: string;
}

export interface ContractAuthResponse {
	token: string;
	user: ContractUser;
}

export type ContractWidgetType =
	'weather' | 'crypto' | 'github' | 'news' | 'status';

export interface ContractWidgetLayout {
	id: string;
	type: ContractWidgetType;
	x: number;
	y: number;
	w: number;
	h: number;
	config?: Record<string, unknown>;
}

export interface ContractLayoutConfig {
	widgets: ContractWidgetLayout[];
}

export interface ContractDashboardSettings {
	theme?: 'light' | 'dark';
	title?: string;
	refresh_interval?: number;
	[key: string]: unknown;
}

export interface ContractDashboard {
	id: string;
	user_id: string;
	name: string;
	layout_json: ContractLayoutConfig;
	settings_json: ContractDashboardSettings;
	created_at: string;
	updated_at: string;
}

/** Omitted layout_json/settings_json are stored as `{"widgets":[]}` and `{}`. */
export interface ContractCreateDashboardRequest {
	name: string;
	layout_json?: ContractLayoutConfig;
	settings_json?: ContractDashboardSettings;
}

export interface ContractUpdateDashboardRequest {
	name?: string;
	layout_json?: ContractLayoutConfig;
	settings_json?: ContractDashboardSettings;
}

export interface ContractGitHubEvent {
	id: string;
	type: string;
	repo: { name: string };
	created_at: string;
	payload?: Record<string, unknown> | null;
}

export interface ContractWeatherData {
	city: string;
	country: string;
	temperature: number;
	feels_like: number;
	humidity: number;
	description: string;
	wind_speed: number;
	icon: string;
}

export interface ContractNewsArticle {
	title: string;
	description: string;
	url: string;
	source: { name: string };
	publishedAt: string;
	urlToImage?: string | null;
}

/** A numeric field that is null upstream stays null. */
export interface ContractCryptoPrice {
	id: string;
	symbol: string;
	name: string;
	current_price: number | null;
	price_change_percentage_24h: number | null;
	market_cap: number | null;
	total_volume: number | null;
}

export type ContractStatusErrorClass =
	| 'invalid_url'
	| 'blocked_address'
	| 'dns_error'
	| 'timeout'
	| 'connect_error'
	| 'request_error';

/**
 * One result per requested URL, in request order. A 3xx is reported as that status (redirects are not
 * followed). On failure `status` and `response_time_ms` are 0 and `error` is a generic class.
 */
export interface ContractStatusCheck {
	url: string;
	status: number;
	response_time_ms: number;
	error?: ContractStatusErrorClass | null;
}
