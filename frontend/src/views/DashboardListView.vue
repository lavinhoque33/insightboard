<template>
	<div class="flex-1 bg-base-100">
		<header class="border-b border-base-300 bg-base-200">
			<div
				class="container mx-auto flex flex-wrap items-center justify-between gap-4 px-4 py-6"
			>
				<div class="min-w-0">
					<h1
						class="mb-1 bg-linear-to-r from-primary to-secondary bg-clip-text text-3xl font-bold text-transparent sm:text-4xl"
					>
						My Dashboards
					</h1>
					<p class="text-base-content/70">
						Organize and manage your data visualizations
					</p>
				</div>
				<button
					type="button"
					class="btn btn-primary"
					:aria-disabled="creating"
					@click="handleCreateDashboard"
				>
					<svg
						aria-hidden="true"
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
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
					New Dashboard
				</button>
			</div>
		</header>

		<div class="container mx-auto px-4 py-8">
			<div
				v-if="dashboardStore.actionError"
				class="alert alert-error mb-6"
				role="alert"
			>
				<span class="text-sm">{{ dashboardStore.actionError }}</span>
			</div>

			<div
				v-if="dashboardStore.listLoading && !dashboardStore.dashboards.length"
				class="flex flex-col items-center justify-center py-24"
				role="status"
			>
				<span
					class="loading loading-spinner loading-lg text-primary"
					aria-hidden="true"
				></span>
				<p class="mt-4 text-lg text-base-content/70">
					Loading your dashboards…
				</p>
			</div>

			<div
				v-else-if="dashboardStore.listError"
				class="alert alert-error"
				role="alert"
			>
				<div>
					<h2 class="font-bold">Error loading dashboards</h2>
					<p class="text-sm">{{ dashboardStore.listError }}</p>
				</div>
				<button
					type="button"
					class="btn btn-sm"
					@click="dashboardStore.fetchDashboards"
				>
					Retry
				</button>
			</div>

			<div
				v-else-if="!dashboardStore.dashboards.length"
				class="mx-auto flex max-w-md flex-col items-center gap-4 py-24 text-center"
			>
				<h2 class="text-2xl font-bold">No dashboards yet</h2>
				<p class="text-base-content/70">
					Create your first dashboard to start visualizing your data and
					insights.
				</p>
				<button
					type="button"
					class="btn btn-primary"
					:aria-disabled="creating"
					@click="handleCreateDashboard"
				>
					Create Dashboard
				</button>
			</div>

			<ul v-else class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
				<li
					v-for="dashboard in dashboardStore.dashboards"
					:key="dashboard.id"
					class="card card-border bg-base-100 shadow-sm transition-shadow hover:shadow-md"
				>
					<div class="card-body">
						<h2 class="card-title line-clamp-2 break-words">
							{{ dashboard.name }}
						</h2>

						<dl class="my-2 grid grid-cols-2 gap-3">
							<div class="rounded-box bg-base-200 p-3">
								<dt class="text-xs text-base-content/70">Widgets</dt>
								<dd class="text-2xl font-bold text-primary">
									{{ dashboard.layout.widgets.length }}
								</dd>
							</div>
							<div class="rounded-box bg-base-200 p-3">
								<dt class="text-xs text-base-content/70">Created</dt>
								<dd class="text-sm font-semibold">
									<time :datetime="dashboard.created_at">
										{{ formatDate(dashboard.created_at) }}
									</time>
								</dd>
							</div>
						</dl>

						<div class="card-actions">
							<router-link
								:to="`/dashboards/${dashboard.id}`"
								class="btn btn-primary btn-sm flex-1"
							>
								Open<span class="sr-only"> {{ dashboard.name }}</span>
							</router-link>
							<button
								type="button"
								class="btn btn-ghost btn-sm text-error"
								:aria-disabled="deletingId === dashboard.id"
								@click="handleDeleteDashboard(dashboard.id)"
							>
								Delete<span class="sr-only"> {{ dashboard.name }}</span>
							</button>
						</div>
					</div>
				</li>
			</ul>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useDashboardStore } from '../stores/dashboard';

const router = useRouter();
const dashboardStore = useDashboardStore();

const creating = ref(false);
const deletingId = ref<string | null>(null);

onMounted(() => dashboardStore.fetchDashboards());

const handleCreateDashboard = async () => {
	if (creating.value) return;
	creating.value = true;
	try {
		const dashboard = await dashboardStore.createDashboard('New Dashboard');
		await router.push(`/dashboards/${dashboard.id}`);
	} catch {
		// The store keeps the message the page shows
	} finally {
		creating.value = false;
	}
};

const handleDeleteDashboard = async (id: string) => {
	if (deletingId.value) return;
	if (
		!confirm(
			'Are you sure you want to delete this dashboard? This action cannot be undone.',
		)
	) {
		return;
	}

	deletingId.value = id;
	try {
		await dashboardStore.deleteDashboard(id);
	} catch {
		// The store keeps the message the page shows
	} finally {
		deletingId.value = null;
	}
};

const formatDate = (dateString: string): string => {
	const date = new Date(dateString);
	const diffDays = Math.floor(
		Math.abs(Date.now() - date.getTime()) / (1000 * 60 * 60 * 24),
	);

	if (diffDays === 0) return 'today';
	if (diffDays === 1) return 'yesterday';
	if (diffDays < 7) return `${diffDays}d ago`;
	return date.toLocaleDateString();
};
</script>
