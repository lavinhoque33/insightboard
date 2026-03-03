/**
 * InsightBoard API contract v1.
 * Source of truth: /contracts/api-contract.v1.json
 */

export interface ContractUser {
	id: string;
	email: string;
	created_at: string;
}

export interface ContractAuthResponse {
	token: string;
	user: ContractUser;
}

export interface ContractDashboard {
	id: string;
	user_id: string;
	name: string;
	layout_json: {
		widgets: Array<{
			id: string;
			type: 'weather' | 'crypto' | 'github' | 'news' | 'status';
			x: number;
			y: number;
			w: number;
			h: number;
			config?: Record<string, unknown>;
		}>;
	};
	settings_json: {
		theme?: 'light' | 'dark';
		title?: string;
		refresh_interval?: number;
		[key: string]: unknown;
	};
	created_at: string;
	updated_at: string;
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

export interface ContractCryptoPrice {
	id: string;
	symbol: string;
	name: string;
	current_price: number;
	price_change_percentage_24h: number;
	market_cap: number;
	total_volume: number;
}

export interface ContractStatusCheck {
	url: string;
	status: number;
	response_time_ms: number;
	error?: string | null;
}
