import { AxiosError, type AxiosResponse } from 'axios';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import apiClient, { onUnauthorized } from '../client';
import { errorMessage } from '../errorMessage';
import type { ApiError } from '../../types';
import { tokenExpiringIn } from '../../test/session';

const sent: { url?: string; authorization?: unknown }[] = [];

// Replace the network adapter so the real interceptors run against canned responses
function respondWith(status: number, data: unknown, url = '/dashboards'): void {
	apiClient.defaults.adapter = async (config) => {
		sent.push({ url: config.url, authorization: config.headers.Authorization });
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

describe('api client', () => {
	const originalAdapter = apiClient.defaults.adapter;
	const unauthorized = vi.fn();

	beforeEach(() => {
		sent.length = 0;
		localStorage.setItem('auth_token', tokenExpiringIn(3600));
		onUnauthorized(unauthorized);
	});

	afterEach(() => {
		apiClient.defaults.adapter = originalAdapter;
		onUnauthorized(null);
		unauthorized.mockReset();
		localStorage.clear();
	});

	it('sends the stored token as a Bearer header', async () => {
		respondWith(200, []);
		const token = localStorage.getItem('auth_token');

		await apiClient.get('/dashboards');

		expect(sent).toEqual([
			{ url: '/dashboards', authorization: `Bearer ${token}` },
		]);
	});

	it('sends no token to login and register', async () => {
		respondWith(200, {});

		await apiClient.post('/auth/login', {});
		await apiClient.post('/auth/register', {});

		expect(sent.map((r) => r.authorization)).toEqual([undefined, undefined]);
	});

	it('does not send an expired token and ends the session instead', async () => {
		localStorage.setItem('auth_token', tokenExpiringIn(-60));
		respondWith(200, []);

		const err = await failure();

		expect(sent).toHaveLength(0);
		expect(err.status).toBe(401);
		expect(unauthorized).toHaveBeenCalledOnce();
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

	it('reports an unreachable server with status 0', async () => {
		apiClient.defaults.adapter = async (config) => {
			throw new AxiosError('Network Error', AxiosError.ERR_NETWORK, config, {});
		};

		const err = await failure();

		expect(err.status).toBe(0);
		expect(err.message).toContain('No response from server');
	});

	it('ends the session on a 401 from a protected endpoint', async () => {
		respondWith(401, { error: 'Unauthorized' });

		const err = await failure();

		expect(err.status).toBe(401);
		expect(unauthorized).toHaveBeenCalledOnce();
	});

	it.each(['/auth/login', '/auth/register'])(
		'keeps the session on a 401 from %s and shows the server text',
		async (url) => {
			respondWith(401, { error: 'Invalid email or password' }, url);

			const err = await failure(url);

			expect(err.message).toBe('Invalid email or password');
			expect(unauthorized).not.toHaveBeenCalled();
		},
	);
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
