/**
 * Data loading shared by the widgets: loading, error, and last-updated state; a
 * reload when the configured source changes; auto-refresh on the registry
 * interval; and no state updates from responses that arrive after the source
 * changed or the widget was removed.
 */

import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import type { Ref } from 'vue';
import { errorMessage } from '../api/errorMessage';

interface Options<T> {
	/** Current source (city, username, ...); empty means not configured */
	source: () => string;
	fetch: (source: string) => Promise<T>;
	/** Seconds between automatic refreshes; 0 or undefined disables them */
	refreshInterval: () => number | undefined;
	notConfigured: string;
	failure: string;
}

export function useWidgetData<T>(options: Options<T>) {
	const data = ref<T | null>(null) as Ref<T | null>;
	const loading = ref(false);
	const error = ref<string | null>(null);
	const lastUpdated = ref<Date | null>(null);

	let request = 0;
	let active = true;
	let timer: number | undefined;

	async function load() {
		const source = options.source();
		const current = ++request;
		if (!source) {
			data.value = null;
			loading.value = false;
			error.value = options.notConfigured;
			return;
		}
		loading.value = true;
		error.value = null;
		try {
			const result = await options.fetch(source);
			if (!active || current !== request) return;
			data.value = result;
			lastUpdated.value = new Date();
		} catch (err) {
			if (!active || current !== request) return;
			data.value = null;
			error.value = errorMessage(err, options.failure);
		} finally {
			if (active && current === request) loading.value = false;
		}
	}

	function schedule() {
		window.clearInterval(timer);
		const seconds = options.refreshInterval();
		if (seconds && seconds > 0)
			timer = window.setInterval(load, seconds * 1000);
	}

	onMounted(() => {
		void load();
		schedule();
	});
	watch(options.source, () => void load());
	watch(options.refreshInterval, schedule);
	onBeforeUnmount(() => {
		active = false;
		window.clearInterval(timer);
	});

	return { data, loading, error, lastUpdated, reload: load };
}
