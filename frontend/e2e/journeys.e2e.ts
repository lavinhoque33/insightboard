import { expect, test } from '@playwright/test';
import type { Page, Request, Route } from '@playwright/test';

// User journeys against the real backend, whose providers point at the local fixture
// (see playwright.config.ts). Selectors are roles, labels, and visible text only.

const PASSWORD = 'e2e-Password-123';
const STATUS_URL = 'http://127.0.0.1:18091/';

function uniqueEmail(): string {
	const suffix = Math.random().toString(36).slice(2, 8);
	return `e2e-${Date.now()}-${suffix}@example.com`;
}

async function register(page: Page, email: string) {
	await page.goto('/register');
	await page.getByLabel('Email Address').fill(email);
	// The label text carries a required marker, so match the accessible name exactly
	await page
		.getByRole('textbox', { name: 'Password', exact: true })
		.fill(PASSWORD);
	await page.getByLabel('Confirm Password').fill(PASSWORD);
	await page.getByRole('button', { name: 'Create Account' }).click();
	await expect(
		page.getByRole('heading', { level: 1, name: 'My Dashboards' }),
	).toBeVisible();
}

async function login(page: Page, email: string) {
	await page.getByLabel('Email Address').fill(email);
	await page.getByLabel('Password').fill(PASSWORD);
	await page.getByRole('button', { name: 'Sign In' }).click();
	await expect(
		page.getByRole('heading', { level: 1, name: 'My Dashboards' }),
	).toBeVisible();
}

async function logout(page: Page) {
	await page.getByRole('button', { name: 'Account menu' }).click();
	await page.getByRole('link', { name: 'Logout' }).click();
	await expect(
		page.getByRole('heading', { level: 1, name: 'Welcome Back' }),
	).toBeVisible();
}

/** Creates a dashboard from the list page and returns its id from the editor URL. */
async function createDashboard(page: Page): Promise<string> {
	await page.getByRole('button', { name: 'New Dashboard' }).click();
	await expect(page).toHaveURL(/\/dashboards\/[^/]+$/);
	await expect(
		page.getByRole('heading', { level: 1, name: 'New Dashboard' }),
	).toBeVisible();
	return new URL(page.url()).pathname.split('/').pop() ?? '';
}

async function addWidget(page: Page, typeName: RegExp) {
	await page.getByRole('button', { name: 'Add Widget', exact: true }).click();
	await page.getByRole('button', { name: typeName }).click();
}

async function openWidgetConfig(page: Page, title: string) {
	await page.getByRole('button', { name: `Options for ${title}` }).click();
	await page.getByRole('button', { name: 'Configure', exact: true }).click();
}

const allSaved = (page: Page) => page.getByText('All changes saved');

