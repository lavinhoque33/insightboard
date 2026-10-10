import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useSaveQueue } from '../useSaveQueue';

interface Deferred {
	promise: Promise<void>;
	resolve: () => void;
	reject: (err: unknown) => void;
}

// A save the test settles by hand, to observe overlap
function deferred(): Deferred {
	let resolve: () => void = () => {};
	let reject: (err: unknown) => void = () => {};
	const promise = new Promise<void>((res, rej) => {
		resolve = res;
		reject = rej;
	});
	return { promise, resolve, reject };
}

describe('useSaveQueue', () => {
	let state: { n: number };
	let calls: { n: number }[];
	let pending: Deferred[];

	const makeQueue = () =>
		useSaveQueue({
			snapshot: () => ({ ...state }),
			save: (value) => {
				calls.push(value);
				const d = deferred();
				pending.push(d);
				return d.promise;
			},
			delay: 2000,
		});

	beforeEach(() => {
		vi.useFakeTimers();
		state = { n: 0 };
		calls = [];
		pending = [];
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('debounces edits into one save of the latest state', async () => {
		const queue = makeQueue();

		state.n = 1;
		queue.schedule();
		state.n = 2;
		queue.schedule();
		expect(queue.status.value).toBe('pending');
		await vi.advanceTimersByTimeAsync(2000);

		expect(calls).toEqual([{ n: 2 }]);
		expect(queue.status.value).toBe('saving');
		pending[0]!.resolve();
		await vi.advanceTimersByTimeAsync(0);
		expect(queue.status.value).toBe('saved');
	});

	it('never runs two saves at once', async () => {
		const queue = makeQueue();
		state.n = 1;
		const first = queue.flush();

		state.n = 2;
		const second = queue.flush();
		expect(calls).toHaveLength(1);

		pending[0]!.resolve();
		await vi.advanceTimersByTimeAsync(0);
		expect(calls).toEqual([{ n: 1 }, { n: 2 }]);

		pending[1]!.resolve();
		expect(await first).toBe(true);
		expect(await second).toBe(true);
	});

	it('sends nothing when the state equals the last save', async () => {
		const queue = makeQueue();

		expect(await queue.flush()).toBe(true);
		expect(calls).toHaveLength(0);
		expect(queue.hasPendingWork()).toBe(false);
	});

	it('keeps the edits after a failure and saves them on retry', async () => {
		const queue = makeQueue();
		state.n = 1;
		const attempt = queue.flush();
		pending[0]!.reject({ status: 500, message: 'Database unavailable' });

		expect(await attempt).toBe(false);
		expect(queue.status.value).toBe('error');
		expect(queue.error.value).toBe('Database unavailable');
		expect(queue.hasPendingWork()).toBe(true);

		const retry = queue.flush();
		pending[1]!.resolve();
		expect(await retry).toBe(true);
		expect(calls).toEqual([{ n: 1 }, { n: 1 }]);
		expect(queue.status.value).toBe('saved');
		expect(queue.error.value).toBeNull();
	});

	it('sends nothing after dispose, even with a save scheduled', async () => {
		const queue = makeQueue();
		state.n = 1;
		queue.schedule();

		queue.dispose();
		await vi.advanceTimersByTimeAsync(10_000);
		await queue.flush();

		expect(calls).toHaveLength(0);
	});

	it('starts from the state recorded by markSaved', async () => {
		const queue = makeQueue();
		state.n = 5;
		queue.markSaved();

		expect(queue.hasPendingWork()).toBe(false);
		await queue.flush();
		expect(calls).toHaveLength(0);
	});
});
