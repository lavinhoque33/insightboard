/** * Weather Widget * Displays current weather for a city */
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { fetchWeatherData, type WeatherData } from '../../api/widgets';

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

const loading = ref(false);
const error = ref<string | null>(null);
const weather = ref<WeatherData | null>(null);
const lastUpdated = ref<Date | null>(null);

let refreshTimer: number | null = null;

const loadData = async () => {
	if (!props.config.city) {
		error.value = 'City not configured';
		return;
	}

	loading.value = true;
	error.value = null;

	try {
		weather.value = await fetchWeatherData(props.config.city);
		lastUpdated.value = new Date();
	} catch (err: any) {
		error.value = err.message || 'Failed to load weather data';
		weather.value = null;
	} finally {
		loading.value = false;
	}
};

const setupAutoRefresh = () => {
	if (refreshTimer) {
		clearInterval(refreshTimer);
		refreshTimer = null;
	}

	if (props.refreshInterval && props.refreshInterval > 0) {
		refreshTimer = window.setInterval(() => {
			loadData();
		}, props.refreshInterval * 1000);
	}
};

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

onMounted(() => {
	loadData();
	setupAutoRefresh();
});

watch(
	() => props.refreshInterval,
	() => {
		setupAutoRefresh();
	},
);

watch(
	() => props.config.city,
	() => {
		loadData();
	},
);

// Cleanup on unmount
onMounted(() => {
	return () => {
		if (refreshTimer) {
			clearInterval(refreshTimer);
		}
	};
});
</script>

<template>
	<BaseWidget
		:title="`Weather: ${config.city || 'Not configured'}`"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		:refresh-interval="refreshInterval"
		@refresh="loadData"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="weather"
			class="flex flex-col items-center justify-center h-full space-y-4"
		>
			<!-- Weather Icon and Temperature -->
			<div class="text-center">
				<div class="text-6xl mb-2">
					{{ getWeatherIcon(weather.icon) }}
				</div>
				<div class="text-5xl font-bold text-gray-800">
					{{ Math.round(weather.temperature) }}°C
				</div>
				<div class="text-sm text-gray-500 mt-1">
					Feels like {{ Math.round(weather.feels_like) }}°C
				</div>
			</div>

			<!-- Weather Description -->
			<div class="text-center">
				<p class="text-lg text-gray-700 capitalize">
					{{ weather.description }}
				</p>
				<p class="text-sm text-gray-500">
					{{ weather.city }}, {{ weather.country }}
				</p>
			</div>

			<!-- Weather Details -->
			<div class="grid grid-cols-2 gap-4 w-full max-w-xs mt-4">
				<div class="bg-blue-50 rounded-lg p-3 text-center">
					<div class="text-2xl mb-1">💧</div>
					<div class="text-xs text-gray-600">Humidity</div>
					<div class="text-lg font-semibold text-gray-800">
						{{ weather.humidity }}%
					</div>
				</div>
				<div class="bg-blue-50 rounded-lg p-3 text-center">
					<div class="text-2xl mb-1">💨</div>
					<div class="text-xs text-gray-600">Wind Speed</div>
					<div class="text-lg font-semibold text-gray-800">
						{{ weather.wind_speed }} m/s
					</div>
				</div>
			</div>
		</div>
	</BaseWidget>
</template>