test('register, build, configure, rename, reload, leave, and log out', async ({
	page,
}) => {
	const email = uniqueEmail();
	// Unique per run: the weather cache key is the city, so no earlier entry can answer
	const city = `E2E City ${Date.now()}`;
	const weatherTitle = `Weather: ${city}`;
	const dashboardName = `E2E Dashboard ${Date.now()}`;

	await test.step('register, log out, and log back in', async () => {
		await register(page, email);
		await logout(page);
		await login(page, email);
	});

	const id = await test.step('create a dashboard', () => createDashboard(page));

	const puts: Request[] = [];
	page.on('request', (request) => {
		if (
			request.method() === 'PUT' &&
			new URL(request.url()).pathname === `/api/dashboards/${id}`
		) {
			puts.push(request);
		}
	});

	await test.step('add a Weather widget served by the provider fixture', async () => {
		await addWidget(page, /^Weather\b/);
		const dialog = page.getByRole('dialog', { name: 'Configure Weather' });
		// Opening from the picker moves focus into the dialog
		await expect(dialog.getByLabel('City Name')).toBeFocused();
		await dialog.getByLabel('City Name').fill(city);
		await dialog.getByRole('button', { name: 'Save Changes' }).click();
		await expect(dialog).toBeHidden();
		await expect(
			page.getByRole('heading', { level: 3, name: weatherTitle }),
		).toBeVisible();
		await expect(page.getByText('fixture clear sky')).toBeVisible();
		await expect(page.getByText(`${city}, FX`)).toBeVisible();
	});

	await test.step('add a Status widget for the allowlisted loopback fixture', async () => {
		await addWidget(page, /^Status Monitor\b/);
		const dialog = page.getByRole('dialog', {
			name: 'Configure Status Monitor',
		});
		await dialog.getByLabel('URLs to Monitor').fill(STATUS_URL);
		await dialog.getByRole('button', { name: 'Save Changes' }).click();
		await expect(dialog).toBeHidden();
		await expect(
			page.getByRole('heading', { level: 3, name: 'Status Monitor' }),
		).toBeVisible();
		await expect(page.getByText('1 healthy')).toBeVisible();
		await expect(page.getByText('Healthy', { exact: true })).toBeVisible();
		await expect(allSaved(page)).toBeVisible();
	});

	await test.step('configure existing widgets: the form is prefilled (drill 12)', async () => {
		await openWidgetConfig(page, weatherTitle);
		const weatherDialog = page.getByRole('dialog', {
			name: 'Configure Weather',
		});
		await expect(weatherDialog.getByLabel('City Name')).toHaveValue(city);
		await expect(weatherDialog.getByLabel('City Name')).toBeFocused();
		await weatherDialog.getByRole('button', { name: 'Save Changes' }).click();
		await expect(weatherDialog).toBeHidden();
		// Saving unchanged fields keeps the configuration
		await expect(
			page.getByRole('heading', { level: 3, name: weatherTitle }),
		).toBeVisible();

		await openWidgetConfig(page, 'Status Monitor');
		const statusDialog = page.getByRole('dialog', {
			name: 'Configure Status Monitor',
		});
		await expect(statusDialog.getByLabel('URLs to Monitor')).toHaveValue(
			STATUS_URL,
		);
		await statusDialog.getByRole('button', { name: 'Cancel' }).click();
		await expect(statusDialog).toBeHidden();
	});

	await test.step('keyboard: the dialog takes focus, Escape closes it and returns focus', async () => {
		const options = page.getByRole('button', {
			name: `Options for ${weatherTitle}`,
		});
		await options.focus();
		await page.keyboard.press('Enter');
		await page.getByRole('button', { name: 'Configure', exact: true }).focus();
		await page.keyboard.press('Enter');
		const dialog = page.getByRole('dialog', { name: 'Configure Weather' });
		await expect(dialog.getByLabel('City Name')).toBeFocused();
		await page.keyboard.press('Escape');
		await expect(dialog).toBeHidden();
		await expect(options).toBeFocused();
	});

	await test.step('rename the dashboard', async () => {
		await page.getByRole('button', { name: 'Rename dashboard' }).click();
		await page.getByLabel('Dashboard name').fill(dashboardName);
		await page.getByRole('button', { name: 'Save', exact: true }).click();
		await expect(
			page.getByRole('heading', { level: 1, name: dashboardName }),
		).toBeVisible();
		await expect(allSaved(page)).toBeVisible();
	});

	await test.step('reload: everything persisted', async () => {
		await page.reload();
		await expect(
			page.getByRole('heading', { level: 1, name: dashboardName }),
		).toBeVisible();
		await expect(
			page.getByRole('heading', { level: 3, name: weatherTitle }),
		).toBeVisible();
		await expect(
			page.getByRole('heading', { level: 3, name: 'Status Monitor' }),
		).toBeVisible();
		await expect(page.getByText('fixture clear sky')).toBeVisible();
		await expect(page.getByText('1 healthy')).toBeVisible();
	});

	await test.step('edit, then leave the editor: no PUT after leaving (drill 14)', async () => {
		await openWidgetConfig(page, weatherTitle);
		const dialog = page.getByRole('dialog', { name: 'Configure Weather' });
		await dialog.getByLabel('City Name').fill(`${city} Two`);
		await dialog.getByRole('button', { name: 'Save Changes' }).click();
		await expect(dialog).toBeHidden();
		// Leave inside the auto-save debounce: the editor saves once, before leaving
		await page.getByRole('link', { name: 'Back to dashboards' }).click();
		await expect(
			page.getByRole('heading', { level: 1, name: 'My Dashboards' }),
		).toBeVisible();
		const putsWhenLeft = puts.length;
		// Longer than the editor's 2 s auto-save debounce
		await page.waitForTimeout(3_000);
		expect(puts.length).toBe(putsWhenLeft);

		// The edit made before leaving was saved, not lost
		const card = page
			.getByRole('listitem')
			.filter({ has: page.getByRole('heading', { name: dashboardName }) });
		await card.getByRole('link', { name: `Open ${dashboardName}` }).click();
		await expect(
			page.getByRole('heading', { level: 3, name: `${weatherTitle} Two` }),
		).toBeVisible();
	});

	await test.step('log out: the session ends', async () => {
		await logout(page);
		await page.goto('/dashboards');
		await expect(page).toHaveURL(/\/login/);
		await expect(
			page.getByRole('heading', { level: 1, name: 'Welcome Back' }),
		).toBeVisible();
	});
});

test('a failed save shows an error with Retry and loses nothing (drill 13)', async ({
	page,
}) => {
	await register(page, uniqueEmail());
	const id = await createDashboard(page);
	const renamed = `Retry Dashboard ${Date.now()}`;

	// The browser fails every save of this dashboard until the route is removed
	const failPuts = (route: Route) =>
		route.request().method() === 'PUT'
			? route.fulfill({ status: 500, json: { error: 'e2e forced failure' } })
			: route.fallback();
	await page.route(`**/api/dashboards/${id}`, failPuts);

	await page.getByRole('button', { name: 'Rename dashboard' }).click();
	await page.getByLabel('Dashboard name').fill(renamed);
	await page.getByRole('button', { name: 'Save', exact: true }).click();

	const failure = page
		.getByRole('alert')
		.filter({ hasText: 'Your changes are not saved' });
	await expect(failure).toBeVisible();
	await expect(
		page.getByRole('heading', { level: 1, name: renamed }),
	).toBeVisible();

	await page.unroute(`**/api/dashboards/${id}`, failPuts);
	const saved = page.waitForResponse(
		(response) =>
			response.request().method() === 'PUT' &&
			new URL(response.url()).pathname === `/api/dashboards/${id}` &&
			response.status() === 200,
	);
	await failure.getByRole('button', { name: 'Retry' }).click();
	await saved;
	await expect(failure).toBeHidden();
	await expect(allSaved(page)).toBeVisible();

	await page.reload();
	await expect(
		page.getByRole('heading', { level: 1, name: renamed }),
	).toBeVisible();
});
