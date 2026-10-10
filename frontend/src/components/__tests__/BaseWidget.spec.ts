import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import BaseWidget from '../BaseWidget.vue';

describe('BaseWidget', () => {
	it('emits configure when Configure is clicked from options menu', async () => {
		const wrapper = mount(BaseWidget, {
			props: {
				title: 'Status Monitor',
				loading: false,
				error: null,
			},
		});

		await wrapper.get('[data-testid="widget-options-btn"]').trigger('click');

		const configureButton = wrapper.get('[data-testid="widget-configure-btn"]');
		expect(configureButton.attributes('disabled')).toBeUndefined();

		await configureButton.trigger('click');

		expect(wrapper.emitted('configure')).toHaveLength(1);
	});

	it('keeps the relative update time current', async () => {
		vi.useFakeTimers();
		const wrapper = mount(BaseWidget, {
			props: { title: 'Weather', lastUpdated: new Date() },
		});
		expect(wrapper.text()).toContain('Just now');

		await vi.advanceTimersByTimeAsync(3 * 60_000);

		expect(wrapper.text()).toContain('3m ago');
		wrapper.unmount();
		vi.useRealTimers();
	});

	it('names its icon buttons and exposes the menu state', async () => {
		const wrapper = mount(BaseWidget, {
			props: { title: 'Weather' },
			attachTo: document.body,
		});
		const options = wrapper.get('[data-testid="widget-options-btn"]');
		expect(options.attributes('aria-label')).toBe('Options for Weather');
		expect(options.attributes('aria-expanded')).toBe('false');
		expect(wrapper.find('[aria-label="Refresh Weather"]').exists()).toBe(true);

		await options.trigger('click');
		expect(options.attributes('aria-expanded')).toBe('true');

		await wrapper
			.get('[data-menu-panel]')
			.trigger('keydown', { key: 'Escape' });
		expect(options.attributes('aria-expanded')).toBe('false');
		expect(document.activeElement).toBe(options.element);
		wrapper.unmount();
	});
});
