/** * Weather Widget * Displays current weather for a city */
<script setup lang="ts">
import BaseWidget from '../BaseWidget.vue';
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

const getWeatherIcon = (iconCode: string): string => {
	const icons: Record<string, string> = {
		'01d': '☀️',
		'01n': '🌙',
		'02d': '⛅',
		'02n': '☁️',
		'03d': '☁️',
		'03n': '☁️',
		'04d': '☁️',
		'04n': '☁️',
		'09d': '🌧️',
		'09n': '🌧️',
		'10d': '🌦️',
		'10n': '🌧️',
		'11d': '⛈️',
		'11n': '⛈️',
		'13d': '❄️',
		'13n': '❄️',
		'50d': '🌫️',
		'50n': '🌫️',
	};
	return icons[iconCode] || '🌤️';
};
</script>

<template>
	<BaseWidget
		:title="`Weather: ${config.city || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="weather"
			class="flex flex-col items-center justify-center space-y-6 py-4"
		>
			<!-- Main Temperature Card -->
			<div
				class="bg-linear-to-br from-primary/10 to-secondary/10 rounded-2xl p-8 w-full text-center border border-base-300"
			>
				<!-- Weather Icon -->
				<div class="text-7xl mb-4">
					{{ getWeatherIcon(weather.icon) }}
				</div>

				<!-- Temperature -->
				<div
					class="text-5xl font-bold text-transparent bg-linear-to-r from-primary to-secondary bg-clip-text mb-2"
				>
					{{ Math.round(weather.temperature) }}°C
				</div>

				<!-- Feels Like -->
				<div class="text-sm text-base-content/60 mb-4">
					Feels like {{ Math.round(weather.feels_like) }}°C
				</div>

				<!-- Description -->
				<p class="text-lg font-semibold text-base-content capitalize">
					{{ weather.description }}
				</p>
				<p class="text-sm text-base-content/60 mt-1">
					{{ weather.city }}, {{ weather.country }}
				</p>
			</div>

			<!-- Weather Details Grid -->
			<div class="divider my-2"></div>

			<div class="grid grid-cols-2 gap-3 w-full">
				<!-- Humidity -->
				<div class="stat bg-base-200 rounded-lg p-3">
					<div class="stat-title text-sm flex items-center gap-1">
						<span>💧 Humidity</span>
					</div>
					<div class="stat-value text-2xl text-primary text-center">
						{{ weather.humidity }}%
					</div>
				</div>

				<!-- Wind Speed -->
				<div class="stat bg-base-200 rounded-lg p-3">
					<div class="stat-title text-sm flex items-center gap-1">
						<span>💨 Wind Speed</span>
					</div>
					<div class="stat-value text-2xl text-secondary text-center">
						{{ weather.wind_speed }} m/s
					</div>
				</div>
			</div>
		</div>
	</BaseWidget>
</template>
