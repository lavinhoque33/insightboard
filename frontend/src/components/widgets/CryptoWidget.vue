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
		<!-- Empty State -->
		<div
			v-if="cryptos.length === 0 && !loading && !error"
			class="flex flex-col items-center justify-center py-8 text-center"
		>
			<div class="text-4xl mb-3">💰</div>
			<p class="text-sm text-base-content/60">
				No cryptocurrency data found
			</p>
		</div>

		<!-- Crypto List -->
		<div v-else class="space-y-3">
			<div
				v-for="crypto in cryptos"
				:key="crypto.id"
				class="card card-bordered border-base-300 bg-gradient-to-br from-base-100 to-base-200 hover:shadow-lg transition-all duration-300 group"
			>
				<div class="card-body p-4">
					<!-- Header: Symbol and Price -->
					<div class="flex items-center justify-between mb-3">
						<div class="flex items-center gap-2">
							<div class="badge badge-primary gap-1 font-bold">
								{{ crypto.symbol.toUpperCase() }}
							</div>
							<span class="text-sm text-base-content/60">
								{{ crypto.name }}
							</span>
						</div>
						<div class="text-right">
							<div
								class="text-xl font-bold text-transparent bg-gradient-to-r from-primary to-secondary bg-clip-text"
							>
								{{ formatPrice(crypto.current_price) }}
							</div>
						</div>
					</div>

					<!-- Divider -->
					<div class="divider my-1"></div>

					<!-- Price Change and Market Cap -->
					<div class="flex items-center justify-between text-sm">
						<!-- Price Change Badge -->
						<div
							class="badge"
							:class="{
								'badge-success':
									crypto.price_change_percentage_24h > 0,
								'badge-error':
									crypto.price_change_percentage_24h < 0,
								'badge-ghost':
									crypto.price_change_percentage_24h === 0,
							}"
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
							<span class="text-xs">(24h)</span>
						</div>

						<!-- Market Cap -->
						<div class="text-base-content/70">
							<span class="text-xs">MCap: </span>
							<span class="font-semibold">{{
								formatMarketCap(crypto.market_cap)
							}}</span>
						</div>
					</div>

					<!-- Volume -->
					<div class="mt-2 pt-2 border-t border-base-300">
						<div
							class="flex items-center justify-between text-xs text-base-content/60"
						>
							<span>24h Volume</span>
							<span class="font-semibold text-base-content">{{
								formatMarketCap(crypto.total_volume)
							}}</span>
						</div>
					</div>
				</div>
			</div>
		</div>
	</BaseWidget>
</template>
