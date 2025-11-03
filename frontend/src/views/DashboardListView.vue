<template>
	<div class="min-h-[calc(100vh-70px)] bg-base-100">
		<!-- Header with Create Button -->
		<header
			class="bg-gradient-to-r from-base-200 to-base-300 shadow-md border-b border-base-300 sticky top-[70px] z-40"
		>
			<div class="container mx-auto px-4 py-6">
				<div class="flex justify-between items-center">
					<div>
						<h1
							class="text-4xl font-bold text-transparent bg-gradient-to-r from-primary to-secondary bg-clip-text mb-2"
						>
							My Dashboards
						</h1>
						<p class="text-base-content/60">
							Organize and manage your data visualizations
						</p>
					</div>
					<button
						@click="handleCreateDashboard"
						:disabled="dashboardStore.loading"
						class="btn btn-primary btn-lg gap-2 shadow-lg"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-6 w-6"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 4v16m8-8H4"
							/>
						</svg>
						<span>New Dashboard</span>
					</button>
				</div>
			</div>
		</header>

		<!-- Main Content -->
		<main class="container mx-auto px-4 py-8">
			<!-- Loading State -->
			<div
				v-if="loading && !dashboardStore.dashboards.length"
				class="flex flex-col items-center justify-center py-32"
			>
				<span
					class="loading loading-spinner loading-lg text-primary"
				></span>
				<p class="mt-4 text-base-content/60 text-lg font-medium">
					Loading your dashboards...
				</p>
			</div>

			<!-- Error State -->
			<transition
				enter-active-class="transition ease-out duration-200"
				enter-from-class="opacity-0 -translate-y-2"
				enter-to-class="opacity-100 translate-y-0"
				leave-active-class="transition ease-in duration-150"
				leave-from-class="opacity-100 translate-y-0"
				leave-to-class="opacity-0 -translate-y-2"
			>
				<div
					v-if="dashboardStore.error"
					class="alert alert-error shadow-2xl"
					role="alert"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="stroke-current shrink-0 h-6 w-6"
						fill="none"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M10 14l-2-2m0 0l-2-2m2 2l2-2m-2 2l-2 2m2-2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
					<div>
						<h3 class="font-bold">Error loading dashboards</h3>
						<div class="text-sm">{{ dashboardStore.error }}</div>
					</div>
					<button
						@click="loadDashboards"
						class="btn btn-sm btn-ghost gap-2"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
							/>
						</svg>
						Retry
					</button>
				</div>
			</transition>

			<!-- Empty State -->
			<div
				v-if="!loading && !dashboardStore.hasDashboards"
				class="flex flex-col items-center justify-center py-32"
			>
				<div class="text-center max-w-md">
					<div
						class="inline-flex items-center justify-center w-24 h-24 rounded-full bg-base-200 mb-6"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-12 w-12 text-base-content/40"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4"
							/>
						</svg>
					</div>
					<h3 class="text-2xl font-bold text-base-content mb-2">
						No dashboards yet
					</h3>
					<p class="text-base-content/60 mb-8">
						Create your first dashboard to start visualizing your
						data and insights.
					</p>
					<button
						@click="handleCreateDashboard"
						class="btn btn-primary btn-lg gap-2"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-6 w-6"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 4v16m8-8H4"
							/>
						</svg>
						Create Dashboard
					</button>
				</div>
			</div>

			<!-- Dashboard Grid -->
			<div
				v-else-if="!loading"
				class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
			>
				<div
					v-for="dashboard in dashboardStore.dashboards"
					:key="dashboard.id"
					class="card bg-base-100 shadow-lg hover:shadow-xl transition-all duration-300 border border-base-300 overflow-hidden group"
				>
					<!-- Card Image/Header -->
					<div
						class="h-32 bg-gradient-to-br from-primary/10 to-secondary/10 flex items-center justify-center"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-16 w-16 text-primary/40 group-hover:text-primary/60 transition-colors"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
							/>
						</svg>
					</div>

					<!-- Card Body -->
					<div class="card-body">
						<!-- Title -->
						<h2
							class="card-title text-xl line-clamp-2 text-base-content"
						>
							{{
								dashboard.settings.title || 'Untitled Dashboard'
							}}
						</h2>

						<!-- Stats -->
						<div class="grid grid-cols-2 gap-3 my-4">
							<div class="stat bg-base-200 rounded-lg p-3">
								<div class="stat-title text-xs">Widgets</div>
								<div class="stat-value text-2xl text-primary">
									{{ dashboard.layout.widgets.length }}
								</div>
							</div>
							<div class="stat bg-base-200 rounded-lg p-3">
								<div class="stat-title text-xs">Created</div>
								<div class="stat-value text-xs">
									<time
										:datetime="dashboard.created_at"
										class="block"
									>
										{{ formatDate(dashboard.created_at) }}
									</time>
								</div>
							</div>
						</div>

						<!-- Divider -->
						<div class="divider my-2"></div>

						<!-- Actions -->
						<div class="card-actions gap-2">
							<router-link
								:to="`/dashboards/${dashboard.id}`"
								class="btn btn-primary btn-sm gap-1 flex-1"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-4 w-4"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
									/>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
									/>
								</svg>
								Open
							</router-link>
							<button
								@click="handleDeleteDashboard(dashboard.id)"
								:disabled="deleting"
								class="btn btn-ghost btn-sm text-error gap-1"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-4 w-4"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
									/>
								</svg>
								Delete
							</button>
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
	if (
		!confirm(
			'Are you sure you want to delete this dashboard? This action cannot be undone.',
		)
	) {
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
		return `${diffDays}d ago`;
	} else {
		return date.toLocaleDateString();
	}
};
</script>
