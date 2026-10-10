/**
 * Icon and accent colour of each widget type. Class strings are written out in
 * full so Tailwind finds them.
 */

import type { WidgetType as WidgetKind } from '../../types';
import type { IconName } from './icons';

interface WidgetVisual {
	icon: IconName;
	/** Tinted chip: background, icon colour, ring */
	chip: string;
	/** Solid accent fill (layout thumbnails) */
	fill: string;
}

export const WIDGET_VISUALS: Record<WidgetKind, WidgetVisual> = {
	github: {
		icon: 'git-branch',
		chip: 'bg-widget-github/12 text-widget-github ring-widget-github/25',
		fill: 'fill-widget-github',
	},
	weather: {
		icon: 'cloud-sun',
		chip: 'bg-widget-weather/12 text-widget-weather ring-widget-weather/25',
		fill: 'fill-widget-weather',
	},
	news: {
		icon: 'newspaper',
		chip: 'bg-widget-news/12 text-widget-news ring-widget-news/25',
		fill: 'fill-widget-news',
	},
	crypto: {
		icon: 'coins',
		chip: 'bg-widget-crypto/12 text-widget-crypto ring-widget-crypto/25',
		fill: 'fill-widget-crypto',
	},
	status: {
		icon: 'activity',
		chip: 'bg-widget-status/12 text-widget-status ring-widget-status/25',
		fill: 'fill-widget-status',
	},
};
