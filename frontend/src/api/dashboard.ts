/**
 * Dashboard API service
 */

import apiClient from './client';
import type { Dashboard, LayoutConfig } from '../types';
import type {
	ContractCreateDashboardRequest,
	ContractDashboard,
	ContractUpdateDashboardRequest,
} from '../contracts/apiContract';

// The client does not use `settings_json`; it is left untouched on the server
function mapServerToClient(server: ContractDashboard): Dashboard {
	return {
		id: server.id,
		user_id: server.user_id,
		name: server.name,
		layout: server.layout_json,
		created_at: server.created_at,
		updated_at: server.updated_at,
	};
}

/** Fields an update may change; omitted fields keep their stored value. */
export interface DashboardChanges {
	name?: string;
	layout?: LayoutConfig;
}

export const dashboardApi = {
	/**
	 * Get all dashboards for the current user
	 */
	async list(): Promise<Dashboard[]> {
		const response = await apiClient.get<ContractDashboard[]>('/dashboards');
		return response.data.map(mapServerToClient);
	},

	/**
	 * Get a specific dashboard by ID
	 */
	async get(id: string): Promise<Dashboard> {
		const response = await apiClient.get<ContractDashboard>(
			`/dashboards/${encodeURIComponent(id)}`,
		);
		return mapServerToClient(response.data);
	},

	/**
	 * Create a new dashboard; the server stores an empty layout and settings
	 */
	async create(name: string): Promise<Dashboard> {
		const request: ContractCreateDashboardRequest = { name };
		const response = await apiClient.post<ContractDashboard>(
			'/dashboards',
			request,
		);
		return mapServerToClient(response.data);
	},

	/**
	 * Update an existing dashboard
	 */
	async update(id: string, changes: DashboardChanges): Promise<Dashboard> {
		const request: ContractUpdateDashboardRequest = {};
		if (changes.name !== undefined) request.name = changes.name;
		if (changes.layout) request.layout_json = changes.layout;

		const response = await apiClient.put<ContractDashboard>(
			`/dashboards/${encodeURIComponent(id)}`,
			request,
		);
		return mapServerToClient(response.data);
	},

	/**
	 * Delete a dashboard
	 */
	async delete(id: string): Promise<void> {
		await apiClient.delete(`/dashboards/${encodeURIComponent(id)}`);
	},
};
