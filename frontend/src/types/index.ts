/**
 * Core TypeScript interfaces and types for the InsightBoard application
 */

import type {
	ContractLayoutConfig,
	ContractWidgetType,
} from '../contracts/apiContract';

// User model
export interface User {
	id: string;
	email: string;
	created_at: string;
}

/**
 * Dashboard as the client uses it. `name` is the title (owner, 2026-10-10);
 * `settings_json` is neither read nor written by the client, so it is not mapped.
 */
export interface Dashboard {
	id: string;
	user_id: string;
	name: string;
	layout: LayoutConfig;
	created_at: string;
	updated_at: string;
}

// Layout configuration for GridStack
export type LayoutConfig = ContractLayoutConfig;
export type WidgetType = ContractWidgetType;

// Per-widget user configuration; each widget narrows the keys it reads
export type WidgetConfig = Record<string, unknown>;

// Authentication
export interface AuthResponse {
	token: string;
	user: User;
}

// API Error handling
export interface FieldError {
	field: string;
	message: string;
}

// Normalised error the API client rejects with. `fieldErrors` is set for 422.
export interface ApiError {
	message: string;
	status: number;
	fieldErrors?: FieldError[];
}
