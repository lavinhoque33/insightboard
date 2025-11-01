/** * Crypto Widget * Displays cryptocurrency prices with price change
indicators */
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { fetchCryptoData, type CryptoPrice } from '../../api/widgets';

interface Props {
	config: {
		cryptoIds: string; // Comma-separated crypto IDs (e.g., "bitcoin,ethereum,cardano")
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
const cryptos = ref<CryptoPrice[]>([]);
const lastUpdated = ref<Date | null>(null);

let refreshTimer: number | null = null;

const loadData = async () => {
	if (!props.config.cryptoIds) {
		error.value = 'Cryptocurrency IDs not configured';
		return;
	}

	loading.value = true;
	error.value = null;

	try {
		cryptos.value = await fetchCryptoData(props.config.cryptoIds);
		lastUpdated.value = new Date();
	} catch (err: any) {
		error.value = err.message || 'Failed to load crypto data';
		cryptos.value = [];
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

const formatPrice = (price: number): string => {
	if (price >= 1) {
		return `$${price.toLocaleString(undefined, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		})}`;
	}
	return `$${price.toFixed(6)}`;
};

const formatMarketCap = (marketCap: number): string => {
	if (marketCap >= 1e12) return `$${(marketCap / 1e12).toFixed(2)}T`;
	if (marketCap >= 1e9) return `$${(marketCap / 1e9).toFixed(2)}B`;
	if (marketCap >= 1e6) return `$${(marketCap / 1e6).toFixed(2)}M`;
	return `$${marketCap.toLocaleString()}`;
};

const getPriceChangeClass = (change: number): string => {
	if (change > 0) return 'text-green-600';
	if (change < 0) return 'text-red-600';
	return 'text-gray-600';
};

const getPriceChangeIcon = (change: number): string => {
	if (change > 0) return '▲';
	if (change < 0) return '▼';
	return '━';
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
	() => props.config.cryptoIds,
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
		title="Cryptocurrency Prices"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		:refresh-interval="refreshInterval"
		@refresh="loadData"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="cryptos.length === 0 && !loading && !error"
			class="text-center text-gray-500 py-8"
		>
			<p class="text-sm">No cryptocurrency data found</p>
		</div>

		<div v-else class="space-y-3">
			<div
				v-for="crypto in cryptos"
				:key="crypto.id"
				class="bg-gradient-to-r from-gray-50 to-white p-4 rounded-lg border border-gray-200 hover:shadow-md transition-shadow"
			>
				<!-- Crypto Header -->
				<div class="flex items-center justify-between mb-3">
					<div class="flex items-center space-x-2">
						<span class="font-bold text-lg text-gray-800">{{
							crypto.symbol.toUpperCase()
						}}</span>
						<span class="text-sm text-gray-500">{{
							crypto.name
						}}</span>
					</div>
					<div class="text-right">
						<div class="font-bold text-lg text-gray-800">
							{{ formatPrice(crypto.current_price) }}
						</div>
					</div>
				</div>

				<!-- Price Change -->
				<div class="flex items-center justify-between text-sm">
					<div
						:class="
							getPriceChangeClass(
								crypto.price_change_percentage_24h,
							)
						"
						class="font-semibold flex items-center space-x-1"
					>
						<span>{{
							getPriceChangeIcon(
								crypto.price_change_percentage_24h,
							)
						}}</span>
						<span
							>{{
								Math.abs(
									crypto.price_change_percentage_24h,
								).toFixed(2)
							}}%</span
						>
						<span class="text-xs text-gray-500">(24h)</span>
					</div>
					<div class="text-gray-600">
						<span class="text-xs">MCap: </span>
						<span class="font-medium">{{
							formatMarketCap(crypto.market_cap)
						}}</span>
					</div>
				</div>

				<!-- Volume -->
				<div class="mt-2 pt-2 border-t border-gray-100">
					<div
						class="flex items-center justify-between text-xs text-gray-500"
					>
						<span>24h Volume</span>
						<span class="font-medium">{{
							formatMarketCap(crypto.total_volume)
						}}</span>
					</div>
				</div>
			</div>
		</div>
	</BaseWidget>
</template>
