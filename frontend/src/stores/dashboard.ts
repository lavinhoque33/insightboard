/**
 * Dashboard store using Pinia
 * Holds the dashboard list and the dashboard open in the editor, each with
 * explicit loading and error state.
 */

import { defineStore } from 'pinia';
import { ref } from 'vue';
import { dashboardApi } from '../api';
import type { DashboardChanges } from '../api/dashboard';
import type { Dashboard } from '../types';
import { apiErrorMessage, apiErrorStatus } from '../api/errorMessage';

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export const useDashboardStore = defineStore('dashboard', () => {
	// List page
	const dashboards = ref<Dashboard[]>([]);
	const listLoading = ref(false);
	const listError = ref<string | null>(null);
	// Create and delete failures on the list page
	const actionError = ref<string | null>(null);

	// Editor: the dashboard loaded by id
	const current = ref<Dashboard | null>(null);
	const currentLoading = ref(false);
	const currentError = ref<string | null>(null);
	// Unknown id, foreign id (the server answers 404 for both), or not an id at all
	const currentNotFound = ref(false);
	let currentRequest = 0;

	async function fetchDashboards(): Promise<void> {
		listLoading.value = true;
		listError.value = null;
		try {
			dashboards.value = await dashboardApi.list();
		} catch (err) {
			listError.value = apiErrorMessage(err, 'Could not load dashboards');
		} finally {
			listLoading.value = false;
		}
	}

	async function fetchDashboard(id: string): Promise<void> {
		const request = ++currentRequest;
		current.value = null;
		currentError.value = null;
		currentNotFound.value = false;

		if (!UUID.test(id)) {
			currentNotFound.value = true;
			return;
		}

		currentLoading.value = true;
		try {
			const data = await dashboardApi.get(id);
			if (request === currentRequest) current.value = data;
		} catch (err) {
			if (request !== currentRequest) return;
			if (apiErrorStatus(err) === 404) {
				currentNotFound.value = true;
			} else {
				currentError.value = apiErrorMessage(
					err,
					'Could not load the dashboard',
				);
			}
		} finally {
			if (request === currentRequest) currentLoading.value = false;
		}
	}

	async function createDashboard(name: string): Promise<Dashboard> {
		actionError.value = null;
		try {
			const data = await dashboardApi.create(name);
			dashboards.value.push(data);
			return data;
		} catch (err) {
			actionError.value = apiErrorMessage(
				err,
				'Could not create the dashboard',
			);
			throw err;
		}
	}

	/**
	 * Persist changes. Failures are rethrown for the caller (the editor's save
	 * queue) to show; the store copies only change on success.
	 */
	async function updateDashboard(
		id: string,
		changes: DashboardChanges,
	): Promise<Dashboard> {
		const data = await dashboardApi.update(id, changes);
		const index = dashboards.value.findIndex((d) => d.id === id);
		if (index !== -1) dashboards.value[index] = data;
		if (current.value?.id === id) current.value = data;
		return data;
	}

	async function deleteDashboard(id: string): Promise<void> {
		actionError.value = null;
		try {
			await dashboardApi.delete(id);
			dashboards.value = dashboards.value.filter((d) => d.id !== id);
			if (current.value?.id === id) current.value = null;
		} catch (err) {
			actionError.value = apiErrorMessage(
				err,
				'Could not delete the dashboard',
			);
			throw err;
		}
	}

	/** Forget everything (logout). In-flight loads finish without effect. */
	function reset() {
		currentRequest++;
		dashboards.value = [];
		listLoading.value = false;
		listError.value = null;
		actionError.value = null;
		current.value = null;
		currentLoading.value = false;
		currentError.value = null;
		currentNotFound.value = false;
	}

	return {
		dashboards,
		listLoading,
		listError,
		actionError,
		current,
		currentLoading,
		currentError,
		currentNotFound,
		fetchDashboards,
		fetchDashboard,
		createDashboard,
		updateDashboard,
		deleteDashboard,
		reset,
	};
});
