/**
 * Widget Data API
 * Functions to fetch data from backend widget endpoints
 */

import apiClient from './client';
import type {
	ContractCryptoPrice,
	ContractGitHubEvent,
	ContractNewsArticle,
	ContractStatusCheck,
	ContractWeatherData,
} from '../contracts/apiContract';

/**
 * GitHub Widget Data
 */
export type GitHubEvent = ContractGitHubEvent;

export const fetchGitHubData = async (
	username: string,
): Promise<GitHubEvent[]> => {
	const response = await apiClient.get<GitHubEvent[]>('/data/github', {
		params: { username },
	});
	return response.data;
};

/**
 * Weather Widget Data
 */
export type WeatherData = ContractWeatherData;

export const fetchWeatherData = async (city: string): Promise<WeatherData> => {
	const response = await apiClient.get<WeatherData>('/data/weather', {
		params: { city },
	});
	return response.data;
};

/**
 * News Widget Data
 */
export type NewsArticle = ContractNewsArticle;

export const fetchNewsData = async (topic: string): Promise<NewsArticle[]> => {
	const response = await apiClient.get<NewsArticle[]>('/data/news', {
		params: { topic },
	});
	return response.data;
};

/**
 * Crypto Widget Data
 */
export type CryptoPrice = ContractCryptoPrice;

export const fetchCryptoData = async (ids: string): Promise<CryptoPrice[]> => {
	const response = await apiClient.get<CryptoPrice[]>('/data/crypto', {
		params: { ids },
	});
	return response.data;
};

/**
 * Status Widget Data
 */
export type StatusCheck = ContractStatusCheck;

export const fetchStatusData = async (urls: string): Promise<StatusCheck[]> => {
	const response = await apiClient.get<StatusCheck[]>('/data/status', {
		params: { urls },
	});
	return response.data;
};
