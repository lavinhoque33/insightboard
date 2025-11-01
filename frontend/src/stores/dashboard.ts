/**
 * Dashboard store using Pinia
 * Manages dashboard state and operations
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { dashboardApi } from '../api';
import type {
	Dashboard,
	LayoutConfig,
	DashboardSettings,
	ApiError,
} from '../types';

export const useDashboardStore = defineStore('dashboard', () => {
	// State
	const dashboards = ref<Dashboard[]>([]);
	const currentDashboard = ref<Dashboard | null>(null);
	const loading = ref(false);
	const error = ref<string | null>(null);

	// Getters
	const getDashboardById = computed(() => {
		return (id: string) => dashboards.value.find((d) => d.id === id);
	});

	const hasDashboards = computed(() => dashboards.value.length > 0);

	/**
	 * Fetch all dashboards
	 */
	async function fetchDashboards(): Promise<void> {
		loading.value = true;
		error.value = null;

		try {
			const data = await dashboardApi.list();
			dashboards.value = data;
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Fetch a specific dashboard
	 */
	async function fetchDashboard(id: string): Promise<void> {
		loading.value = true;
		error.value = null;

		try {
			const data = await dashboardApi.get(id);
			currentDashboard.value = data;

			// Update in dashboards array if exists
			const index = dashboards.value.findIndex((d) => d.id === id);
			if (index !== -1) {
				dashboards.value[index] = data;
			} else {
				dashboards.value.push(data);
			}
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Create a new dashboard
	 */
	async function createDashboard(
		layout: LayoutConfig,
		settings: DashboardSettings,
	): Promise<Dashboard> {
		loading.value = true;
		error.value = null;

		try {
			const data = await dashboardApi.create(layout, settings);
			dashboards.value.push(data);
			currentDashboard.value = data;
			return data;
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Update an existing dashboard
	 */
	async function updateDashboard(
		id: string,
		layout?: LayoutConfig,
		settings?: DashboardSettings,
	): Promise<Dashboard> {
		loading.value = true;
		error.value = null;

		try {
			const data = await dashboardApi.update(id, layout, settings);

			// Update in dashboards array
			const index = dashboards.value.findIndex((d) => d.id === id);
			if (index !== -1) {
				dashboards.value[index] = data;
			}

			// Update current dashboard if it's the same
			if (currentDashboard.value?.id === id) {
				currentDashboard.value = data;
			}

			return data;
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Delete a dashboard
	 */
	async function deleteDashboard(id: string): Promise<void> {
		loading.value = true;
		error.value = null;

		try {
			await dashboardApi.delete(id);

			// Remove from dashboards array
			dashboards.value = dashboards.value.filter((d) => d.id !== id);

			// Clear current dashboard if it's the deleted one
			if (currentDashboard.value?.id === id) {
				currentDashboard.value = null;
			}
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Set current dashboard
	 */
	function setCurrentDashboard(dashboard: Dashboard | null) {
		currentDashboard.value = dashboard;
	}

	/**
	 * Clear error
	 */
	function clearError() {
		error.value = null;
	}

	/**
	 * Reset store state
	 */
	function reset() {
		dashboards.value = [];
		currentDashboard.value = null;
		loading.value = false;
		error.value = null;
	}

	return {
		// State
		dashboards,
		currentDashboard,
		loading,
		error,
		// Getters
		getDashboardById,
		hasDashboards,
		// Actions
		fetchDashboards,
		fetchDashboard,
		createDashboard,
		updateDashboard,
		deleteDashboard,
		setCurrentDashboard,
		clearError,
		reset,
	};
});
