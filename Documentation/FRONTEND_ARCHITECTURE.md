# Frontend Architecture Guide

**InsightBoard Frontend - Vue 3 + TypeScript + Pinia + TailwindCSS**

> This guide explains the frontend architecture from the ground up, assuming no prior knowledge. Whether you're new to Vue or an experienced developer, you'll understand how everything fits together.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Overview](#overview)
3. [Technology Stack Explained](#technology-stack-explained)
4. [Project Structure](#project-structure)
5. [Type System (TypeScript)](#type-system-typescript)
6. [API Layer Architecture](#api-layer-architecture)
7. [State Management with Pinia](#state-management-with-pinia)
8. [Routing System](#routing-system)
9. [Component Architecture](#component-architecture)
10. [Styling with TailwindCSS](#styling-with-tailwindcss)
11. [Authentication Flow](#authentication-flow)
12. [Data Flow Diagrams](#data-flow-diagrams)
13. [Best Practices & Patterns](#best-practices--patterns)
14. [Common Scenarios](#common-scenarios)
15. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Critical Setup Requirements

Before the frontend can run, **three configuration files must be properly set up**. Missing any of these will cause the app to crash or fail silently.

#### 1. Application Initialization (`main.ts`)

The `main.ts` file is the entry point that initializes Vue, Pinia, and Router.

**✅ Correct Setup:**

```typescript
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import router from './router';
import App from './App.vue';
import './style.css';

const app = createApp(App);
const pinia = createPinia();

// CRITICAL: Install plugins before mounting
app.use(pinia); // ← State management
app.use(router); // ← Navigation
app.mount('#app');
```

**❌ Common Mistake:**

```typescript
// DON'T: This skips plugin installation
createApp(App).mount('#app'); // ← Pinia and Router won't work!
```

**Why This Matters:** Without `app.use(pinia)` and `app.use(router)`, components cannot access stores or routing, causing errors like:

-   `Cannot access 'useAuthStore' before initialization`
-   `Cannot read property '$router' of undefined`

---

#### 2. Development Proxy (`vite.config.ts`)

The Vite config must proxy API requests to the backend to avoid CORS errors.

**✅ Correct Setup:**

```typescript
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
	plugins: [vue()],
	server: {
		proxy: {
			'/api': {
				target: 'http://localhost:8080', // Backend server
				changeOrigin: true,
			},
		},
	},
});
```

**Why This Matters:** Without the proxy:

-   Frontend runs on `localhost:5173`
-   Backend runs on `localhost:8080`
-   Browser blocks cross-origin requests (CORS error)
-   All API calls return 404 or CORS errors

**How It Works:**

```
Request: POST /api/auth/login
         ↓ (Vite intercepts)
Forwarded to: http://localhost:8080/api/auth/login
         ↓ (Backend responds)
Response returned to frontend
```

---

#### 3. TailwindCSS Integration (`style.css`)

TailwindCSS requires specific directives at the top of your CSS file.

**✅ Correct Setup:**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Your custom styles below */
body {
	margin: 0;
	font-family: system-ui, sans-serif;
}
```

**❌ Common Mistake:**

```css
/* DON'T: Missing @tailwind directives */
body {
	background: white;
}
/* Tailwind classes like 'bg-blue-600' won't work! */
```

**Why This Matters:** The `@tailwind` directives tell PostCSS to generate Tailwind's utility classes. Without them:

-   `class="bg-blue-600"` has no effect
-   All Tailwind utilities are missing
-   Only custom CSS works

---

### Running the Frontend

```bash
# Install dependencies
npm install

# Start dev server (runs on localhost:5173)
npm run dev

# Ensure backend is also running (localhost:8080)
# Otherwise API calls will fail
```

**Verify Setup:**

1. Dev server starts without errors ✅
2. Navigate to `http://localhost:5173` ✅
3. Console has no red errors ✅
4. Tailwind styles are visible (colors, spacing) ✅

---

## Overview

### What is InsightBoard Frontend?

InsightBoard's frontend is a **Single Page Application (SPA)** built with Vue 3. Think of it as a dynamic web application that:

-   Loads once and never refreshes the entire page
-   Fetches data from the backend API as needed
-   Updates the UI instantly without page reloads
-   Provides a smooth, app-like experience

### Architecture Philosophy

```
┌─────────────────────────────────────────────────────┐
│                   User Interface                     │
│              (Vue Components + HTML)                 │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│              State Management                        │
│            (Pinia Stores: auth, dashboard)          │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                 API Layer                            │
│          (Axios client + interceptors)              │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ HTTP Requests
                  ▼
           Backend REST API
```

**Key Principles:**

1. **Separation of Concerns** - Each layer has one job
2. **Type Safety** - TypeScript catches errors before runtime
3. **Centralized State** - One source of truth for app data
4. **Reusability** - Components and functions used across the app
5. **Developer Experience** - Fast development with hot reload and TypeScript

---

## Technology Stack Explained

### Why Vue 3?

**Vue 3** is a progressive JavaScript framework that makes building interactive UIs simple and maintainable.

**Key Advantages:**

-   **Reactive by Default** - When data changes, the UI updates automatically
-   **Composition API** - Modern way to organize component logic
-   **Small Bundle Size** - Fast loading for users
-   **Great Documentation** - Easy to learn

**Example of Reactivity:**

```typescript
const count = ref(0); // Create reactive data

function increment() {
	count.value++; // Change data...
	// UI updates automatically! No manual DOM manipulation
}
```

### Why TypeScript?

**TypeScript** adds type checking to JavaScript, catching bugs before they reach users.

**Benefits:**

```typescript
// ❌ JavaScript - Runtime error (too late!)
function login(email, password) {
	return api.login(email.trim(), password); // What if email is undefined?
}

// ✅ TypeScript - Compile-time error (caught early!)
function login(email: string, password: string): Promise<AuthResponse> {
	return api.login(email.trim(), password); // TypeScript ensures email exists
}
```

### Why Pinia?

**Pinia** is Vue's official state management library (successor to Vuex).

**What Problem Does It Solve?**
Without state management:

```
Component A needs user data → Fetch from API
Component B needs user data → Fetch from API again (wasteful!)
Component C needs user data → Fetch from API again (slow!)
```

With Pinia:

```
Component A needs user data → Check store → Fetch once if needed
Component B needs user data → Get from store (instant!)
Component C needs user data → Get from store (instant!)
```

### Why TailwindCSS?

**TailwindCSS** is a utility-first CSS framework.

**Traditional CSS:**

```css
/* styles.css */
.button {
	background-color: #3b82f6;
	padding: 0.5rem 1rem;
	border-radius: 0.375rem;
	color: white;
}
```

```html
<button class="button">Click Me</button>
```

**Tailwind Approach:**

```html
<button class="bg-blue-500 px-4 py-2 rounded text-white">Click Me</button>
```

**Benefits:**

-   No context switching between HTML and CSS files
-   No naming things (hardest problem in programming!)
-   Responsive design built-in (`sm:`, `md:`, `lg:`)
-   Tree-shaking removes unused styles automatically

---

## Project Structure

```
frontend/
├── src/
│   ├── api/                    # 🌐 API Communication Layer
│   │   ├── client.ts          # Axios setup + interceptors
│   │   ├── auth.ts            # Authentication endpoints
│   │   └── dashboard.ts       # Dashboard endpoints
│   │
│   ├── stores/                 # 🗄️ State Management (Pinia)
│   │   ├── auth.ts            # User authentication state
│   │   └── dashboard.ts       # Dashboard data state
│   │
│   ├── router/                 # 🧭 Navigation & Routing
│   │   └── index.ts           # Routes + auth guards
│   │
│   ├── views/                  # 📄 Page Components
│   │   ├── LoginView.vue      # Login page
│   │   ├── RegisterView.vue   # Registration page
│   │   ├── DashboardListView.vue    # Dashboard list
│   │   └── DashboardEditorView.vue  # Dashboard editor
│   │
│   ├── components/             # 🧩 Reusable Components
│   │   └── widgets/           # Widget components (Phase 8)
│   │
│   ├── types/                  # 📐 TypeScript Definitions
│   │   └── index.ts           # All interfaces & types
│   │
│   ├── composables/            # 🔧 Reusable Vue Logic
│   │   └── (future use)
│   │
│   ├── App.vue                 # 🏠 Root Component
│   └── main.ts                 # 🚀 Application Entry Point
│
├── public/                     # Static assets (images, etc.)
├── index.html                  # HTML template
├── vite.config.ts             # Vite bundler configuration
├── tailwind.config.js         # Tailwind customization
├── tsconfig.json              # TypeScript configuration
└── package.json               # Dependencies & scripts
```

### Understanding the Layers

#### 1. **Types Layer** (`src/types/`)

**Purpose:** Define the "shape" of data throughout the app

**Why First?** Because types are used everywhere else. Think of it as the contract between frontend and backend.

#### 2. **API Layer** (`src/api/`)

**Purpose:** All communication with the backend

**Why Important?** Centralizes all HTTP requests. If the backend URL changes, you update one file, not 50 components.

#### 3. **Store Layer** (`src/stores/`)

**Purpose:** Manage application state (data that multiple components need)

**When to Use?**

-   ✅ User authentication status (needed everywhere)
-   ✅ Dashboard list (needed in multiple views)
-   ❌ Form input value (only one component needs it - use local state)

#### 4. **Router Layer** (`src/router/`)

**Purpose:** Map URLs to components

**Example:** `/dashboards` → shows `DashboardListView.vue`

#### 5. **Views Layer** (`src/views/`)

**Purpose:** Full-page components that represent routes

**Difference from Components?**

-   **Views** = Full pages (`LoginView`, `DashboardListView`)
-   **Components** = Reusable pieces (`Button`, `Widget`, `Modal`)

---

## Type System (TypeScript)

### Core Types Explained

#### User Type

```typescript
export interface User {
	id: string; // UUID from database
	email: string; // User's email address
	created_at: string; // ISO 8601 timestamp
}
```

**Why These Fields?**

-   `id` - Unique identifier for API calls
-   `email` - Display in UI, used for login
-   `created_at` - Could show "Member since..." in profile

#### Dashboard Type

```typescript
export interface Dashboard {
	id: string;
	user_id: string; // Who owns this dashboard
	layout: LayoutConfig; // Widget positions
	settings: DashboardSettings; // Theme, title, etc.
	created_at: string;
	updated_at: string;
}
```

#### Layout Configuration

```typescript
export interface LayoutConfig {
	widgets: WidgetLayout[]; // Array of widgets
}

export interface WidgetLayout {
	id: string; // Unique widget instance ID
	type: WidgetType; // 'weather', 'crypto', etc.
	x: number; // Horizontal position
	y: number; // Vertical position
	w: number; // Width (grid units)
	h: number; // Height (grid units)
	config?: Record<string, unknown>; // Widget-specific settings
}
```

**GridStack Integration:**
These properties map directly to GridStack.js library:

```
┌─────────────────────────────────┐
│  Dashboard Grid (12 columns)    │
├─────────┬───────────────────────┤
│ Widget  │     Widget            │  ← Each widget has x, y, w, h
│ x:0 y:0 │     x:3 y:0           │
│ w:3 h:2 │     w:9 h:2           │
├─────────┴───────────────────────┤
│         Widget                   │
│         x:0 y:2                  │
│         w:12 h:3                 │
└──────────────────────────────────┘
```

#### Widget Types

```typescript
export type WidgetType =
	| 'weather' // Weather forecast
	| 'crypto' // Cryptocurrency prices
	| 'github' // GitHub activity
	| 'news' // News headlines
	| 'status'; // URL status checker
```

**Why Union Type?**
TypeScript will enforce that only these 5 values are valid:

```typescript
const widget: WidgetType = 'weather'; // ✅ OK
const widget: WidgetType = 'stocks'; // ❌ TypeScript error!
```

#### API Response Types

```typescript
export interface AuthResponse {
	token: string; // JWT token for authentication
	user: User; // User data
}

export interface ApiError {
	message: string; // Human-readable error
	status: number; // HTTP status code (404, 500, etc.)
	details?: Record<string, unknown>; // Optional extra info
}
```

### Adding New Types

**When to add a new type?**

1. New API endpoint returns different data
2. New component needs a complex prop type
3. New widget type added

**Example: Adding a "Todo" widget**

```typescript
// 1. Add to WidgetType union
export type WidgetType =
	| 'weather'
	| 'crypto'
	| 'github'
	| 'news'
	| 'status'
	| 'todo'; // ← New type

// 2. Define widget-specific config
export interface TodoWidgetConfig {
	listId: string;
	showCompleted: boolean;
}
```

---

## API Layer Architecture

### Overview: Why an API Layer?

**Problem Without API Layer:**

```vue
<!-- LoginView.vue -->
<script setup>
const login = async () => {
	const response = await fetch('http://localhost:8080/api/auth/login', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ email, password }),
	});
	// Repeated in every component that needs API calls!
};
</script>
```

**Solution With API Layer:**

```vue
<!-- LoginView.vue -->
<script setup>
import { authApi } from '@/api';

const login = async () => {
	const response = await authApi.login(email, password);
	// Clean, simple, reusable!
};
</script>
```

### API Client (`src/api/client.ts`)

The foundation of all API communication.

#### Axios Instance Configuration

```typescript
const apiClient: AxiosInstance = axios.create({
	baseURL: import.meta.env.VITE_API_URL || '/api', // Where backend lives
	timeout: 10000, // Fail after 10 seconds
	headers: {
		'Content-Type': 'application/json', // Send JSON data
	},
});
```

**Environment Variables:**

-   Development: `/api` (proxied to `http://localhost:8080/api` by Vite)
-   Production: `https://api.insightboard.com` (set via VITE_API_URL)

#### Request Interceptor: Adding Authentication

```typescript
apiClient.interceptors.request.use((config: InternalAxiosRequestConfig) => {
	const token = localStorage.getItem('auth_token');
	if (token && config.headers) {
		config.headers.Authorization = `Bearer ${token}`;
	}
	return config;
});
```

**What This Does:**

```
Every API Request:
┌─────────────────────────────────┐
│ 1. Check localStorage for token │
│ 2. If exists, add to headers    │
│ 3. Send request with auth       │
└─────────────────────────────────┘

Example:
GET /api/dashboards
Headers:
  Authorization: Bearer eyJhbGc...  ← Automatically added!
```

#### Response Interceptor: Error Handling

```typescript
apiClient.interceptors.response.use(
	(response) => response, // Success: pass through
	(error: AxiosError<ApiError>) => {
		if (error.response?.status === 401) {
			// Unauthorized: token expired or invalid
			localStorage.removeItem('auth_token');
			localStorage.removeItem('user');
			window.location.href = '/login'; // Force re-login
		}

		// Normalize error format
		const apiError: ApiError = {
			message: error.response?.data?.message || 'An error occurred',
			status: error.response?.status || 0,
		};

		return Promise.reject(apiError);
	},
);
```

**Error Handling Flow:**

```
API Request Fails:
┌──────────────────────────────────────┐
│ Is it a 401 Unauthorized?            │
│ YES → Clear auth, redirect to login  │
│ NO  → Format error, pass to caller   │
└──────────────────────────────────────┘
```

### Auth API Service (`src/api/auth.ts`)

```typescript
export const authApi = {
	async register(email: string, password: string): Promise<AuthResponse> {
		const response = await apiClient.post<AuthResponse>('/auth/register', {
			email,
			password,
		});
		return response.data;
	},

	async login(email: string, password: string): Promise<AuthResponse> {
		const response = await apiClient.post<AuthResponse>('/auth/login', {
			email,
			password,
		});
		return response.data;
	},

	async getCurrentUser(): Promise<User> {
		const response = await apiClient.get<User>('/auth/me');
		return response.data;
	},
};
```

**Usage Example:**

```typescript
// In a component or store
try {
	const { token, user } = await authApi.login('user@example.com', 'password');
	console.log('Logged in as:', user.email);
} catch (error) {
	console.error('Login failed:', error.message);
}
```

### Dashboard API Service (`src/api/dashboard.ts`)

Complete CRUD (Create, Read, Update, Delete) operations:

```typescript
export const dashboardApi = {
	// READ: Get all dashboards
	async list(): Promise<Dashboard[]> {
		const response = await apiClient.get<Dashboard[]>('/dashboards');
		return response.data;
	},

	// READ: Get specific dashboard
	async get(id: string): Promise<Dashboard> {
		const response = await apiClient.get<Dashboard>(`/dashboards/${id}`);
		return response.data;
	},

	// CREATE: New dashboard
	async create(
		layout: LayoutConfig,
		settings: DashboardSettings,
	): Promise<Dashboard> {
		const response = await apiClient.post<Dashboard>('/dashboards', {
			layout,
			settings,
		});
		return response.data;
	},

	// UPDATE: Modify existing
	async update(
		id: string,
		layout?: LayoutConfig,
		settings?: DashboardSettings,
	): Promise<Dashboard> {
		const response = await apiClient.put<Dashboard>(`/dashboards/${id}`, {
			layout,
			settings,
		});
		return response.data;
	},

	// DELETE: Remove dashboard
	async delete(id: string): Promise<void> {
		await apiClient.delete(`/dashboards/${id}`);
	},
};
```

**Partial Updates:**

```typescript
// Update only layout (keep existing settings)
await dashboardApi.update(dashboardId, newLayout, undefined);

// Update only settings (keep existing layout)
await dashboardApi.update(dashboardId, undefined, newSettings);

// Update both
await dashboardApi.update(dashboardId, newLayout, newSettings);
```

---

## State Management with Pinia

### What is State Management?

**State** = Data that your application needs to remember

**Example Without State Management:**

```vue
<!-- NavBar.vue -->
<template>
	<div>User: {{ userEmail }}</div>
	<!-- Need user email here -->
</template>

<!-- ProfilePage.vue -->
<template>
	<div>Email: {{ userEmail }}</div>
	<!-- Need same email here -->
</template>

<!-- Problem: How do both components access the same user data? -->
```

**Solutions:**

1. ❌ Fetch user data twice (wasteful)
2. ❌ Pass data through props (tedious for deeply nested components)
3. ✅ **Use Pinia store** (one source of truth)

### Auth Store (`src/stores/auth.ts`)

#### Store Structure

```typescript
export const useAuthStore = defineStore('auth', () => {
	// STATE: Reactive data
	const user = ref<User | null>(null);
	const token = ref<string | null>(null);
	const loading = ref(false);
	const error = ref<string | null>(null);

	// GETTERS: Computed values
	const isAuthenticated = computed(() => !!token.value && !!user.value);

	// ACTIONS: Functions that modify state
	async function login(email: string, password: string) {
		// Implementation...
	}

	function logout() {
		// Implementation...
	}

	// RETURN: Public API
	return {
		user,
		token,
		loading,
		error, // State
		isAuthenticated, // Getters
		login,
		logout, // Actions
	};
});
```

#### State Persistence

**Problem:** User logs in, refreshes page, gets logged out (bad UX!)

**Solution:** Save to localStorage

```typescript
function initialize() {
	const savedToken = localStorage.getItem('auth_token');
	const savedUser = localStorage.getItem('user');

	if (savedToken && savedUser) {
		token.value = savedToken;
		try {
			user.value = JSON.parse(savedUser);
		} catch (e) {
			// Invalid data, clear everything
			logout();
		}
	}
}

// Called automatically when store is created
initialize();
```

#### Login Action

```typescript
async function login(email: string, password: string): Promise<void> {
	loading.value = true; // Show loading spinner
	error.value = null; // Clear previous errors

	try {
		const response = await authApi.login(email, password);

		// Update state
		token.value = response.token;
		user.value = response.user;

		// Persist to localStorage
		localStorage.setItem('auth_token', response.token);
		localStorage.setItem('user', JSON.stringify(response.user));
	} catch (err) {
		const apiError = err as ApiError;
		error.value = apiError.message; // Store error for UI
		throw err; // Re-throw so component can handle it
	} finally {
		loading.value = false; // Hide loading spinner
	}
}
```

#### Using Auth Store in Components

```vue
<script setup lang="ts">
import { useAuthStore } from '@/stores/auth';

const authStore = useAuthStore();

const handleLogin = async () => {
	try {
		await authStore.login(email.value, password.value);
		// Login succeeded, redirect happens in component
	} catch (error) {
		// Error already stored in authStore.error
		// UI will show error message automatically
	}
};
</script>

<template>
	<div>
		<!-- Show loading state -->
		<button :disabled="authStore.loading">
			{{ authStore.loading ? 'Logging in...' : 'Login' }}
		</button>

		<!-- Show error message -->
		<div v-if="authStore.error" class="error">
			{{ authStore.error }}
		</div>

		<!-- Show user info if logged in -->
		<div v-if="authStore.isAuthenticated">
			Welcome, {{ authStore.user?.email }}!
		</div>
	</div>
</template>
```

### Dashboard Store (`src/stores/dashboard.ts`)

#### Managing Collections

```typescript
export const useDashboardStore = defineStore('dashboard', () => {
	// STATE
	const dashboards = ref<Dashboard[]>([]); // All dashboards
	const currentDashboard = ref<Dashboard | null>(null); // Active one
	const loading = ref(false);
	const error = ref<string | null>(null);

	// GETTERS
	const hasDashboards = computed(() => dashboards.value.length > 0);

	const getDashboardById = computed(() => {
		return (id: string) => dashboards.value.find((d) => d.id === id);
	});

	// ACTIONS
	async function fetchDashboards() {
		loading.value = true;
		error.value = null;

		try {
			const data = await dashboardApi.list();
			dashboards.value = data; // Replace entire array
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	async function createDashboard(
		layout: LayoutConfig,
		settings: DashboardSettings,
	): Promise<Dashboard> {
		loading.value = true;
		error.value = null;

		try {
			const data = await dashboardApi.create(layout, settings);
			dashboards.value.push(data); // Add to array
			currentDashboard.value = data; // Set as current
			return data;
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	async function updateDashboard(
		id: string,
		layout?: LayoutConfig,
		settings?: DashboardSettings,
	): Promise<Dashboard> {
		loading.value = true;
		error.value = null;

		try {
			const data = await dashboardApi.update(id, layout, settings);

			// Update in array
			const index = dashboards.value.findIndex((d) => d.id === id);
			if (index !== -1) {
				dashboards.value[index] = data;
			}

			// Update current if it's the same
			if (currentDashboard.value?.id === id) {
				currentDashboard.value = data;
			}

			return data;
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	async function deleteDashboard(id: string): Promise<void> {
		loading.value = true;
		error.value = null;

		try {
			await dashboardApi.delete(id);

			// Remove from array
			dashboards.value = dashboards.value.filter((d) => d.id !== id);

			// Clear current if it was deleted
			if (currentDashboard.value?.id === id) {
				currentDashboard.value = null;
			}
		} catch (err) {
			const apiError = err as ApiError;
			error.value = apiError.message;
			throw err;
		} finally {
			loading.value = false;
		}
	}

	return {
		dashboards,
		currentDashboard,
		loading,
		error,
		hasDashboards,
		getDashboardById,
		fetchDashboards,
		createDashboard,
		updateDashboard,
		deleteDashboard,
	};
});
```

---

## Routing System

### Router Configuration (`src/router/index.ts`)

#### Route Definitions

```typescript
const routes: RouteRecordRaw[] = [
	{
		path: '/',
		name: 'home',
		redirect: () => {
			const authStore = useAuthStore();
			return authStore.isAuthenticated ? '/dashboards' : '/login';
		},
	},
	{
		path: '/login',
		name: 'login',
		component: LoginView,
		meta: { requiresGuest: true }, // Only for non-authenticated users
	},
	{
		path: '/register',
		name: 'register',
		component: RegisterView,
		meta: { requiresGuest: true },
	},
	{
		path: '/dashboards',
		name: 'dashboards',
		component: DashboardListView,
		meta: { requiresAuth: true }, // Protected route
	},
	{
		path: '/dashboards/:id',
		name: 'dashboard-editor',
		component: () => import('../views/DashboardEditorView.vue'), // Lazy-loaded
		meta: { requiresAuth: true },
	},
];
```

#### Navigation Guards

**What are Guards?**
Functions that run before navigating to a route. They can:

-   Allow navigation (user can proceed)
-   Redirect to different route (e.g., login page)
-   Cancel navigation entirely

```typescript
router.beforeEach((to, from, next) => {
	const authStore = useAuthStore();

	// Check if route requires authentication
	if (to.meta.requiresAuth && !authStore.isAuthenticated) {
		// Save where user wanted to go
		next({
			name: 'login',
			query: { redirect: to.fullPath }, // Save intended destination
		});
		return;
	}

	// Check if route is for guests only
	if (to.meta.requiresGuest && authStore.isAuthenticated) {
		// Already logged in, go to dashboards
		next({ name: 'dashboards' });
		return;
	}

	// All checks passed, proceed
	next();
});
```

**Flow Diagram:**

```
User tries to access /dashboards:
┌─────────────────────────────────────┐
│ Is user authenticated?              │
├─────────────────────────────────────┤
│ YES → Allow access                  │
│ NO  → Redirect to /login?redirect=/dashboards
└─────────────────────────────────────┘

After login:
┌─────────────────────────────────────┐
│ Is there a redirect query param?    │
├─────────────────────────────────────┤
│ YES → Go to saved destination       │
│ NO  → Go to /dashboards             │
└─────────────────────────────────────┘
```

#### Programmatic Navigation

```typescript
import { useRouter } from 'vue-router';

const router = useRouter();

// Navigate to route by name
router.push({ name: 'dashboards' });

// Navigate with parameters
router.push({ name: 'dashboard-editor', params: { id: 'abc-123' } });

// Navigate with query string
router.push({ path: '/login', query: { redirect: '/dashboards' } });

// Go back
router.back();

// Replace current history entry (user can't go back)
router.replace({ name: 'home' });
```

---

## Component Architecture

### View Components

#### LoginView.vue

```vue
<template>
	<div class="min-h-screen flex items-center justify-center bg-gray-50">
		<div class="max-w-md w-full space-y-8">
			<div>
				<h2 class="text-3xl font-extrabold text-gray-900">
					Sign in to InsightBoard
				</h2>
			</div>

			<form @submit.prevent="handleSubmit">
				<!-- Email Input -->
				<input
					v-model="email"
					type="email"
					required
					placeholder="Email address"
					class="..."
				/>

				<!-- Password Input -->
				<input
					v-model="password"
					type="password"
					required
					placeholder="Password"
					class="..."
				/>

				<!-- Error Message -->
				<div v-if="authStore.error" class="bg-red-50 p-4">
					{{ authStore.error }}
				</div>

				<!-- Submit Button -->
				<button type="submit" :disabled="authStore.loading" class="...">
					{{ authStore.loading ? 'Signing in...' : 'Sign in' }}
				</button>
			</form>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();

const email = ref('');
const password = ref('');

const handleSubmit = async () => {
	try {
		await authStore.login(email.value, password.value);

		// Redirect to intended destination or dashboards
		const redirect = route.query.redirect as string;
		router.push(redirect || '/dashboards');
	} catch (error) {
		// Error is already stored in authStore.error
		console.error('Login failed:', error);
	}
};
</script>
```

**Key Features:**

1. **v-model** - Two-way binding for form inputs
2. **@submit.prevent** - Handle form submission, prevent page reload
3. **:disabled** - Dynamic attribute binding based on loading state
4. **Conditional rendering** - Show error only if it exists

#### DashboardListView.vue

Complete dashboard management interface with:

**States:**

1. **Loading** - Spinner while fetching data
2. **Error** - Error message with retry button
3. **Empty** - Friendly message when no dashboards exist
4. **Content** - Grid of dashboard cards

```vue
<template>
	<div class="min-h-screen bg-gray-50">
		<!-- Header with Create Button -->
		<header class="bg-white shadow">
			<div class="max-w-7xl mx-auto py-6 px-4 flex justify-between">
				<h1 class="text-3xl font-bold">My Dashboards</h1>
				<button @click="handleCreateDashboard">New Dashboard</button>
			</div>
		</header>

		<main class="max-w-7xl mx-auto py-6">
			<!-- Loading State -->
			<div v-if="loading && !dashboardStore.dashboards.length">
				<div class="animate-spin ..."></div>
				<p>Loading dashboards...</p>
			</div>

			<!-- Error State -->
			<div v-else-if="dashboardStore.error" class="bg-red-50 p-4">
				<h3>Error: {{ dashboardStore.error }}</h3>
				<button @click="loadDashboards">Try again</button>
			</div>

			<!-- Empty State -->
			<div v-else-if="!dashboardStore.hasDashboards">
				<svg class="..."><!-- Icon --></svg>
				<h3>No dashboards</h3>
				<p>Get started by creating a new dashboard.</p>
				<button @click="handleCreateDashboard">Create Dashboard</button>
			</div>

			<!-- Dashboard Grid -->
			<div
				v-else
				class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4"
			>
				<div
					v-for="dashboard in dashboardStore.dashboards"
					:key="dashboard.id"
					class="bg-white shadow rounded-lg p-6"
				>
					<h3>
						{{ dashboard.settings.title || 'Untitled Dashboard' }}
					</h3>
					<p>Created {{ formatDate(dashboard.created_at) }}</p>
					<p>{{ dashboard.layout.widgets.length }} widgets</p>

					<div class="flex space-x-3 mt-4">
						<router-link :to="`/dashboards/${dashboard.id}`">
							Open
						</router-link>
						<button @click="handleDeleteDashboard(dashboard.id)">
							Delete
						</button>
					</div>
				</div>
			</div>
		</main>
	</div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useDashboardStore } from '@/stores/dashboard';

const router = useRouter();
const dashboardStore = useDashboardStore();
const loading = ref(false);

onMounted(async () => {
	await loadDashboards();
});

const loadDashboards = async () => {
	loading.value = true;
	try {
		await dashboardStore.fetchDashboards();
	} catch (error) {
		console.error('Failed to load dashboards:', error);
	} finally {
		loading.value = false;
	}
};

const handleCreateDashboard = async () => {
	try {
		const dashboard = await dashboardStore.createDashboard(
			{ widgets: [] }, // Empty layout
			{ title: 'New Dashboard', theme: 'light' },
		);
		router.push(`/dashboards/${dashboard.id}`);
	} catch (error) {
		console.error('Failed to create dashboard:', error);
	}
};

const handleDeleteDashboard = async (id: string) => {
	if (!confirm('Are you sure?')) return;

	try {
		await dashboardStore.deleteDashboard(id);
	} catch (error) {
		console.error('Failed to delete dashboard:', error);
	}
};

const formatDate = (dateString: string): string => {
	const date = new Date(dateString);
	const now = new Date();
	const diffDays = Math.floor(
		(now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24),
	);

	if (diffDays === 0) return 'today';
	if (diffDays === 1) return 'yesterday';
	if (diffDays < 7) return `${diffDays} days ago`;
	return date.toLocaleDateString();
};
</script>
```

---

## Styling with TailwindCSS

### Configuration (`tailwind.config.js`)

```javascript
export default {
	content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
	theme: {
		extend: {
			colors: {
				primary: {
					50: '#eff6ff',
					100: '#dbeafe',
					// ... full color scale
					600: '#2563eb', // Main brand color
					700: '#1d4ed8',
					// ...
				},
			},
		},
	},
};
```

### Using Custom Colors

```vue
<template>
	<!-- Background -->
	<div class="bg-primary-600">...</div>

	<!-- Text -->
	<h1 class="text-primary-700">...</h1>

	<!-- Hover state -->
	<button class="bg-primary-600 hover:bg-primary-700">...</button>

	<!-- Responsive -->
	<div class="bg-gray-100 md:bg-primary-50 lg:bg-white">...</div>
</template>
```

### Common Patterns

#### Responsive Grid

```html
<!-- 1 column mobile, 2 tablet, 3 desktop -->
<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
	<div>Item 1</div>
	<div>Item 2</div>
	<div>Item 3</div>
</div>
```

#### Centered Container

```html
<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
	<!-- Content automatically centered with responsive padding -->
</div>
```

#### Button Styles

```html
<!-- Primary button -->
<button
	class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-md shadow-sm"
>
	Click Me
</button>

<!-- Secondary button -->
<button
	class="bg-white hover:bg-gray-50 text-gray-700 px-4 py-2 rounded-md border border-gray-300"
>
	Cancel
</button>
```

---

## Authentication Flow

### Complete Flow Diagram

```
1. App Starts
   ↓
   Initialize Auth Store
   ↓
   Check localStorage for token
   ↓
   ┌─────────────┬─────────────┐
   │ Token Found │ No Token    │
   ↓             ↓
   Set State     Remain Logged Out
   ↓

2. User Navigates to /dashboards
   ↓
   Router Guard Checks Auth
   ↓
   ┌─────────────────┬──────────────────┐
   │ Authenticated   │ Not Authenticated │
   ↓                 ↓
   Allow Access      Redirect to /login?redirect=/dashboards

3. User Submits Login Form
   ↓
   Call authStore.login()
   ↓
   POST /api/auth/login
   ↓
   ┌──────────┬──────────┐
   │ Success  │ Failure  │
   ↓          ↓
   Store      Show Error
   Token &    Message
   User Data
   ↓
   Save to
   localStorage
   ↓
   Redirect to
   /dashboards
   (or saved destination)

4. Subsequent API Calls
   ↓
   Request Interceptor
   ↓
   Add Authorization Header
   ↓
   Backend Validates Token
   ↓
   ┌──────────┬──────────┐
   │ Valid    │ Invalid  │
   ↓          ↓
   Return     401 Response
   Data       ↓
              Response Interceptor
              ↓
              Clear Auth
              ↓
              Redirect to /login
```

### Security Considerations

**Token Storage:**

-   **Current:** localStorage (simple, persistent across tabs)
-   **Pros:** Survives page refresh, easy to implement
-   **Cons:** Vulnerable to XSS attacks
-   **Alternative:** httpOnly cookies (more secure, but requires backend changes)

**Token Expiration:**

```typescript
// Future enhancement: Check token expiration
function isTokenExpired(token: string): boolean {
	try {
		const payload = JSON.parse(atob(token.split('.')[1]));
		return payload.exp * 1000 < Date.now();
	} catch {
		return true;
	}
}
```

---

## Data Flow Diagrams

### Creating a Dashboard

```
User Action: Click "New Dashboard" Button
    ↓
DashboardListView.vue
    ↓
handleCreateDashboard()
    ↓
dashboardStore.createDashboard({ widgets: [] }, { title: '...' })
    ↓
Dashboard Store
    ↓
Set loading = true
    ↓
dashboardApi.create(layout, settings)
    ↓
API Client
    ↓
POST /api/dashboards
Headers: Authorization: Bearer <token>
Body: { layout, settings }
    ↓
Backend API
    ↓
[Insert into Database]
    ↓
Return new dashboard object
    ↓
API Client receives response
    ↓
Dashboard Store
    ↓
Add to dashboards array
Set as currentDashboard
Set loading = false
    ↓
Vue Reactivity System
    ↓
UI Updates Automatically
    ↓
New dashboard appears in grid
    ↓
Router
    ↓
Navigate to /dashboards/:id
```

### Loading Dashboard Data

```
User navigates to /dashboards
    ↓
DashboardListView mounted
    ↓
onMounted() hook
    ↓
loadDashboards()
    ↓
dashboardStore.fetchDashboards()
    ↓
GET /api/dashboards
    ↓
Backend returns array of dashboards
    ↓
dashboardStore.dashboards = data
    ↓
Vue updates v-for loop
    ↓
Dashboard cards rendered
```

---

## Best Practices & Patterns

### 1. Component Composition

**Bad: Monolithic Component**

```vue
<script setup>
// 500 lines of code in one component
const dashboards = ref([]);
const user = ref(null);
const notifications = ref([]);
// ... everything in one file
</script>
```

**Good: Composable Logic**

```typescript
// composables/useDashboards.ts
export function useDashboards() {
	const store = useDashboardStore();

	async function loadDashboards() {
		await store.fetchDashboards();
	}

	return { dashboards: store.dashboards, loadDashboards };
}

// Component
<script setup>
	import {useDashboards} from '@/composables/useDashboards'; const{' '}
	{(dashboards, loadDashboards)} = useDashboards();
</script>;
```

### 2. Error Handling

**Pattern: Try-Catch with User Feedback**

```typescript
async function saveData() {
	try {
		await api.save(data);
		// Success feedback
		showToast('Saved successfully!', 'success');
	} catch (error) {
		// User-friendly error
		showToast('Failed to save. Please try again.', 'error');
		// Developer feedback
		console.error('Save failed:', error);
	}
}
```

### 3. Loading States

**Pattern: Skeleton Screens**

```vue
<template>
	<div v-if="loading" class="animate-pulse">
		<!-- Skeleton UI matching actual content -->
		<div class="h-8 bg-gray-200 rounded w-1/4 mb-4"></div>
		<div class="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
		<div class="h-4 bg-gray-200 rounded w-1/2"></div>
	</div>

	<div v-else>
		<!-- Actual content -->
	</div>
</template>
```

### 4. Type Safety

**Always type function parameters and returns:**

```typescript
// ❌ Bad
async function updateDashboard(id, data) {
	return api.put(`/dashboards/${id}`, data);
}

// ✅ Good
async function updateDashboard(
	id: string,
	data: Partial<Dashboard>,
): Promise<Dashboard> {
	return api.put<Dashboard>(`/dashboards/${id}`, data);
}
```

---

## Common Scenarios

### Scenario 1: Adding a New Page

**Steps:**

1. Create view component: `src/views/NewPageView.vue`
2. Add route in `src/router/index.ts`:

```typescript
{
    path: '/new-page',
    name: 'new-page',
    component: NewPageView,
    meta: { requiresAuth: true },  // If protected
}
```

3. Add navigation link in `App.vue` or other component

### Scenario 2: Adding a New API Endpoint

**Steps:**

1. Define types in `src/types/index.ts`:

```typescript
export interface NewData {
	id: string;
	name: string;
}
```

2. Create API service in `src/api/new-service.ts`:

```typescript
export const newApi = {
	async getData(): Promise<NewData[]> {
		const response = await apiClient.get<NewData[]>('/new-endpoint');
		return response.data;
	},
};
```

3. Use in component:

```typescript
import { newApi } from '@/api/new-service';

const data = ref<NewData[]>([]);

onMounted(async () => {
	data.value = await newApi.getData();
});
```

### Scenario 3: Adding Global State

**When to use Pinia:**

-   Data needed by multiple components
-   Data that persists across routes
-   Complex state logic

**Steps:**

1. Create store: `src/stores/new-store.ts`
2. Define state, getters, actions
3. Use in components: `const store = useNewStore()`

---

## Troubleshooting

### Issue: "Cannot read property of undefined"

**Cause:** Accessing nested properties before data loads

**Solution:** Optional chaining

```vue
<!-- ❌ Crashes if user is null -->
<div>{{ user.email }}</div>

<!-- ✅ Safe -->
<div>{{ user?.email }}</div>
<div>{{ user?.profile?.avatar }}</div>
```

### Issue: API calls return 401

**Causes:**

1. Token expired
2. Token not sent
3. Token malformed

**Debug:**

```typescript
// Check token exists
console.log('Token:', localStorage.getItem('auth_token'));

// Check request headers
apiClient.interceptors.request.use((config) => {
	console.log('Request headers:', config.headers);
	return config;
});
```

**Solution:** Clear storage and re-login

```typescript
localStorage.clear();
window.location.href = '/login';
```

### Issue: Page doesn't update after API call

**Cause:** Not using reactive references

**Solution:**

```typescript
// ❌ Wrong
let data = [];
data = await api.getData(); // UI won't update

// ✅ Correct
const data = ref([]);
data.value = await api.getData(); // UI updates
```

### Issue: CORS errors in development

**Cause:** Browser blocking requests to different origin

**Solution:** Vite proxy (already configured)

```typescript
// vite.config.ts
export default defineConfig({
	server: {
		proxy: {
			'/api': {
				target: 'http://localhost:8080',
				changeOrigin: true,
			},
		},
	},
});
```

### Issue: TypeScript errors in .vue files

**Solution:** Restart TypeScript server in VS Code

-   Press `Ctrl+Shift+P`
-   Type "Restart TypeScript Server"
-   Press Enter

### Issue: App crashes on load with "Cannot access useAuthStore"

**Cause:** Pinia not initialized in `main.ts`

**Solution:** Ensure plugins are installed before mounting

```typescript
// main.ts - CORRECT ORDER
const app = createApp(App);
const pinia = createPinia();

app.use(pinia); // ← Must be BEFORE mount
app.use(router); // ← Must be BEFORE mount
app.mount('#app');
```

### Issue: Tailwind classes don't work

**Cause:** Missing `@tailwind` directives in `style.css`

**Solution:** Add directives at the top of `style.css`

```css
/* style.css - Must be at the TOP */
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Your custom styles below */
```

After adding, restart the dev server:

```bash
# Stop: Ctrl+C
# Start: npm run dev
```

### Issue: All API calls return 404 in development

**Causes:**

1. Backend server not running
2. Proxy misconfigured

**Solutions:**

**Check 1: Backend is running**

```bash
curl http://localhost:8080/healthz
# Should return: ok
```

**Check 2: Proxy correctly configured**

```typescript
// vite.config.ts must have server.proxy section
server: {
  proxy: {
    '/api': { target: 'http://localhost:8080', changeOrigin: true }
  }
}
```

**Check 3: API calls use relative paths**

```typescript
// ✅ Correct - uses proxy
await api.post('/api/auth/login', ...)

// ❌ Wrong - bypasses proxy
await api.post('http://localhost:8080/api/auth/login', ...)
```

### Issue: White screen with no errors in console

**Debugging steps:**

1. Check browser console (F12) for any warnings
2. Verify `<div id="app"></div>` exists in `index.html`
3. Check network tab - did `main.ts` load?
4. Try hard refresh: `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
5. Clear Vite cache and restart:
    ```bash
    rm -rf node_modules/.vite
    npm run dev
    ```

---

## Phase 7: Widget System & Dashboard Editor

### Overview

Phase 7 introduces the complete widget system with drag-and-drop dashboard editing, five functional widgets, and a dynamic configuration system. This is the heart of InsightBoard's functionality.

### Architecture Components

#### 1. Widget Registry System (`stores/widgets.ts`)

The widget registry is a centralized store that manages all available widget types and their configurations.

**What is a Widget?**

A widget is a self-contained, configurable component that displays data from external APIs (GitHub, Weather, News, etc.). Think of it like an app on your phone - each has its own purpose, settings, and display.

**Key Concepts:**

```typescript
// Widget Type Definition - The "blueprint" for a widget
interface WidgetType {
	id: string; // Unique identifier (e.g., "github", "weather")
	name: string; // Display name
	description: string; // User-facing description
	icon: string; // Emoji or icon class
	component: Component; // Vue component to render
	defaultConfig: Record<string, any>; // Default settings
	configSchema: WidgetConfigField[]; // Form fields for configuration
	defaultSize: { w: number; h: number }; // Grid dimensions
	refreshInterval?: number; // Auto-refresh in seconds
}

// Widget Instance - A specific widget placed on a dashboard
interface WidgetInstance {
	id: string; // Unique instance ID (generated)
	type: string; // Widget type ID (links to WidgetType)
	config: Record<string, any>; // User's configuration
	layout: { x: number; y: number; w: number; h: number }; // Position
}
```

**How Registration Works:**

```typescript
// In widgetRegistry.ts
const widgetStore = useWidgetStore();

widgetStore.registerWidget({
	id: 'github',
	name: 'GitHub Activity',
	component: GitHubWidget, // ← The actual Vue component
	defaultConfig: { username: '' }, // ← What user needs to configure
	configSchema: [
		{
			// ← How to render the config form
			key: 'username',
			label: 'GitHub Username',
			type: 'text',
			required: true,
			validation: (value) => {
				// Custom validation logic
				if (!/^[a-z\d-]{1,39}$/i.test(value)) {
					return 'Invalid username format';
				}
				return null;
			},
		},
	],
	defaultSize: { w: 4, h: 3 }, // 4 columns wide, 3 rows tall
	refreshInterval: 300, // Refresh every 5 minutes
});
```

**Why This Design?**

-   **Extensibility**: Adding a new widget is as simple as registering it
-   **Type Safety**: TypeScript ensures all widgets follow the same contract
-   **Configuration**: Each widget defines its own settings dynamically
-   **Validation**: Built-in validation prevents invalid configurations

#### 2. Base Widget Component (`BaseWidget.vue`)

All widgets wrap themselves in `BaseWidget`, which provides:

-   ✅ Header with title and last update time
-   ✅ Refresh button with loading spinner
-   ✅ Settings menu (configure, remove)
-   ✅ Loading state with skeleton
-   ✅ Error state with retry button
-   ✅ Consistent styling and UX

**How It Works:**

```vue
<script setup lang="ts">
// Child widget emits events, BaseWidget handles UI
interface Emits {
	(e: 'refresh'): void; // User clicked refresh
	(e: 'configure'): void; // User clicked configure
	(e: 'remove'): void; // User clicked remove
}
</script>

<template>
	<div class="widget-base">
		<!-- Header with title, timestamp, actions -->
		<div class="widget-header">
			<h3>{{ title }}</h3>
			<button @click="emit('refresh')">🔄</button>
		</div>

		<!-- Loading/Error/Content -->
		<div v-if="loading">Loading...</div>
		<div v-else-if="error">{{ error }}</div>
		<div v-else>
			<slot />
			<!-- Child widget renders here -->
		</div>
	</div>
</template>
```

**Why This Pattern?**

-   **DRY (Don't Repeat Yourself)**: Common UI logic in one place
-   **Consistency**: All widgets look and behave the same
-   **Maintainability**: Fix a bug once, it's fixed everywhere

#### 3. Individual Widget Components

Each widget is a specialized Vue component that:

1. **Fetches data** from the API
2. **Displays data** in a meaningful way
3. **Handles auto-refresh** using intervals
4. **Emits events** for user actions

**Example: GitHub Widget (`GitHubWidget.vue`)**

```vue
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { fetchGitHubData } from '../../api/widgets';

interface Props {
	config: { username: string }; // Widget configuration
	refreshInterval?: number; // Seconds between refreshes
}

const props = defineProps<Props>();
const events = ref([]);
const loading = ref(false);
const error = ref(null);

const loadData = async () => {
	loading.value = true;
	error.value = null;
	try {
		events.value = await fetchGitHubData(props.config.username);
	} catch (err) {
		error.value = err.message;
	} finally {
		loading.value = false;
	}
};

// Auto-refresh setup
let timer = null;
watch(
	() => props.refreshInterval,
	(interval) => {
		if (timer) clearInterval(timer);
		if (interval > 0) {
			timer = setInterval(loadData, interval * 1000);
		}
	},
	{ immediate: true },
);

onMounted(loadData);
</script>

<template>
	<BaseWidget
		:title="`GitHub: ${config.username}`"
		:loading="loading"
		:error="error"
		@refresh="loadData"
		@configure="$emit('configure')"
		@remove="$emit('remove')"
	>
		<!-- Widget-specific UI -->
		<div v-for="event in events" :key="event.id">
			{{ event.type }} - {{ event.repo.name }}
		</div>
	</BaseWidget>
</template>
```

**Key Features:**

-   **Props-driven configuration**: Parent passes `config` object
-   **Auto-refresh**: Uses `setInterval` based on `refreshInterval` prop
-   **Error handling**: Displays errors through BaseWidget
-   **Event emission**: Notifies parent of user actions

#### 4. Widget Configuration Modal (`WidgetConfigModal.vue`)

A dynamic form that renders based on a widget's `configSchema`.

**How It Works:**

```typescript
// Widget defines schema
configSchema: [
  {
    key: 'username',
    label: 'GitHub Username',
    type: 'text', // Input type
    required: true,
    placeholder: 'e.g., octocat',
    helpText: 'Enter your GitHub username',
    validation: (value) => value ? null : 'Required field'
  },
  {
    key: 'showPrivate',
    label: 'Show Private Repos',
    type: 'checkbox',
    helpText: 'Include private repositories'
  }
]

// Modal renders form dynamically
<input v-if="field.type === 'text'" v-model="formData[field.key]" />
<input v-else-if="field.type === 'checkbox'" type="checkbox" v-model="formData[field.key]" />
<select v-else-if="field.type === 'select'" v-model="formData[field.key]">
  <option v-for="opt in field.options" :value="opt.value">{{ opt.label }}</option>
</select>
```

**Supported Field Types:**

-   `text`: Single-line text input
-   `number`: Numeric input with validation
-   `textarea`: Multi-line text input
-   `select`: Dropdown with options
-   `checkbox`: Boolean toggle
-   `url`: URL input with validation

**Validation Flow:**

1. User edits form
2. On save, modal validates each field
3. If errors, show error messages
4. If valid, emit `@save` event with config
5. Parent updates widget config and saves dashboard

#### 5. Dashboard Editor View (`DashboardEditorView.vue`)

The main view that ties everything together with GridStack for drag-and-drop layout.

**What is GridStack?**

GridStack is a JavaScript library that provides drag-and-drop grid layouts. Think of it like organizing windows on your desktop - you can move, resize, and rearrange items.

**Key Features:**

-   📐 **12-column grid system**: Widgets span 1-12 columns
-   🖱️ **Drag-and-drop**: Click and drag to reposition
-   📏 **Resize**: Drag corners to resize
-   💾 **Auto-save**: Changes save automatically after 2 seconds
-   🔄 **Persistence**: Layout stored in database

**How It Works:**

```vue
<script setup lang="ts">
import { GridStack } from 'gridstack';
import 'gridstack/dist/gridstack.min.css';

const gridInstance = ref(null);
const widgets = ref([]); // Widget instances

// Initialize GridStack
const initializeGrid = () => {
	gridInstance.value = GridStack.init({
		cellHeight: 80, // Each row is 80px tall
		margin: 10, // 10px gap between widgets
		column: 12, // 12-column grid
		animate: true, // Smooth animations
	});

	// Listen for changes (drag, resize)
	gridInstance.value.on('change', () => {
		scheduleAutoSave(); // Debounced save
	});
};

// Add widget to grid
const addWidget = (typeId) => {
	const widget = widgetStore.createWidgetInstance(typeId);
	widgets.value.push(widget);

	gridInstance.value.addWidget({
		id: widget.id,
		x: 0,
		y: 0,
		w: 4,
		h: 3,
	});
};

// Save layout to database
const saveDashboard = async () => {
	const gridItems = gridInstance.value.getGridItems();

	// Update widget positions from grid
	gridItems.forEach((el) => {
		const widget = widgets.value.find((w) => w.id === el.id);
		const node = el.gridstackNode;
		widget.layout = {
			x: node.x,
			y: node.y,
			w: node.w,
			h: node.h,
		};
	});

	// Save to backend
	await dashboardStore.updateDashboard(dashboardId, {
		layout: { widgets: widgets.value },
	});
};
</script>

<template>
	<div class="dashboard-editor">
		<!-- Top bar with Add Widget button -->
		<div class="toolbar">
			<button @click="showWidgetPicker = true">+ Add Widget</button>
		</div>

		<!-- GridStack container -->
		<div ref="gridContainer" class="grid-stack">
			<div
				v-for="widget in widgets"
				:key="widget.id"
				class="grid-stack-item"
				:gs-x="widget.layout.x"
				:gs-y="widget.layout.y"
				:gs-w="widget.layout.w"
				:gs-h="widget.layout.h"
			>
				<div class="grid-stack-item-content">
					<component
						:is="getWidgetComponent(widget.type)"
						:config="widget.config"
						@configure="configureWidget(widget.id)"
						@remove="removeWidget(widget.id)"
					/>
				</div>
			</div>
		</div>
	</div>
</template>
```

**GridStack Layout Persistence:**

```typescript
// Database stores layout as JSONB
{
  "widgets": [
    {
      "id": "widget-123",
      "type": "github",
      "x": 0,     // Column position (0-11)
      "y": 0,     // Row position
      "w": 4,     // Width in columns
      "h": 3,     // Height in rows
      "config": { "username": "octocat" }
    }
  ]
}
```

#### 6. API Integration (`api/widgets.ts`)

Each widget type has a corresponding API function that fetches data from the backend.

**Structure:**

```typescript
// GitHub data fetcher
export const fetchGitHubData = async (username: string) => {
	const response = await apiClient.get('/data/github', {
		params: { username },
	});
	return response.data;
};

// Weather data fetcher
export const fetchWeatherData = async (city: string) => {
	const response = await apiClient.get('/data/weather', {
		params: { city },
	});
	return response.data;
};
```

**Benefits:**

-   **Centralized**: All API calls in one place
-   **Type-safe**: Return types match backend responses
-   **Reusable**: Multiple widgets can use same API
-   **Cached**: Backend handles caching automatically

### Data Flow Diagram

```
User Action (Add Widget)
    ↓
Dashboard Editor
    ↓
Widget Store (Create Instance)
    ↓
GridStack (Add to Grid)
    ↓
Widget Component (Render)
    ↓
API Call (Fetch Data)
    ↓
Backend (Return Cached or Fresh Data)
    ↓
Widget Updates UI
    ↓
Auto-Save Triggers (After 2s)
    ↓
Dashboard Store (Update DB)
```

### Widget Lifecycle

1. **Registration**: Widget type registered in `widgetRegistry.ts`
2. **Creation**: User clicks "Add Widget" → creates instance
3. **Rendering**: Vue renders widget component in grid
4. **Configuration**: User opens settings → modal shows config form
5. **Data Loading**: Widget calls API endpoint
6. **Auto-Refresh**: Timer triggers periodic refreshes
7. **Persistence**: Layout/config saved to database
8. **Removal**: User deletes widget → removed from grid and DB

### Adding a New Widget (Step-by-Step)

**Step 1: Create Widget Component**

```vue
<!-- src/components/widgets/MyWidget.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import BaseWidget from '../BaseWidget.vue';
import { fetchMyData } from '../../api/widgets';

interface Props {
	config: { myParam: string };
	refreshInterval?: number;
}

const props = defineProps<Props>();
const data = ref(null);
const loading = ref(false);
const error = ref(null);

const loadData = async () => {
	loading.value = true;
	try {
		data.value = await fetchMyData(props.config.myParam);
	} catch (err) {
		error.value = err.message;
	} finally {
		loading.value = false;
	}
};

onMounted(loadData);
</script>

<template>
	<BaseWidget
		title="My Widget"
		:loading="loading"
		:error="error"
		@refresh="loadData"
		@configure="$emit('configure')"
		@remove="$emit('remove')"
	>
		<div>{{ data }}</div>
	</BaseWidget>
</template>
```

**Step 2: Add API Function**

```typescript
// src/api/widgets.ts
export const fetchMyData = async (param: string) => {
	const response = await apiClient.get('/data/my-widget', {
		params: { param },
	});
	return response.data;
};
```

**Step 3: Register Widget**

```typescript
// src/composables/widgetRegistry.ts
import MyWidget from '../components/widgets/MyWidget.vue';

widgetStore.registerWidget({
	id: 'my-widget',
	name: 'My Custom Widget',
	description: 'Does something awesome',
	icon: '🚀',
	component: MyWidget,
	defaultConfig: { myParam: '' },
	configSchema: [
		{
			key: 'myParam',
			label: 'Parameter',
			type: 'text',
			required: true,
		},
	],
	defaultSize: { w: 4, h: 3 },
	refreshInterval: 300,
});
```

**Step 4: Implement Backend Endpoint**

```rust
// backend/src/widgets/my_widget.rs
pub async fn my_widget_handler(
    Query(params): Query<MyWidgetQuery>,
) -> impl IntoResponse {
    // Fetch data from external API or database
    let data = fetch_external_data(&params.param).await?;
    Json(data)
}
```

**Step 5: Done!**

Widget automatically appears in "Add Widget" picker and works like all others.

### Common Patterns & Best Practices

#### Auto-Refresh Pattern

```typescript
let refreshTimer: number | null = null;

const setupAutoRefresh = () => {
	if (refreshTimer) clearInterval(refreshTimer);

	if (props.refreshInterval > 0) {
		refreshTimer = window.setInterval(() => {
			loadData();
		}, props.refreshInterval * 1000);
	}
};

// Watch for changes to refreshInterval
watch(() => props.refreshInterval, setupAutoRefresh, { immediate: true });

// Cleanup on unmount
onBeforeUnmount(() => {
	if (refreshTimer) clearInterval(refreshTimer);
});
```

#### Error Handling Pattern

```typescript
const loadData = async () => {
	loading.value = true;
	error.value = null; // Clear previous errors

	try {
		data.value = await fetchData();
	} catch (err: any) {
		error.value = err.message || 'Failed to load data';
		console.error('Widget error:', err);
	} finally {
		loading.value = false;
	}
};
```

#### Debounced Auto-Save Pattern

```typescript
let autoSaveTimer: number | null = null;

const scheduleAutoSave = () => {
	if (autoSaveTimer) clearTimeout(autoSaveTimer);

	// Wait 2 seconds after last change before saving
	autoSaveTimer = window.setTimeout(() => {
		saveDashboard();
	}, 2000);
};
```

### Troubleshooting

#### Widget Not Appearing

-   ✅ Check widget is registered in `widgetRegistry.ts`
-   ✅ Verify `initializeWidgets()` called in `main.ts`
-   ✅ Check component import path is correct

#### API Calls Failing

-   ✅ Verify backend endpoint exists and is running
-   ✅ Check Vite proxy configuration in `vite.config.ts`
-   ✅ Inspect Network tab in browser DevTools
-   ✅ Ensure authentication token is being sent

#### GridStack Layout Not Saving

-   ✅ Check `gridInstance.value.on('change')` is set up
-   ✅ Verify `saveDashboard()` updates correct dashboard
-   ✅ Check database JSONB column accepts layout structure

#### Widget Not Refreshing

-   ✅ Verify `refreshInterval` prop is passed to widget
-   ✅ Check `setupAutoRefresh()` is called with correct interval
-   ✅ Ensure interval timer is cleared on unmount

### Performance Considerations

#### Lazy Loading Widgets

```typescript
// Dynamically import widgets only when needed
const GitHubWidget = defineAsyncComponent(
	() => import('./components/widgets/GitHubWidget.vue'),
);
```

#### Debouncing Grid Updates

```typescript
// Don't save on every pixel movement
let saveTimeout = null;
gridInstance.value.on('change', () => {
	clearTimeout(saveTimeout);
	saveTimeout = setTimeout(saveDashboard, 2000);
});
```

#### Caching API Responses

Backend already caches responses with Redis. Frontend can add additional caching:

```typescript
const cache = new Map();

const fetchWithCache = async (key, fetcher, ttl = 300000) => {
	const cached = cache.get(key);
	if (cached && Date.now() - cached.timestamp < ttl) {
		return cached.data;
	}

	const data = await fetcher();
	cache.set(key, { data, timestamp: Date.now() });
	return data;
};
```

---

## Conclusion

You now understand:

-   ✅ How Vue 3 reactive system works
-   ✅ How TypeScript provides safety
-   ✅ How Pinia manages global state
-   ✅ How the API layer centralizes HTTP calls
-   ✅ How Vue Router handles navigation
-   ✅ How components communicate
-   ✅ How authentication flows through the app
-   ✅ **How the widget system works with GridStack**
-   ✅ **How to add new widgets to the dashboard**
-   ✅ **How configuration and persistence work**

**The architecture is modular, type-safe, and scalable.** Each piece has a clear responsibility, making the codebase maintainable as it grows.

**Phase 7 Complete!** The dashboard editor with drag-and-drop widgets is fully functional.

**Next Steps:**

-   Testing and CI/CD (Phase 10)
-   Deployment preparation (Phase 11)
-   Production optimization and monitoring
