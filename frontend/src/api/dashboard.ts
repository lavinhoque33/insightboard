/**
 * Dashboard API service
 */

import apiClient from './client';
import type {
	Dashboard,
	CreateDashboardRequest,
	UpdateDashboardRequest,
	LayoutConfig,
	DashboardSettings,
} from '../types';

export const dashboardApi = {
	/**
	 * Get all dashboards for the current user
	 */
	async list(): Promise<Dashboard[]> {
		const response = await apiClient.get<Dashboard[]>('/dashboards');
		return response.data;
	},

	/**
	 * Get a specific dashboard by ID
	 */
	async get(id: string): Promise<Dashboard> {
		const response = await apiClient.get<Dashboard>(`/dashboards/${id}`);
		return response.data;
	},

	/**
	 * Create a new dashboard
	 */
	async create(
		layout: LayoutConfig,
		settings: DashboardSettings,
	): Promise<Dashboard> {
		const request: CreateDashboardRequest = { layout, settings };
		const response = await apiClient.post<Dashboard>(
			'/dashboards',
			request,
		);
		return response.data;
	},

	/**
	 * Update an existing dashboard
	 */
	async update(
		id: string,
		layout?: LayoutConfig,
		settings?: DashboardSettings,
	): Promise<Dashboard> {
		const request: UpdateDashboardRequest = { layout, settings };
		const response = await apiClient.put<Dashboard>(
			`/dashboards/${id}`,
			request,
		);
		return response.data;
	},

	/**
	 * Delete a dashboard
	 */
	async delete(id: string): Promise<void> {
		await apiClient.delete(`/dashboards/${id}`);
	},
};
