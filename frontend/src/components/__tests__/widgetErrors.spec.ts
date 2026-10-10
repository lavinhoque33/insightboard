import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import CryptoWidget from '../widgets/CryptoWidget.vue';
import StatusWidget from '../widgets/StatusWidget.vue';
import { fetchCryptoData, fetchStatusData } from '../../api/widgets';

vi.mock('../../api/widgets', () => ({
	fetchCryptoData: vi.fn(),
	fetchStatusData: vi.fn(),
}));

describe('widget error and null rendering', () => {
	beforeEach(() => {
		vi.mocked(fetchCryptoData).mockReset();
		vi.mocked(fetchStatusData).mockReset();
	});

	it('renders null crypto numbers as a dash', async () => {
		vi.mocked(fetchCryptoData).mockResolvedValue([
			{
				id: 'obscure',
				symbol: 'obs',
				name: 'Obscure',
				current_price: null,
				price_change_percentage_24h: null,
				market_cap: null,
				total_volume: null,
			},
		]);
		const wrapper = mount(CryptoWidget, {
			props: { config: { cryptoIds: 'obscure' } },
		});
		await flushPromises();

		expect(wrapper.text()).toContain('OBS');
		expect(wrapper.text()).toContain('—');
		expect(wrapper.text()).not.toContain('NaN');
	});

	it('shows provider-key text for a 503', async () => {
		vi.mocked(fetchCryptoData).mockRejectedValue({ status: 503, message: 'x' });
		const wrapper = mount(CryptoWidget, {
			props: { config: { cryptoIds: 'bitcoin' } },
		});
		await flushPromises();

		expect(wrapper.text()).toContain('provider key is not configured');
	});

	it('renders generic per-URL error classes readably', async () => {
		vi.mocked(fetchStatusData).mockResolvedValue([
			{
				url: 'http://10.0.0.1',
				status: 0,
				response_time_ms: 0,
				error: 'blocked_address',
			},
		]);
		const wrapper = mount(StatusWidget, {
			props: { config: { urls: 'http://10.0.0.1' } },
		});
		await flushPromises();

		expect(wrapper.text()).toContain('Blocked');
		expect(wrapper.text()).not.toContain('blocked_address');
	});
});
