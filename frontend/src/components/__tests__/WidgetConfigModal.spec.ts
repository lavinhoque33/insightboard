import { mount } from '@vue/test-utils';
import { afterEach, describe, expect, it } from 'vitest';
import WidgetConfigModal from '../WidgetConfigModal.vue';
import type { WidgetConfigField } from '../../stores/widgets';

const schema: WidgetConfigField[] = [
	{
		key: 'city',
		label: 'City Name',
		type: 'text',
		required: true,
		validation: (value) => (value.length < 2 ? 'City name is too short' : null),
	},
	{
		key: 'topic',
		label: 'News Topic',
		type: 'select',
		required: true,
		options: [
			{ label: 'Technology', value: 'technology' },
			{ label: 'Science', value: 'science' },
		],
	},
];

const mountModal = (initialConfig: Record<string, unknown>) =>
	mount(WidgetConfigModal, {
		props: { widgetTypeName: 'Weather', configSchema: schema, initialConfig },
		attachTo: document.body,
	});

describe('WidgetConfigModal', () => {
	afterEach(() => {
		document.body.innerHTML = '';
	});

	it('pre-fills the current configuration of an existing widget', () => {
		const wrapper = mountModal({ city: 'London', topic: 'science' });

		expect(
			(wrapper.get('#widget-config-city').element as HTMLInputElement).value,
		).toBe('London');
		expect(
			(wrapper.get('#widget-config-topic').element as HTMLSelectElement).value,
		).toBe('science');
		wrapper.unmount();
	});

	it('pre-fills registry defaults for a new widget', () => {
		const wrapper = mountModal({ city: '', topic: 'technology' });

		expect(
			(wrapper.get('#widget-config-topic').element as HTMLSelectElement).value,
		).toBe('technology');
		wrapper.unmount();
	});

	it('saving unchanged keeps every field, including keys the form does not show', async () => {
		const wrapper = mountModal({
			city: 'London',
			topic: 'science',
			units: 'metric',
		});

		await wrapper.get('form').trigger('submit');

		expect(wrapper.emitted('save')).toEqual([
			[{ city: 'London', topic: 'science', units: 'metric' }],
		]);
		expect(wrapper.emitted('close')).toHaveLength(1);
		wrapper.unmount();
	});

	it('trims input and passes the entered string to the validator', async () => {
		const wrapper = mountModal({ city: 'London', topic: 'science' });

		await wrapper.get('#widget-config-city').setValue('  x ');
		await wrapper.get('form').trigger('submit');

		expect(wrapper.emitted('save')).toBeUndefined();
		const input = wrapper.get('#widget-config-city');
		expect(input.attributes('aria-invalid')).toBe('true');
		expect(wrapper.get('#widget-config-city-error').text()).toBe(
			'City name is too short',
		);

		await input.setValue('  Paris ');
		await wrapper.get('form').trigger('submit');
		expect(wrapper.emitted('save')?.[0]).toEqual([
			{ city: 'Paris', topic: 'science' },
		]);
		wrapper.unmount();
	});

	it('requires required fields', async () => {
		const wrapper = mountModal({ city: '', topic: 'technology' });

		await wrapper.get('form').trigger('submit');

		expect(wrapper.emitted('save')).toBeUndefined();
		expect(wrapper.get('#widget-config-city-error').text()).toBe(
			'City Name is required',
		);
		wrapper.unmount();
	});

	it('is a labelled modal dialog that takes focus and closes on Escape', async () => {
		const opener = document.createElement('button');
		document.body.appendChild(opener);
		opener.focus();

		const wrapper = mountModal({ city: 'London', topic: 'science' });
		const dialog = wrapper.get('[role="dialog"]');
		expect(dialog.attributes('aria-modal')).toBe('true');
		expect(
			document.getElementById(dialog.attributes('aria-labelledby') ?? '')
				?.textContent,
		).toContain('Configure Weather');
		expect(document.activeElement?.id).toBe('widget-config-city');

		await dialog.trigger('keydown', { key: 'Escape' });
		expect(wrapper.emitted('close')).toHaveLength(1);

		wrapper.unmount();
		expect(document.activeElement).toBe(opener);
	});

	it('keeps Tab focus inside the dialog', async () => {
		const wrapper = mountModal({ city: 'London', topic: 'science' });
		const dialog = wrapper.get('[role="dialog"]');
		const buttons = dialog.findAll('button');
		const last = buttons[buttons.length - 1]!.element as HTMLElement;
		last.focus();

		await dialog.trigger('keydown', { key: 'Tab' });

		expect(document.activeElement?.getAttribute('aria-label')).toBe('Close');
		wrapper.unmount();
	});
});
