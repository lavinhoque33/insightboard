/**
 * Authentication store using Pinia
 * Manages user authentication state, login, register, and logout
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { authApi } from '../api';
import type { User, ApiError } from '../types';

export const useAuthStore = defineStore('auth', () => {
	// State
	const user = ref<User | null>(null);
	const token = ref<string | null>(null);
	const loading = ref(false);
	const error = ref<string | null>(null);

	// Getters
	const isAuthenticated = computed(() => !!token.value && !!user.value);

	/**
	 * Initialize store from localStorage
	 */
	function initialize() {
		const savedToken = localStorage.getItem('auth_token');
		const savedUser = localStorage.getItem('user');

		if (savedToken && savedUser) {
			token.value = savedToken;
			try {
				user.value = JSON.parse(savedUser);
			} catch (e) {
				// Invalid user data, clear everything
				logout();
			}
		}
	}

	/**
	 * Register a new user
	 */
	async function register(email: string, password: string): Promise<void> {
		loading.value = true;
		error.value = null;

		try {
			const response = await authApi.register(email, password);

			// Store token and user
			token.value = response.token;
			user.value = response.user;

			// Persist to localStorage
			localStorage.setItem('auth_token', response.token);
			localStorage.setItem('user', JSON.stringify(response.user));
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Login user
	 */
	async function login(email: string, password: string): Promise<void> {
		loading.value = true;
		error.value = null;

		try {
			const response = await authApi.login(email, password);

			// Store token and user
			token.value = response.token;
			user.value = response.user;

			// Persist to localStorage
			localStorage.setItem('auth_token', response.token);
			localStorage.setItem('user', JSON.stringify(response.user));
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Logout user
	 */
	function logout() {
		user.value = null;
		token.value = null;
		error.value = null;

		// Clear localStorage
		localStorage.removeItem('auth_token');
		localStorage.removeItem('user');
	}

	/**
	 * Fetch current user from API
	 */
	async function fetchCurrentUser(): Promise<void> {
		if (!token.value) {
			return;
		}

		loading.value = true;
		error.value = null;

		try {
			const userData = await authApi.getCurrentUser();
			user.value = userData;
			localStorage.setItem('user', JSON.stringify(userData));
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			// If fetching user fails, logout
			logout();
			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Clear error
	 */
	function clearError() {
		error.value = null;
	}

	// Initialize on store creation
	initialize();

	return {
		// State
		user,
		token,
		loading,
		error,
		// Getters
		isAuthenticated,
		// Actions
		register,
		login,
		logout,
		fetchCurrentUser,
		clearError,
	};
});
