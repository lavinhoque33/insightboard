import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const STORAGE_KEY = 'insightboard-theme';

// A controllable `prefers-color-scheme: dark` media query
function fakeMedia(initiallyDark: boolean) {
	const listeners = new Set<() => void>();
	const media = {
		matches: initiallyDark,
		addEventListener: (_: string, listener: () => void) =>
			listeners.add(listener),
		removeEventListener: (_: string, listener: () => void) =>
			listeners.delete(listener),
	};
	vi.stubGlobal(
		'matchMedia',
		vi.fn(() => media),
	);
	return {
		setDark(dark: boolean) {
			media.matches = dark;
			listeners.forEach((listener) => listener());
		},
	};
}

// The module keeps the choice in module state, so each test loads it fresh
async function loadTheme() {
	vi.resetModules();
	const module = await import('../useTheme');
	module.initTheme();
	return module;
}

const appliedTheme = () => document.documentElement.getAttribute('data-theme');

describe('useTheme', () => {
	beforeEach(() => {
		localStorage.clear();
		document.documentElement.removeAttribute('data-theme');
		document.documentElement.style.colorScheme = '';
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('applies a stored choice over the OS preference', async () => {
		localStorage.setItem(STORAGE_KEY, 'light');
		const os = fakeMedia(true);
		const { useTheme } = await loadTheme();

		expect(useTheme().choice.value).toBe('light');
		expect(appliedTheme()).toBe('insight-light');
		expect(document.documentElement.style.colorScheme).toBe('light');

		os.setDark(false);
		os.setDark(true);
		expect(appliedTheme()).toBe('insight-light');
	});

	it('stores an explicit choice and applies it at once', async () => {
		fakeMedia(false);
		const { useTheme } = await loadTheme();

		useTheme().setChoice('dark');

		expect(localStorage.getItem(STORAGE_KEY)).toBe('dark');
		expect(appliedTheme()).toBe('insight-dark');
		expect(document.documentElement.style.colorScheme).toBe('dark');
	});

	it('"system" removes the stored choice and follows OS changes', async () => {
		localStorage.setItem(STORAGE_KEY, 'dark');
		const os = fakeMedia(false);
		const { useTheme } = await loadTheme();
		expect(appliedTheme()).toBe('insight-dark');

		useTheme().setChoice('system');

		expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
		expect(useTheme().choice.value).toBe('system');
		expect(appliedTheme()).toBe('insight-light');

		os.setDark(true);
		expect(appliedTheme()).toBe('insight-dark');
		os.setDark(false);
		expect(appliedTheme()).toBe('insight-light');
	});

	it('falls back to the OS preference for an invalid stored value', async () => {
		localStorage.setItem(STORAGE_KEY, 'sepia');
		fakeMedia(true);
		const { useTheme } = await loadTheme();

		expect(useTheme().choice.value).toBe('system');
		expect(appliedTheme()).toBe('insight-dark');
	});
});
