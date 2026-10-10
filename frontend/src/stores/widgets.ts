/**
 * Widget Registry Store
 * Manages available widget types, their configurations, and runtime instances
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Component } from 'vue';
import type { WidgetConfig, WidgetType as WidgetKind } from '../types';

/**
 * Widget Type Definition
 * Describes a widget's metadata, configuration schema, and component
 */
export interface WidgetType {
	id: WidgetKind;
	name: string;
	description: string;
	icon: string; // Icon class or emoji
	component: Component;
	defaultConfig: WidgetConfig;
	configSchema: WidgetConfigField[];
	defaultSize: {
		w: number; // Width in grid units
		h: number; // Height in grid units
	};
	minSize: {
		w: number;
		h: number;
	};
	refreshInterval: number; // Auto-refresh interval in seconds (0 = manual only)
}

/**
 * Widget Configuration Field Schema
 * Defines how to render configuration UI for widget settings. Every field holds
 * a string, so validators receive the (trimmed) string the user entered.
 */
interface ConfigFieldBase {
	key: string;
	label: string;
	required?: boolean;
	helpText?: string;
}

export interface TextConfigField extends ConfigFieldBase {
	type: 'text' | 'textarea';
	placeholder?: string;
	validation?: (value: string) => string | null; // Error message or null
}

export interface SelectConfigField extends ConfigFieldBase {
	type: 'select';
	options: { label: string; value: string }[];
}

export type WidgetConfigField = TextConfigField | SelectConfigField;

/**
 * Widget Instance
 * Represents a specific widget placed on a dashboard
 */
export interface WidgetInstance {
	id: string; // Unique instance ID
	type: WidgetKind;
	config: WidgetConfig; // User configuration
	layout: {
		x: number;
		y: number;
		w: number;
		h: number;
	};
}

export const useWidgetStore = defineStore('widgets', () => {
	// Widget type registry
	const widgetTypes = ref<Map<WidgetKind, WidgetType>>(new Map());

	/**
	 * Register a widget type
	 */
	const registerWidget = (widget: WidgetType) => {
		widgetTypes.value.set(widget.id, widget);
	};

	/**
	 * Get widget type by ID
	 */
	const getWidgetType = (id: WidgetKind): WidgetType | undefined => {
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
		typeId: WidgetKind,
		position?: { x: number; y: number },
	): WidgetInstance | null => {
		const widgetType = getWidgetType(typeId);
		if (!widgetType) return null;

		return {
			id: `widget-${crypto.randomUUID()}`,
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

	return {
		widgetTypes,
		availableWidgets,
		registerWidget,
		getWidgetType,
		createWidgetInstance,
	};
});
