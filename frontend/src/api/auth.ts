/**
 * Authentication API service
 */

import apiClient from './client';
import type {
	AuthResponse,
	LoginRequest,
	RegisterRequest,
	User,
} from '../types';

export const authApi = {
	/**
	 * Register a new user
	 */
	async register(email: string, password: string): Promise<AuthResponse> {
		const request: RegisterRequest = { email, password };
		const response = await apiClient.post<AuthResponse>(
			'/auth/register',
			request,
		);
		return response.data;
	},

	/**
	 * Login user
	 */
	async login(email: string, password: string): Promise<AuthResponse> {
		const request: LoginRequest = { email, password };
		const response = await apiClient.post<AuthResponse>(
			'/auth/login',
			request,
		);
		return response.data;
	},

	/**
	 * Get current authenticated user
	 */
	async getCurrentUser(): Promise<User> {
		const response = await apiClient.get<User>('/auth/me');
		return response.data;
	},
};
