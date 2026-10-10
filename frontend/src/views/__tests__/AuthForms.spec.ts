import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import { defineComponent } from 'vue';
import { createMemoryHistory, createRouter } from 'vue-router';
import LoginView from '../LoginView.vue';
import RegisterView from '../RegisterView.vue';
import { useAuthStore } from '../../stores/auth';

const Stub = defineComponent({ render: () => null });

// The forms read the route (redirect target, expired-session notice)
const makeRouter = () =>
	createRouter({
		history: createMemoryHistory(),
		routes: [
			{ path: '/login', name: 'login', component: Stub },
			{ path: '/register', name: 'register', component: Stub },
			{ path: '/dashboards', name: 'dashboards', component: Stub },
		],
	});

describe('auth forms field errors', () => {
	beforeEach(() => {
		setActivePinia(createPinia());
	});

	it.each([
		['login', LoginView],
		['register', RegisterView],
	])('%s shows 422 details next to the fields', async (_name, view) => {
		const store = useAuthStore();
		const wrapper = mount(view, { global: { plugins: [makeRouter()] } });
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
		expect(
			wrapper.get('#password').attributes('aria-describedby')?.split(' '),
		).toContain('password-error');
		expect(wrapper.get('#password-error').text()).toContain('8 characters');
	});

	it('register shows a 409 message visibly', async () => {
		const store = useAuthStore();
		const wrapper = mount(RegisterView, {
			global: { plugins: [makeRouter()] },
		});
		store.error = 'Email already registered';
		await wrapper.vm.$nextTick();

		expect(wrapper.get('[role="alert"]').text()).toContain(
			'Email already registered',
		);
	});

	it.each([
		['login', LoginView],
		['register', RegisterView],
	])(
		'%s keeps focus in the field while the request runs',
		async (_name, view) => {
			const store = useAuthStore();
			const wrapper = mount(view, {
				global: { plugins: [makeRouter()] },
				attachTo: document.body,
			});
			const email = wrapper.get('#email');
			(email.element as HTMLInputElement).focus();

			store.loading = true;
			await wrapper.vm.$nextTick();

			expect(email.attributes('disabled')).toBeUndefined();
			expect(document.activeElement).toBe(email.element);
			wrapper.unmount();
		},
	);
});
