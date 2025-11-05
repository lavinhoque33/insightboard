/**
 * Widget Registry Initialization
 * Registers all available widget types with their configurations
 */

import { markRaw } from 'vue';
import { useWidgetStore } from '../stores/widgets';
import GitHubWidget from '../components/widgets/GitHubWidget.vue';
import WeatherWidget from '../components/widgets/WeatherWidget.vue';
import NewsWidget from '../components/widgets/NewsWidget.vue';
import CryptoWidget from '../components/widgets/CryptoWidget.vue';
import StatusWidget from '../components/widgets/StatusWidget.vue';

/**
 * Initialize and register all widgets
 * Call this during app initialization
 */
export const initializeWidgets = () => {
	const widgetStore = useWidgetStore();

	// GitHub Widget
	widgetStore.registerWidget({
		id: 'github',
		name: 'GitHub Activity',
		description: 'Display recent GitHub events for a user',
		icon: '🐙',
		component: markRaw(GitHubWidget),
		defaultConfig: {
			username: '',
		},
		configSchema: [
			{
				key: 'username',
				label: 'GitHub Username',
				type: 'text',
				required: true,
				placeholder: 'e.g., octocat',
				helpText: 'Enter a GitHub username to track their activity',
				validation: (value: string) => {
					if (
						!/^[a-z\d](?:[a-z\d]|-(?=[a-z\d])){0,38}$/i.test(value)
					) {
						return 'Invalid GitHub username format';
					}
					return null;
				},
			},
		],
		defaultSize: { w: 4, h: 3 },
		minSize: { w: 3, h: 2 },
		refreshInterval: 300, // 5 minutes
	});

	// Weather Widget
	widgetStore.registerWidget({
		id: 'weather',
		name: 'Weather',
		description: 'Display current weather for a city',
		icon: '🌤️',
		component: markRaw(WeatherWidget),
		defaultConfig: {
			city: '',
		},
		configSchema: [
			{
				key: 'city',
				label: 'City Name',
				type: 'text',
				required: true,
				placeholder: 'e.g., London, New York, Tokyo',
				helpText: 'Enter a city name to display weather information',
			},
		],
		defaultSize: { w: 3, h: 3 },
		minSize: { w: 2, h: 2 },
		refreshInterval: 600, // 10 minutes
	});

	// News Widget
	widgetStore.registerWidget({
		id: 'news',
		name: 'News Feed',
		description: 'Display latest news articles by topic',
		icon: '📰',
		component: markRaw(NewsWidget),
		defaultConfig: {
			topic: 'technology',
		},
		configSchema: [
			{
				key: 'topic',
				label: 'News Topic',
				type: 'select',
				required: true,
				options: [
					{ label: 'Technology', value: 'technology' },
					{ label: 'Business', value: 'business' },
					{ label: 'Science', value: 'science' },
					{ label: 'Health', value: 'health' },
					{ label: 'Sports', value: 'sports' },
					{ label: 'Entertainment', value: 'entertainment' },
				],
				helpText: 'Select a news category to display',
			},
		],
		defaultSize: { w: 4, h: 4 },
		minSize: { w: 3, h: 3 },
		refreshInterval: 900, // 15 minutes
	});

	// Crypto Widget
	widgetStore.registerWidget({
		id: 'crypto',
		name: 'Cryptocurrency Prices',
		description: 'Track cryptocurrency prices and changes',
		icon: '💰',
		component: markRaw(CryptoWidget),
		defaultConfig: {
			cryptoIds: 'bitcoin,ethereum,cardano',
		},
		configSchema: [
			{
				key: 'cryptoIds',
				label: 'Cryptocurrency IDs',
				type: 'text',
				required: true,
				placeholder: 'e.g., bitcoin,ethereum,cardano',
				helpText:
					'Enter comma-separated CoinGecko IDs (e.g., bitcoin, ethereum, cardano, solana)',
				validation: (value: string) => {
					if (!value.trim()) {
						return 'At least one cryptocurrency ID is required';
					}
					return null;
				},
			},
		],
		defaultSize: { w: 3, h: 4 },
		minSize: { w: 3, h: 3 },
		refreshInterval: 60, // 1 minute
	});

	// Status Monitor Widget
	widgetStore.registerWidget({
		id: 'status',
		name: 'Status Monitor',
		description: 'Monitor website uptime and response times',
		icon: '🔍',
		component: markRaw(StatusWidget),
		defaultConfig: {
			urls: '',
		},
		configSchema: [
			{
				key: 'urls',
				label: 'URLs to Monitor',
				type: 'textarea',
				required: true,
				placeholder: 'https://example.com,https://api.example.com',
				helpText:
					'Enter comma-separated URLs to monitor (include https://)',
				validation: (value: string) => {
					const urls = value.split(',').map((u) => u.trim());
					for (const url of urls) {
						try {
							new URL(url);
						} catch {
							return `Invalid URL: ${url}`;
						}
					}
					return null;
				},
			},
		],
		defaultSize: { w: 4, h: 3 },
		minSize: { w: 3, h: 2 },
		refreshInterval: 120, // 2 minutes
	});
};
