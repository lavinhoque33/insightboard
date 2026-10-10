import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import LoginView from '../LoginView.vue';
import RegisterView from '../RegisterView.vue';
import { useAuthStore } from '../../stores/auth';

const stubs = { 'router-link': true };

describe('auth forms field errors', () => {
	beforeEach(() => {
		setActivePinia(createPinia());
	});

	it.each([
		['login', LoginView],
		['register', RegisterView],
	])('%s shows 422 details next to the fields', async (_name, view) => {
		const store = useAuthStore();
		const wrapper = mount(view, { global: { stubs } });
		store.fieldErrors = [
			{ field: 'email', message: 'must be a valid email address' },
			{ field: 'password', message: 'must be at least 8 characters' },
		];
		store.error = 'Please correct the highlighted fields.';
		await wrapper.vm.$nextTick();

		const email = wrapper.get('#email');
		expect(email.attributes('aria-invalid')).toBe('true');
		expect(email.attributes('aria-describedby')).toBe('email-error');
		expect(wrapper.get('#email-error').text()).toBe(
			'must be a valid email address',
		);
		expect(wrapper.get('#password').attributes('aria-describedby')).toBe(
			'password-error',
		);
		expect(wrapper.get('#password-error').text()).toContain('8 characters');
	});

	it('register shows a 409 message visibly', async () => {
		const store = useAuthStore();
		const wrapper = mount(RegisterView, { global: { stubs } });
		store.error = 'Email already registered';
		await wrapper.vm.$nextTick();

		expect(wrapper.get('[role="alert"]').text()).toContain(
			'Email already registered',
		);
	});
});
