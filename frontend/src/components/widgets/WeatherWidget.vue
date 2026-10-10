<!-- Weather Widget: current weather for a city -->
<script setup lang="ts">
import BaseWidget from '../BaseWidget.vue';
import AppIcon from '../ui/AppIcon.vue';
import type { IconName } from '../ui/icons';
import { useWidgetData } from '../../composables/useWidgetData';
import { fetchWeatherData } from '../../api/widgets';

interface Props {
	config: {
		city: string;
	};
	refreshInterval?: number;
}

interface Emits {
	(e: 'configure'): void;
	(e: 'remove'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const {
	data: weather,
	loading,
	error,
	lastUpdated,
	reload,
} = useWidgetData({
	source: () => props.config.city,
	fetch: fetchWeatherData,
	refreshInterval: () => props.refreshInterval,
	notConfigured: 'City not configured',
	failure: 'Failed to load weather data',
});

// OpenWeather icon code (e.g. "10d") to a drawn icon
const CONDITION_ICONS: Record<string, IconName> = {
	'01d': 'sun',
	'01n': 'moon',
	'02d': 'cloud-sun',
	'02n': 'cloud',
	'03': 'cloud',
	'04': 'cloud',
	'09': 'cloud-rain',
	'10': 'cloud-rain',
	'11': 'cloud-lightning',
	'13': 'snowflake',
	'50': 'fog',
};

const conditionIcon = (code: string): IconName =>
	CONDITION_ICONS[code] ?? CONDITION_ICONS[code.slice(0, 2)] ?? 'cloud-sun';
</script>

<template>
	<BaseWidget
		type="weather"
		:title="`Weather: ${config.city || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div v-if="weather" class="flex h-full flex-col justify-between gap-3">
			<div class="flex items-center justify-between gap-3">
				<div class="min-w-0">
					<p
						class="flex items-start text-5xl font-semibold leading-none tracking-tighter tabular-nums"
					>
						{{ Math.round(weather.temperature) }}
						<span class="ml-0.5 mt-1 text-xl font-medium text-base-content/70"
							>°C</span
						>
					</p>
					<p class="mt-2 truncate text-sm">
						<span class="font-medium capitalize">{{
							weather.description
						}}</span>
						<span class="text-base-content/70">
							· {{ weather.city }}, {{ weather.country }}</span
						>
					</p>
				</div>
				<div class="relative shrink-0" aria-hidden="true">
					<div
						class="absolute inset-0 scale-125 rounded-full bg-widget-weather/20 blur-xl"
					></div>
					<AppIcon
						:name="conditionIcon(weather.icon)"
						class="relative size-12 text-widget-weather @xs:size-14"
					/>
				</div>
			</div>

			<dl class="grid grid-cols-3 gap-2">
				<div class="rounded-lg bg-base-200/70 px-2.5 py-1.5">
					<dt
						class="flex items-center gap-1 truncate text-[11px] text-base-content/70"
					>
						<AppIcon name="thermometer" class="size-3" />
						Feels like
					</dt>
					<dd class="mt-0.5 text-sm font-semibold tabular-nums">
						{{ Math.round(weather.feels_like) }}°C
					</dd>
				</div>
				<div class="rounded-lg bg-base-200/70 px-2.5 py-1.5">
					<dt
						class="flex items-center gap-1 truncate text-[11px] text-base-content/70"
					>
						<AppIcon name="droplet" class="size-3" />
						Humidity
					</dt>
					<dd class="mt-0.5 text-sm font-semibold tabular-nums">
						{{ weather.humidity }}%
					</dd>
				</div>
				<div class="rounded-lg bg-base-200/70 px-2.5 py-1.5">
					<dt
						class="flex items-center gap-1 truncate text-[11px] text-base-content/70"
					>
						<AppIcon name="wind" class="size-3" />
						Wind
					</dt>
					<dd class="mt-0.5 text-sm font-semibold tabular-nums">
						{{ weather.wind_speed }} m/s
					</dd>
				</div>
			</dl>
		</div>
	</BaseWidget>
</template>
