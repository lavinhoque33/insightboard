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

// Helper to convert server dashboard shape to frontend `Dashboard`
function mapServerToClient(server: any): Dashboard {
	return {
		id: server.id,
		user_id: server.user_id,
		// backend stores layout as `layout_json` and settings as `settings_json`
		layout: server.layout_json ?? { widgets: [] },
		settings: server.settings_json ?? {},
		created_at: server.created_at,
		updated_at: server.updated_at,
	} as Dashboard;
}

export const dashboardApi = {
	/**
	 * Get all dashboards for the current user
	 */
	async list(): Promise<Dashboard[]> {
		const response = await apiClient.get<any[]>('/dashboards');
		return response.data.map(mapServerToClient);
	},

	/**
	 * Get a specific dashboard by ID
	 */
	async get(id: string): Promise<Dashboard> {
		const response = await apiClient.get<any>(`/dashboards/${id}`);
		return mapServerToClient(response.data);
	},

	/**
	 * Create a new dashboard
	 */
	async create(
		layout: LayoutConfig,
		settings: DashboardSettings,
	): Promise<Dashboard> {
		// Backend expects: { name: string, layout_json: any, settings_json: any }
		const request = {
			name: settings.title || 'New Dashboard',
			layout_json: layout,
			settings_json: settings,
		};
		const response = await apiClient.post<any>('/dashboards', request);
		return mapServerToClient(response.data);
	},

	/**
	 * Update an existing dashboard
	 */
	async update(
		id: string,
		layout?: LayoutConfig,
		settings?: DashboardSettings,
	): Promise<Dashboard> {
		// Backend update endpoint expects partial fields mapped to name/layout_json/settings_json
		const request: any = {};
		if (settings?.title) request.name = settings.title;
		if (layout) request.layout_json = layout;
		if (settings) request.settings_json = settings;

		const response = await apiClient.put<any>(`/dashboards/${id}`, request);
		return mapServerToClient(response.data);
	},

	/**
	 * Delete a dashboard
	 */
	async delete(id: string): Promise<void> {
		await apiClient.delete(`/dashboards/${id}`);
	},
};
