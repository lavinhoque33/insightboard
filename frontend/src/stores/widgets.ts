/**
 * Widget Registry Store
 * Manages available widget types, their configurations, and runtime instances
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Component } from 'vue';

/**
 * Widget Type Definition
 * Describes a widget's metadata, configuration schema, and component
 */
export interface WidgetType {
	id: string;
	name: string;
	description: string;
	icon: string; // Icon class or emoji
	component: Component;
	defaultConfig: Record<string, any>;
	configSchema: WidgetConfigField[];
	defaultSize: {
		w: number; // Width in grid units
		h: number; // Height in grid units
	};
	minSize?: {
		w: number;
		h: number;
	};
	refreshInterval?: number; // Auto-refresh interval in seconds (0 = manual only)
}

/**
 * Widget Configuration Field Schema
 * Defines how to render configuration UI for widget settings
 */
export interface WidgetConfigField {
	key: string;
	label: string;
	type: 'text' | 'number' | 'select' | 'textarea' | 'checkbox' | 'url';
	required?: boolean;
	placeholder?: string;
	options?: { label: string; value: string | number }[];
	validation?: (value: any) => string | null; // Returns error message or null
	helpText?: string;
}

/**
 * Widget Instance
 * Represents a specific widget placed on a dashboard
 */
export interface WidgetInstance {
	id: string; // Unique instance ID
	type: string; // Widget type ID
	config: Record<string, any>; // User configuration
	layout: {
		x: number;
		y: number;
		w: number;
		h: number;
	};
}

export const useWidgetStore = defineStore('widgets', () => {
	// Widget type registry
	const widgetTypes = ref<Map<string, WidgetType>>(new Map());

	/**
	 * Register a widget type
	 */
	const registerWidget = (widget: WidgetType) => {
		console.log('[registerWidget] Registering widget:', widget.id);
		widgetTypes.value.set(widget.id, widget);
	};

	/**
	 * Get widget type by ID
	 */
	const getWidgetType = (id: string): WidgetType | undefined => {
		return widgetTypes.value.get(id);
	};

	/**
	 * Get all available widget types as array
	 */
	const availableWidgets = computed(() => {
		return Array.from(widgetTypes.value.values());
	});

	/**
	 * Create a new widget instance with default configuration
	 */
	const createWidgetInstance = (
		typeId: string,
		position?: { x: number; y: number },
	): WidgetInstance | null => {
		const widgetType = getWidgetType(typeId);
		if (!widgetType) return null;

		return {
			id: `widget-${Date.now()}-${Math.random()
				.toString(36)
				.substr(2, 9)}`,
			type: typeId,
			config: { ...widgetType.defaultConfig },
			layout: {
				x: position?.x ?? 0,
				y: position?.y ?? 0,
				w: widgetType.defaultSize.w,
				h: widgetType.defaultSize.h,
			},
		};
	};

	/**
	 * Validate widget configuration against schema
	 */
	const validateWidgetConfig = (
		typeId: string,
		config: Record<string, any>,
	): { valid: boolean; errors: Record<string, string> } => {
		const widgetType = getWidgetType(typeId);
		if (!widgetType) {
			return {
				valid: false,
				errors: { _general: 'Widget type not found' },
			};
		}

		const errors: Record<string, string> = {};

		for (const field of widgetType.configSchema) {
			const value = config[field.key];

			// Check required fields
			if (
				field.required &&
				(value === undefined || value === null || value === '')
			) {
				errors[field.key] = `${field.label} is required`;
				continue;
			}

			// Run custom validation if provided
			if (field.validation && value !== undefined && value !== null) {
				const error = field.validation(value);
				if (error) {
					errors[field.key] = error;
				}
			}
		}

		return {
			valid: Object.keys(errors).length === 0,
			errors,
		};
	};

	return {
		widgetTypes,
		availableWidgets,
		registerWidget,
		getWidgetType,
		createWidgetInstance,
		validateWidgetConfig,
	};
});
