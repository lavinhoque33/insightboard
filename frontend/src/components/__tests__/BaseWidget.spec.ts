import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
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
});
