<!-- Crypto Widget: cryptocurrency prices with 24h change -->
<script setup lang="ts">
import { computed } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import AppIcon from '../ui/AppIcon.vue';
import { useWidgetData } from '../../composables/useWidgetData';
import { fetchCryptoData } from '../../api/widgets';

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

const { data, loading, error, lastUpdated, reload } = useWidgetData({
	source: () => props.config.cryptoIds,
	fetch: fetchCryptoData,
	refreshInterval: () => props.refreshInterval,
	notConfigured: 'Cryptocurrency IDs not configured',
	failure: 'Failed to load crypto data',
});
const cryptos = computed(() => data.value ?? []);

const NO_VALUE = '—';

const formatPrice = (price: number | null): string => {
	if (price === null) return NO_VALUE;
	if (price >= 1) {
		return `$${price.toLocaleString(undefined, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		})}`;
	}
	return `$${price.toFixed(6)}`;
};

const formatCompact = (value: number | null): string => {
	if (value === null) return NO_VALUE;
	if (value >= 1e12) return `$${(value / 1e12).toFixed(2)}T`;
	if (value >= 1e9) return `$${(value / 1e9).toFixed(2)}B`;
	if (value >= 1e6) return `$${(value / 1e6).toFixed(2)}M`;
	return `$${value.toLocaleString()}`;
};

type Trend = 'up' | 'down' | 'flat';

const trend = (change: number | null): Trend => {
	if (change === null || change === 0) return 'flat';
	return change > 0 ? 'up' : 'down';
};

const TREND_BADGE: Record<Trend, string> = {
	up: 'bg-success/12 text-success',
	down: 'bg-error/12 text-error',
	flat: 'bg-base-200 text-base-content/70',
};

const TREND_ICON = {
	up: 'arrow-up-right',
	down: 'arrow-down-right',
	flat: 'minus',
} as const;

// Signed text, so the direction never depends on colour alone
const formatChange = (change: number | null): string => {
	if (change === null) return NO_VALUE;
	const sign = change > 0 ? '+' : change < 0 ? '−' : '';
	return `${sign}${Math.abs(change).toFixed(2)}%`;
};
</script>

<template>
	<BaseWidget
		type="crypto"
		title="Cryptocurrency Prices"
		:loading="loading"
		:error="error"
		:last-updated="lastUpdated"
		@refresh="reload"
		@configure="emit('configure')"
		@remove="emit('remove')"
	>
		<div
			v-if="cryptos.length === 0 && !loading && !error"
			class="flex h-full flex-col items-center justify-center gap-2 py-6 text-center"
		>
			<AppIcon name="coins" class="size-7 text-base-content/40" />
			<p class="text-sm text-base-content/70">No cryptocurrency data found</p>
		</div>

		<ul v-else class="flex flex-col divide-y divide-base-300/70">
			<li
				v-for="crypto in cryptos"
				:key="crypto.id"
				class="flex flex-col gap-2 py-2.5 first:pt-0 last:pb-0"
			>
				<div class="flex items-center gap-3">
					<span
						class="grid size-9 shrink-0 place-items-center rounded-full bg-linear-to-br from-widget-crypto/25 to-primary/15 text-[11px] font-bold uppercase tracking-wide text-base-content ring-1 ring-inset ring-base-content/5"
						aria-hidden="true"
					>
						{{ crypto.symbol.slice(0, 4) }}
					</span>
					<div class="min-w-0 flex-1">
						<p class="truncate text-sm font-semibold">{{ crypto.name }}</p>
						<p class="text-xs font-medium text-base-content/70">
							{{ crypto.symbol.toUpperCase() }}
						</p>
					</div>
					<div class="flex flex-col items-end gap-1">
						<p class="text-sm font-semibold tabular-nums @xs:text-base">
							{{ formatPrice(crypto.current_price) }}
						</p>
						<p
							class="inline-flex items-center gap-0.5 rounded-md px-1.5 py-0.5 text-xs font-semibold tabular-nums"
							:class="TREND_BADGE[trend(crypto.price_change_percentage_24h)]"
						>
							<AppIcon
								:name="TREND_ICON[trend(crypto.price_change_percentage_24h)]"
								class="size-3"
							/>
							{{ formatChange(crypto.price_change_percentage_24h) }}
							<span class="sr-only">in 24 hours</span>
						</p>
					</div>
				</div>
				<dl
					class="flex flex-wrap gap-x-4 gap-y-0.5 pl-12 text-[11px] text-base-content/70"
				>
					<div class="flex gap-1">
						<dt>Market cap</dt>
						<dd class="font-medium tabular-nums text-base-content">
							{{ formatCompact(crypto.market_cap) }}
						</dd>
					</div>
					<div class="flex gap-1">
						<dt>24h volume</dt>
						<dd class="font-medium tabular-nums text-base-content">
							{{ formatCompact(crypto.total_volume) }}
						</dd>
					</div>
				</dl>
			</li>
		</ul>
	</BaseWidget>
</template>
