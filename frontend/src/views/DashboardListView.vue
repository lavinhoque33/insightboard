<template>
	<div class="min-h-screen bg-gray-50">
		<!-- Header -->
		<header class="bg-white shadow">
			<div
				class="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8 flex justify-between items-center"
			>
				<h1 class="text-3xl font-bold text-gray-900">My Dashboards</h1>
				<button
					@click="handleCreateDashboard"
					:disabled="dashboardStore.loading"
					class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					<svg
						class="h-5 w-5 mr-2"
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 20 20"
						fill="currentColor"
					>
						<path
							fill-rule="evenodd"
							d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
							clip-rule="evenodd"
						/>
					</svg>
					New Dashboard
				</button>
			</div>
		</header>

		<!-- Main Content -->
		<main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
			<!-- Loading State -->
			<div
				v-if="loading && !dashboardStore.dashboards.length"
				class="text-center py-12"
			>
				<div
					class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"
				></div>
				<p class="mt-4 text-gray-600">Loading dashboards...</p>
			</div>

			<!-- Error State -->
			<div
				v-else-if="dashboardStore.error"
				class="rounded-md bg-red-50 p-4 mx-4 sm:mx-0"
			>
				<div class="flex">
					<div class="ml-3">
						<h3 class="text-sm font-medium text-red-800">
							Error loading dashboards: {{ dashboardStore.error }}
						</h3>
						<button
							@click="loadDashboards"
							class="mt-2 text-sm font-medium text-red-600 hover:text-red-500"
						>
							Try again
						</button>
					</div>
				</div>
			</div>

			<!-- Empty State -->
			<div
				v-else-if="!dashboardStore.hasDashboards"
				class="text-center py-12 px-4"
			>
				<svg
					class="mx-auto h-12 w-12 text-gray-400"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
					/>
				</svg>
				<h3 class="mt-2 text-sm font-medium text-gray-900">
					No dashboards
				</h3>
				<p class="mt-1 text-sm text-gray-500">
					Get started by creating a new dashboard.
				</p>
				<div class="mt-6">
					<button
						@click="handleCreateDashboard"
						class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500"
					>
						<svg
							class="h-5 w-5 mr-2"
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 20 20"
							fill="currentColor"
						>
							<path
								fill-rule="evenodd"
								d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
								clip-rule="evenodd"
							/>
						</svg>
						Create Dashboard
					</button>
				</div>
			</div>

			<!-- Dashboard Grid -->
			<div v-else class="px-4 sm:px-0">
				<div
					class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3"
				>
					<div
						v-for="dashboard in dashboardStore.dashboards"
						:key="dashboard.id"
						class="relative group bg-white overflow-hidden shadow rounded-lg hover:shadow-lg transition-shadow duration-200"
					>
						<div class="p-6">
							<h3
								class="text-lg font-medium text-gray-900 truncate"
							>
								{{
									dashboard.settings.title ||
									'Untitled Dashboard'
								}}
							</h3>
							<p class="mt-2 text-sm text-gray-500">
								Created {{ formatDate(dashboard.created_at) }}
							</p>
							<p class="mt-1 text-sm text-gray-500">
								{{ dashboard.layout.widgets.length }} widget{{
									dashboard.layout.widgets.length !== 1
										? 's'
										: ''
								}}
							</p>

							<!-- Actions -->
							<div class="mt-4 flex space-x-3">
								<router-link
									:to="`/dashboards/${dashboard.id}`"
									class="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500"
								>
									Open
								</router-link>
								<button
									@click="handleDeleteDashboard(dashboard.id)"
									:disabled="deleting"
									class="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed"
								>
									Delete
								</button>
							</div>
						</div>
					</div>
				</div>
			</div>
		</main>
	</div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useDashboardStore } from '../stores/dashboard';

const router = useRouter();
const dashboardStore = useDashboardStore();

const loading = ref(false);
const deleting = ref(false);

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
			{ widgets: [] },
			{ title: 'New Dashboard', theme: 'light' },
		);
		// Navigate to the new dashboard editor
		router.push(`/dashboards/${dashboard.id}`);
	} catch (error) {
		console.error('Failed to create dashboard:', error);
	}
};

const handleDeleteDashboard = async (id: string) => {
	if (!confirm('Are you sure you want to delete this dashboard?')) {
		return;
	}

	deleting.value = true;
	try {
		await dashboardStore.deleteDashboard(id);
	} catch (error) {
		console.error('Failed to delete dashboard:', error);
	} finally {
		deleting.value = false;
	}
};

const formatDate = (dateString: string): string => {
	const date = new Date(dateString);
	const now = new Date();
	const diffTime = Math.abs(now.getTime() - date.getTime());
	const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

	if (diffDays === 0) {
		return 'today';
	} else if (diffDays === 1) {
		return 'yesterday';
	} else if (diffDays < 7) {
		return `${diffDays} days ago`;
	} else {
		return date.toLocaleDateString();
	}
};
</script>
