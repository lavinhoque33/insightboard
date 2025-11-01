/**
 * Core TypeScript interfaces and types for the InsightBoard application
 */

// User model
export interface User {
	id: string;
	email: string;
	created_at: string;
}

// Dashboard model
export interface Dashboard {
	id: string;
	user_id: string;
	layout: LayoutConfig;
	settings: DashboardSettings;
	created_at: string;
	updated_at: string;
}

// Layout configuration for GridStack
export interface LayoutConfig {
	widgets: WidgetLayout[];
}

export interface WidgetLayout {
	id: string;
	type: WidgetType;
	x: number;
	y: number;
	w: number;
	h: number;
	config?: Record<string, unknown>;
}

export type WidgetType = 'weather' | 'crypto' | 'github' | 'news' | 'status';

// Dashboard settings
export interface DashboardSettings {
	theme?: 'light' | 'dark';
	title?: string;
	refresh_interval?: number;
}

// Authentication
export interface AuthResponse {
	token: string;
	user: User;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface RegisterRequest {
	email: string;
	password: string;
}

// API Error handling
export interface ApiError {
	message: string;
	status: number;
	details?: Record<string, unknown>;
}

// API Response wrapper
export interface ApiResponse<T> {
	data: T;
	success: boolean;
}

// Dashboard create/update requests
export interface CreateDashboardRequest {
	layout: LayoutConfig;
	settings: DashboardSettings;
}

export interface UpdateDashboardRequest {
	layout?: LayoutConfig;
	settings?: DashboardSettings;
}
