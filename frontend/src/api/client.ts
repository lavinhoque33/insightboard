/**
 * Axios client configuration with interceptors for authentication and error handling
 */

import axios from 'axios';
import type { AxiosInstance, InternalAxiosRequestConfig } from 'axios';
import type { ApiError, FieldError } from '../types';
import { TOKEN_KEY, isTokenExpired } from './token';

// Error body every backend response uses: `{error}`, plus `details` on 422
interface ErrorBody {
	error?: string;
	details?: FieldError[];
}

// Create axios instance with base configuration
const apiClient: AxiosInstance = axios.create({
	baseURL: import.meta.env.VITE_API_URL || '/api',
	timeout: 10000,
	headers: {
		'Content-Type': 'application/json',
	},
});

const SESSION_EXPIRED = 'Your session has expired. Please sign in again.';

// Login and register answer 401 for bad credentials; that is a form error, not
// an ended session, and they need no token.
const isAuthEndpoint = (url: string | undefined): boolean =>
	/\/auth\/(login|register)$/.test(url ?? '');

let unauthorizedHandler: (() => void) | null = null;

/**
 * Register what happens when the session ends (a 401 from a protected endpoint,
 * or a stored token past its `exp`). The app clears the session and sends the
 * user to login with the current route as the redirect target.
 */
export function onUnauthorized(handler: (() => void) | null): void {
	unauthorizedHandler = handler;
}

// Request interceptor - add the JWT to protected requests; never send a token
// the client already knows is expired.
apiClient.interceptors.request.use((config: InternalAxiosRequestConfig) => {
	if (isAuthEndpoint(config.url)) return config;
	const token = localStorage.getItem(TOKEN_KEY);
	if (!token) return config;
	if (isTokenExpired(token)) {
		unauthorizedHandler?.();
		const apiError: ApiError = { message: SESSION_EXPIRED, status: 401 };
		return Promise.reject(apiError);
	}
	config.headers.Authorization = `Bearer ${token}`;
	return config;
});

// Response interceptor - normalise every failure to `ApiError`
apiClient.interceptors.response.use(
	(response) => response,
	(error: unknown) => {
		// Already normalised (an expired token stopped the request)
		if (!axios.isAxiosError<ErrorBody>(error)) return Promise.reject(error);
		if (error.response) {
			const data: ErrorBody = error.response.data || {};
			let message = data.error || 'An error occurred';
			const fieldErrors =
				error.response.status === 422 && Array.isArray(data.details)
					? data.details
					: undefined;

			if (fieldErrors?.length) {
				message = `${message} — ${fieldErrors
					.map((d) => `${d.field}: ${d.message}`)
					.join('; ')}`;
			}

			const apiError: ApiError = {
				message,
				status: error.response.status,
				fieldErrors,
			};

			if (error.response.status === 401 && !isAuthEndpoint(error.config?.url)) {
				apiError.message = SESSION_EXPIRED;
				unauthorizedHandler?.();
			}

			return Promise.reject(apiError);
		}
		const apiError: ApiError = error.request
			? {
					message: 'No response from server. Please check your connection.',
					status: 0,
				}
			: {
					message: error.message || 'An unexpected error occurred',
					status: 0,
				};
		return Promise.reject(apiError);
	},
);

export default apiClient;
