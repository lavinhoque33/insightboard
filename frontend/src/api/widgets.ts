/**
 * Widget Data API
 * Functions to fetch data from backend widget endpoints
 */

import apiClient from './client';

/**
 * GitHub Widget Data
 */
export interface GitHubEvent {
	type: string;
	repo: { name: string };
	created_at: string;
	payload?: any;
}

export const fetchGitHubData = async (
	username: string,
): Promise<GitHubEvent[]> => {
	const response = await apiClient.get('/data/github', {
		params: { username },
	});
	return response.data;
};

/**
 * Weather Widget Data
 */
export interface WeatherData {
	city: string;
	country: string;
	temperature: number;
	feels_like: number;
	description: string;
	humidity: number;
	wind_speed: number;
	icon: string;
}

export const fetchWeatherData = async (city: string): Promise<WeatherData> => {
	const response = await apiClient.get('/data/weather', {
		params: { city },
	});
	return response.data;
};

/**
 * News Widget Data
 */
export interface NewsArticle {
	title: string;
	description: string;
	url: string;
	source: { name: string };
	publishedAt: string;
	urlToImage?: string;
}

export const fetchNewsData = async (topic: string): Promise<NewsArticle[]> => {
	const response = await apiClient.get('/data/news', {
		params: { topic },
	});
	return response.data;
};

/**
 * Crypto Widget Data
 */
export interface CryptoPrice {
	id: string;
	symbol: string;
	name: string;
	current_price: number;
	price_change_percentage_24h: number;
	market_cap: number;
	total_volume: number;
}

export const fetchCryptoData = async (ids: string): Promise<CryptoPrice[]> => {
	const response = await apiClient.get('/data/crypto', {
		params: { ids },
	});
	return response.data;
};

/**
 * Status Widget Data
 */
export interface StatusCheck {
	url: string;
	status: number;
	response_time_ms: number;
	error?: string;
}

export const fetchStatusData = async (urls: string): Promise<StatusCheck[]> => {
	const response = await apiClient.get('/data/status', {
		params: { urls },
	});
	return response.data;
};
