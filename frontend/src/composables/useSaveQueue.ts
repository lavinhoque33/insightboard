/**
 * Debounced, serialised auto-save. At most one save runs at a time: a save
 * requested while one is in flight runs after it, with the state current at that
 * moment. A snapshot equal to the last saved one sends nothing. A failure stops
 * the queue and keeps the edits in memory until `flush()` is called again
 * (Retry, the next edit, or leaving the page). After `dispose()` nothing is sent.
 */

import { computed, ref } from 'vue';
import type { ComputedRef, Ref } from 'vue';
import { apiErrorMessage } from '../api/errorMessage';

export type SaveStatus = 'idle' | 'pending' | 'saving' | 'saved' | 'error';

interface Options<T> {
	/** The state to persist, read when a save starts */
	snapshot: () => T;
	save: (value: T) => Promise<unknown>;
	/** Debounce for `schedule()`, in milliseconds */
	delay: number;
}

export interface SaveQueue {
	status: ComputedRef<SaveStatus>;
	error: Readonly<Ref<string | null>>;
	/** Record the current state as persisted (after loading) */
	markSaved: () => void;
	/** Save after the debounce delay; restarts the delay */
	schedule: () => void;
	/** Save now; resolves true once everything is persisted */
	flush: () => Promise<boolean>;
	/** Unsaved edits or a save in flight */
	hasPendingWork: () => boolean;
	/** Stop for good: cancel the timer and start no further request */
	dispose: () => void;
}

export function useSaveQueue<T>(options: Options<T>): SaveQueue {
	const state = ref<'idle' | 'saving' | 'saved'>('idle');
	const error = ref<string | null>(null);
	const dirty = ref(false);

	let lastSaved = JSON.stringify(options.snapshot());
	let timer: number | undefined;
	let running: Promise<boolean> | null = null;
	let rerun = false;
	let disposed = false;

	const isDirty = () => JSON.stringify(options.snapshot()) !== lastSaved;

	async function run(): Promise<boolean> {
		do {
			rerun = false;
			if (disposed) return false;
			const value = options.snapshot();
			const key = JSON.stringify(value);
			if (key === lastSaved) continue;
			state.value = 'saving';
			error.value = null;
			try {
				await options.save(value);
				lastSaved = key;
			} catch (err) {
				error.value = apiErrorMessage(err, 'The server did not respond.');
				return false;
			}
		} while (rerun);
		dirty.value = isDirty();
		state.value = dirty.value ? 'idle' : 'saved';
		return !dirty.value;
	}

	function flush(): Promise<boolean> {
		window.clearTimeout(timer);
		timer = undefined;
		if (disposed) return Promise.resolve(false);
		if (running) {
			rerun = true;
			return running;
		}
		running = run().finally(() => {
			running = null;
			dirty.value = isDirty();
		});
		return running;
	}

	function schedule() {
		if (disposed) return;
		dirty.value = isDirty();
		window.clearTimeout(timer);
		timer = window.setTimeout(() => void flush(), options.delay);
	}

	const status = computed<SaveStatus>(() => {
		if (error.value) return 'error';
		if (state.value === 'saving') return 'saving';
		if (dirty.value) return 'pending';
		return state.value;
	});

	return {
		status,
		error,
		markSaved: () => {
			lastSaved = JSON.stringify(options.snapshot());
			dirty.value = false;
			error.value = null;
			state.value = 'idle';
		},
		schedule,
		flush,
		hasPendingWork: () => running !== null || isDirty(),
		dispose: () => {
			disposed = true;
			window.clearTimeout(timer);
			timer = undefined;
		},
	};
}
