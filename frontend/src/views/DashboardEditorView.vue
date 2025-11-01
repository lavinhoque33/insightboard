<template>
	<div class="min-h-screen bg-gray-100">
		<div class="max-w-7xl mx-auto py-12 px-4 sm:px-6 lg:px-8">
			<div class="bg-white shadow rounded-lg p-8 text-center">
				<h2 class="text-2xl font-bold text-gray-900 mb-4">
					Dashboard Editor
				</h2>
				<p class="text-gray-600 mb-4">
					Dashboard ID: {{ dashboardId }}
				</p>
				<p class="text-gray-500">
					This view will be implemented in Phase 7 with GridStack
					integration and widget management.
				</p>
				<router-link
					to="/dashboards"
					class="mt-6 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700"
				>
					Back to Dashboards
				</router-link>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';

const route = useRoute();
const dashboardId = route.params.id as string;
</script>
