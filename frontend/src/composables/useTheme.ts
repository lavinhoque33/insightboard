/**
 * Colour theme: the user's choice (light, dark, or follow the system) is kept
 * in localStorage; "system" tracks `prefers-color-scheme` live. The inline
 * script in index.html applies the same rules before the first paint, so keep
 * the storage key, theme names, and colours in sync with it.
 */

import { readonly, ref } from 'vue';

export type ThemeChoice = 'light' | 'dark' | 'system';

const STORAGE_KEY = 'insightboard-theme';
const THEMES = { light: 'insight-light', dark: 'insight-dark' } as const;
// Page background (base-200) of each theme, for the browser chrome
const THEME_COLORS = { light: '#f6f7fa', dark: '#0d0e14' } as const;

const isChoice = (value: unknown): value is ThemeChoice =>
	value === 'light' || value === 'dark' || value === 'system';

const readChoice = (): ThemeChoice => {
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		return isChoice(stored) ? stored : 'system';
	} catch {
		return 'system';
	}
};

const choice = ref<ThemeChoice>(readChoice());
let media: MediaQueryList | null = null;

const apply = () => {
	const dark =
		choice.value === 'dark' ||
		(choice.value === 'system' && media?.matches === true);
	const mode = dark ? 'dark' : 'light';
	const root = document.documentElement;
	root.setAttribute('data-theme', THEMES[mode]);
	root.style.colorScheme = mode;
	document
		.querySelector('meta[name="theme-color"]')
		?.setAttribute('content', THEME_COLORS[mode]);
};

const setChoice = (next: ThemeChoice) => {
	choice.value = next;
	try {
		if (next === 'system') localStorage.removeItem(STORAGE_KEY);
		else localStorage.setItem(STORAGE_KEY, next);
	} catch {
		// Storage unavailable: the choice lasts for this page only
	}
	apply();
};

/** Starts following the system preference; call once at startup. */
export const initTheme = () => {
	if (media) return;
	media = window.matchMedia('(prefers-color-scheme: dark)');
	media.addEventListener('change', () => {
		if (choice.value === 'system') apply();
	});
	apply();
};

export const useTheme = () => ({ choice: readonly(choice), setChoice });
