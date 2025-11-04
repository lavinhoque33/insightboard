/**
 * Axios client configuration with interceptors for authentication and error handling
 */

import axios from 'axios';
import type {
	AxiosError,
	AxiosInstance,
	InternalAxiosRequestConfig,
} from 'axios';
import type { ApiError } from '../types';

// Create axios instance with base configuration
const apiClient: AxiosInstance = axios.create({
	baseURL: import.meta.env.VITE_API_URL || '/api',
	timeout: 10000,
	headers: {
		'Content-Type': 'application/json',
	},
});

// Request interceptor - add JWT token to all requests
apiClient.interceptors.request.use(
	(config: InternalAxiosRequestConfig) => {
		const token = localStorage.getItem('auth_token');
		if (token && config.headers) {
			config.headers.Authorization = `Bearer ${token}`;
		}
		return config;
	},
	(error: AxiosError) => {
		return Promise.reject(error);
	},
);

// Response interceptor - handle errors globally
apiClient.interceptors.response.use(
	(response) => {
		return response;
	},
	(error: AxiosError<ApiError>) => {
		// Handle different error scenarios
		if (error.response) {
			// Some backends return `error` instead of `message`
			const data: any = error.response.data || {};
			const apiError: ApiError = {
				message: data.message || data.error || 'An error occurred',
				status: error.response.status,
				details: data.details,
			};

			// Handle 401 Unauthorized - careful with login/register endpoints
			if (error.response.status === 401) {
				const reqUrl = (error.config?.url || '').toString();
				const isAuthLogin = reqUrl.includes('/auth/login');
				const isAuthRegister = reqUrl.includes('/auth/register');

				// For login/register failures, let callers handle and show errors
				if (!isAuthLogin && !isAuthRegister) {
					localStorage.removeItem('auth_token');
					localStorage.removeItem('user');
					// Redirect to login will be handled by router guard or here as fallback
					if (typeof window !== 'undefined' && window.location.pathname !== '/login') {
						window.location.href = '/login';
					}
				}
			}

			return Promise.reject(apiError);
		} else if (error.request) {
			// Request made but no response
			const apiError: ApiError = {
				message:
					'No response from server. Please check your connection.',
				status: 0,
			};
			return Promise.reject(apiError);
		} else {
			// Something else happened
			const apiError: ApiError = {
				message: error.message || 'An unexpected error occurred',
				status: 0,
			};
			return Promise.reject(apiError);
		}
	},
);

export default apiClient;
