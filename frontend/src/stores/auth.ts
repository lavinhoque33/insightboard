/**
 * Authentication store using Pinia
 * Manages the session (token and user), login, register, and logout
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { authApi } from '../api';
import { TOKEN_KEY, USER_KEY, isTokenExpired } from '../api/token';
import { apiErrorMessage } from '../api/errorMessage';
import type { User, ApiError, FieldError, AuthResponse } from '../types';
import { useDashboardStore } from './dashboard';

export const useAuthStore = defineStore('auth', () => {
	// State
	const user = ref<User | null>(null);
	const token = ref<string | null>(null);
	// Login and register form state
	const loading = ref(false);
	const error = ref<string | null>(null);
	const fieldErrors = ref<FieldError[]>([]);

	// Getters
	const isAuthenticated = computed(() => !!token.value && !!user.value);

	/**
	 * Record a failed auth call. 422 responses keep per-field messages for the form
	 * and a short summary; every other status shows the backend's message.
	 */
	function setAuthError(err: unknown) {
		fieldErrors.value = (err as Partial<ApiError>).fieldErrors ?? [];
		error.value = fieldErrors.value.length
			? 'Please correct the highlighted fields.'
			: apiErrorMessage(err, 'Something went wrong. Please try again.');
	}

	function setSession(response: AuthResponse) {
		token.value = response.token;
		user.value = response.user;
		localStorage.setItem(TOKEN_KEY, response.token);
		localStorage.setItem(USER_KEY, JSON.stringify(response.user));
	}

	/**
	 * End the session and clear every store that holds the user's data.
	 */
	function logout() {
		user.value = null;
		token.value = null;
		loading.value = false;
		error.value = null;
		fieldErrors.value = [];
		localStorage.removeItem(TOKEN_KEY);
		localStorage.removeItem(USER_KEY);
		useDashboardStore().reset();
	}

	/**
	 * Drop a session whose token is past its `exp` (24 h). Returns whether the
	 * user is still signed in.
	 */
	function checkSession(): boolean {
		if (token.value && isTokenExpired(token.value)) logout();
		return isAuthenticated.value;
	}

	// Restore a stored, unexpired session
	const savedToken = localStorage.getItem(TOKEN_KEY);
	const savedUser = localStorage.getItem(USER_KEY);
	if (savedToken && savedUser && !isTokenExpired(savedToken)) {
		try {
			user.value = JSON.parse(savedUser);
			token.value = savedToken;
		} catch {
			logout();
		}
	} else if (savedToken || savedUser) {
		logout();
	}

	async function authenticate(
		call: () => Promise<AuthResponse>,
	): Promise<void> {
		loading.value = true;
		error.value = null;
		fieldErrors.value = [];
		try {
			setSession(await call());
		} catch (err) {
			setAuthError(err);
			throw err;
		} finally {
			loading.value = false;
		}
	}

	async function register(email: string, password: string): Promise<void> {
		await authenticate(() => authApi.register(email, password));
	}

	async function login(email: string, password: string): Promise<void> {
		await authenticate(() => authApi.login(email, password));
	}

	/**
	 * Confirm a restored session with the server and refresh the user. A 401 ends
	 * the session through the API client's unauthorized handler; other failures
	 * (server unreachable) keep the stored session.
	 */
	async function fetchCurrentUser(): Promise<void> {
		if (!token.value) return;
		try {
			const current = await authApi.getCurrentUser();
			if (!token.value) return;
			user.value = current;
			localStorage.setItem(USER_KEY, JSON.stringify(current));
		} catch {
			// Handled by the unauthorized handler (401) or harmless (offline)
		}
	}

	function clearError() {
		error.value = null;
		fieldErrors.value = [];
	}

	return {
		// State
		user,
		token,
		loading,
		error,
		fieldErrors,
		// Getters
		isAuthenticated,
		// Actions
		register,
		login,
		logout,
		checkSession,
		fetchCurrentUser,
		clearError,
	};
});
