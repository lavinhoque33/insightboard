import { AxiosError, type AxiosResponse } from 'axios';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import apiClient from '../client';
import { errorMessage } from '../errorMessage';
import type { ApiError } from '../../types';

// Replace the network adapter so the real interceptors run against canned responses
function respondWith(status: number, data: unknown, url = '/dashboards'): void {
	apiClient.defaults.adapter = async (config) => {
		const response: AxiosResponse = {
			data,
			status,
			statusText: '',
			headers: {},
			config,
		};
		if (status >= 200 && status < 300) return response;
		throw new AxiosError(
			`Request failed with status code ${status}`,
			AxiosError.ERR_BAD_REQUEST,
			{ ...config, url },
			null,
			response,
		);
	};
}

async function failure(url?: string): Promise<ApiError> {
	try {
		await apiClient.get(url ?? '/dashboards');
	} catch (err) {
		return err as ApiError;
	}
	throw new Error('expected request to fail');
}

describe('api client error normalisation', () => {
	const originalAdapter = apiClient.defaults.adapter;

	beforeEach(() => {
		localStorage.setItem('auth_token', 'token');
		localStorage.setItem('user', '{}');
	});

	afterEach(() => {
		apiClient.defaults.adapter = originalAdapter;
		vi.unstubAllGlobals();
		localStorage.clear();
	});

	it('renders 422 details as field: message items and exposes fieldErrors', async () => {
		const details = [
			{ field: 'email', message: 'must be a valid email address' },
			{ field: 'layout_json.widgets.0.w', message: 'must be between 1 and 12' },
		];
		respondWith(422, { error: 'Validation failed', details });

		const err = await failure();

		expect(err.status).toBe(422);
		expect(err.message).toBe(
			'Validation failed — email: must be a valid email address; layout_json.widgets.0.w: must be between 1 and 12',
		);
		expect(err.message).not.toContain('[object Object]');
		expect(err.fieldErrors).toEqual(details);
	});

	it('uses the {error} text for 409 without field errors', async () => {
		respondWith(409, { error: 'Email already registered' });

		const err = await failure();

		expect(err).toEqual({
			message: 'Email already registered',
			status: 409,
			fieldErrors: undefined,
		});
	});

	it('uses the {error} text for 429', async () => {
		respondWith(429, { error: 'Too many requests' });

		const err = await failure();

		expect(err.status).toBe(429);
		expect(err.message).toBe('Too many requests');
	});

	it('falls back to a generic message when the body has no error', async () => {
		respondWith(500, '<html>proxy</html>');

		expect((await failure()).message).toBe('An error occurred');
	});

	it('clears the session and redirects to /login on 401', async () => {
		const location = { pathname: '/dashboards', href: '/dashboards' };
		vi.stubGlobal('location', location);
		respondWith(401, { error: 'Unauthorized' });

		const err = await failure();

		expect(err.status).toBe(401);
		expect(localStorage.getItem('auth_token')).toBeNull();
		expect(localStorage.getItem('user')).toBeNull();
		expect(location.href).toBe('/login');
	});

	it.each(['/auth/login', '/auth/register'])(
		'keeps the session and does not redirect on 401 from %s',
		async (url) => {
			const location = { pathname: '/register', href: '/register' };
			vi.stubGlobal('location', location);
			respondWith(401, { error: 'Unauthorized' }, url);

			const err = await failure(url);

			expect(err.message).toBe('Unauthorized');
			expect(localStorage.getItem('auth_token')).toBe('token');
			expect(location.href).toBe('/register');
		},
	);

	it('does not redirect when already on /login', async () => {
		const location = { pathname: '/login', href: '/login' };
		vi.stubGlobal('location', location);
		respondWith(401, { error: 'Unauthorized' });

		await failure();

		expect(location.href).toBe('/login');
	});
});

describe('errorMessage', () => {
	it.each([
		[429, 'Too many requests'],
		[502, 'The data provider failed'],
		[503, 'provider key is not configured'],
	])('maps status %i to user text', (status, text) => {
		expect(errorMessage({ status, message: 'raw' }, 'fallback')).toContain(
			text,
		);
	});

	it('passes through other messages and falls back otherwise', () => {
		expect(errorMessage({ status: 404, message: 'Not found' }, 'x')).toBe(
			'Not found',
		);
		expect(errorMessage(null, 'fallback')).toBe('fallback');
	});
});
