<template>
	<div class="mx-auto w-full max-w-7xl flex-1 px-4 py-8 sm:px-6 sm:py-12">
		<header
			class="mb-8 flex animate-rise flex-wrap items-end justify-between gap-4 sm:mb-10"
		>
			<div class="min-w-0">
				<h1 class="text-3xl font-semibold tracking-tight sm:text-4xl">
					My Dashboards
				</h1>
				<p class="mt-2 text-base-content/70">
					Organize and manage your data visualizations
				</p>
			</div>
			<button
				type="button"
				class="btn btn-primary shadow-sm"
				:aria-disabled="creating"
				@click="handleCreateDashboard"
			>
				<span
					v-if="creating"
					class="loading loading-spinner loading-sm"
					aria-hidden="true"
				></span>
				<AppIcon v-else name="plus" class="size-4" />
				New Dashboard
			</button>
		</header>

		<div
			v-if="dashboardStore.actionError"
			class="alert alert-error alert-soft mb-6"
			role="alert"
		>
			<AppIcon name="alert-circle" class="size-5" />
			<span class="text-sm">{{ dashboardStore.actionError }}</span>
		</div>

		<div
			v-if="dashboardStore.listLoading && !dashboardStore.dashboards.length"
			role="status"
		>
			<span class="sr-only">Loading your dashboards…</span>
			<div
				class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3"
				aria-hidden="true"
			>
				<div v-for="n in 3" :key="n" class="surface overflow-hidden">
					<div class="skeleton aspect-[16/7] rounded-none"></div>
					<div class="flex flex-col gap-3 p-4">
						<div class="skeleton h-5 w-2/3"></div>
						<div class="skeleton h-3.5 w-1/2"></div>
					</div>
				</div>
			</div>
		</div>

		<div
			v-else-if="dashboardStore.listError"
			class="alert alert-error alert-soft"
			role="alert"
		>
			<AppIcon name="alert-circle" class="size-5" />
			<div>
				<h2 class="font-semibold">Error loading dashboards</h2>
				<p class="text-sm">{{ dashboardStore.listError }}</p>
			</div>
			<button
				type="button"
				class="btn btn-sm"
				@click="dashboardStore.fetchDashboards"
			>
				<AppIcon name="refresh" class="size-4" />
				Retry
			</button>
		</div>

		<div
			v-else-if="!dashboardStore.dashboards.length"
			class="surface mx-auto flex max-w-xl animate-rise flex-col items-center gap-4 border-dashed px-6 py-12 text-center sm:py-16"
		>
			<EmptyIllustration class="max-w-60" />
			<h2 class="text-2xl font-semibold tracking-tight">No dashboards yet</h2>
			<p class="max-w-sm text-base-content/70">
				Create your first dashboard to start visualizing your data and insights.
			</p>
			<button
				type="button"
				class="btn btn-primary mt-2"
				:aria-disabled="creating"
				@click="handleCreateDashboard"
			>
				<AppIcon name="plus" class="size-4" />
				Create Dashboard
			</button>
		</div>

		<ul v-else class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
			<li
				v-for="(dashboard, index) in dashboardStore.dashboards"
				:key="dashboard.id"
				class="group surface relative flex animate-rise flex-col overflow-hidden transition-[transform,box-shadow,border-color] duration-200 focus-within:border-primary/50 hover:-translate-y-0.5 hover:border-primary/40 hover:shadow-float"
				:style="{ animationDelay: `${Math.min(index, 8) * 40}ms` }"
			>
				<div
					class="relative aspect-[16/7] border-b border-base-300 bg-base-200/60 p-3"
				>
					<div
						v-if="dashboard.layout.widgets.length"
						class="relative size-full"
					>
						<LayoutThumbnail :layout="dashboard.layout" />
					</div>
					<p
						v-else
						class="grid h-full place-items-center rounded-lg border border-dashed border-base-300 text-xs text-base-content/70"
					>
						Empty canvas
					</p>
				</div>

				<div class="flex flex-1 flex-col gap-3 p-4">
					<div class="flex min-h-7 items-center justify-between gap-3">
						<h2 class="line-clamp-2 break-words font-semibold leading-snug">
							{{ dashboard.name }}
						</h2>
						<div
							v-if="widgetTypes(dashboard).length"
							class="flex shrink-0 -space-x-1.5"
						>
							<span
								v-for="type in widgetTypes(dashboard)"
								:key="type"
								class="flex rounded-md bg-base-100 ring-2 ring-base-100"
							>
								<WidgetTypeIcon :type="type" size="sm" />
							</span>
						</div>
					</div>

					<dl
						class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-base-content/70"
					>
						<div class="flex items-center gap-1.5">
							<dt class="sr-only">Widgets</dt>
							<AppIcon name="dashboards" class="size-3.5" />
							<dd class="tabular-nums">
								{{ dashboard.layout.widgets.length }}
								{{
									dashboard.layout.widgets.length === 1 ? 'widget' : 'widgets'
								}}
							</dd>
						</div>
						<div class="flex items-center gap-1.5">
							<dt class="sr-only">Updated</dt>
							<AppIcon name="clock" class="size-3.5" />
							<dd>
								<time
									:datetime="dashboard.updated_at"
									:title="new Date(dashboard.updated_at).toLocaleString()"
								>
									Updated {{ formatDate(dashboard.updated_at) }}
								</time>
							</dd>
						</div>
					</dl>

					<div class="mt-auto flex items-center gap-2 pt-1">
						<router-link
							:to="`/dashboards/${dashboard.id}`"
							class="btn btn-sm btn-soft btn-primary flex-1 after:absolute after:inset-0 after:content-['']"
						>
							Open<span class="sr-only"> {{ dashboard.name }}</span>
							<AppIcon
								name="arrow-right"
								class="size-4 transition-transform duration-200 group-hover:translate-x-0.5"
							/>
						</router-link>
						<button
							type="button"
							class="btn btn-ghost btn-sm btn-square relative z-10 text-base-content/70 hover:bg-error/10 hover:text-error"
							:title="`Delete ${dashboard.name}`"
							:aria-disabled="deletingId === dashboard.id"
							@click="handleDeleteDashboard(dashboard.id)"
						>
							<span
								v-if="deletingId === dashboard.id"
								class="loading loading-spinner loading-xs"
								aria-hidden="true"
							></span>
							<AppIcon v-else name="trash" class="size-4" />
							<span class="sr-only">Delete {{ dashboard.name }}</span>
						</button>
					</div>
				</div>
			</li>
		</ul>
	</div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useDashboardStore } from '../stores/dashboard';
import type { Dashboard, WidgetType as WidgetKind } from '../types';
import AppIcon from '../components/ui/AppIcon.vue';
import EmptyIllustration from '../components/ui/EmptyIllustration.vue';
import WidgetTypeIcon from '../components/ui/WidgetTypeIcon.vue';
import LayoutThumbnail from '../components/LayoutThumbnail.vue';

const router = useRouter();
const dashboardStore = useDashboardStore();

const creating = ref(false);
const deletingId = ref<string | null>(null);

onMounted(() => dashboardStore.fetchDashboards());

// Distinct widget types in layout order
const widgetTypes = (dashboard: Dashboard): WidgetKind[] => [
	...new Set(dashboard.layout.widgets.map((w) => w.type)),
];

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
	const minutes = Math.floor(Math.max(0, Date.now() - date.getTime()) / 60_000);

	if (minutes < 1) return 'just now';
	if (minutes < 60) return `${minutes}m ago`;
	if (minutes < 24 * 60) return `${Math.floor(minutes / 60)}h ago`;
	const days = Math.floor(minutes / (24 * 60));
	if (days === 1) return 'yesterday';
	if (days < 7) return `${days}d ago`;
	return date.toLocaleDateString(undefined, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
	});
};
</script>
