/**
 * Authentication API service
 */

import apiClient from './client';
import type { AuthResponse, User } from '../types';
import type {
	ContractAuthResponse,
	ContractLoginRequest,
	ContractRegisterRequest,
	ContractUser,
} from '../contracts/apiContract';

export const authApi = {
	/**
	 * Register a new user
	 */
	async register(email: string, password: string): Promise<AuthResponse> {
		const request: ContractRegisterRequest = { email, password };
		const response = await apiClient.post<ContractAuthResponse>(
			'/auth/register',
			request,
		);
		return response.data;
	},

	/**
	 * Login user
	 */
	async login(email: string, password: string): Promise<AuthResponse> {
		const request: ContractLoginRequest = { email, password };
		const response = await apiClient.post<ContractAuthResponse>(
			'/auth/login',
			request,
		);
		return response.data;
	},

	/**
	 * Get current authenticated user
	 */
	async getCurrentUser(): Promise<User> {
		const response = await apiClient.get<ContractUser>('/me');
		return response.data;
	},
};
