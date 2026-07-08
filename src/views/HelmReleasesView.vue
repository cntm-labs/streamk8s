<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  namespace: string;
}>();

interface HelmRelease {
  name: string;
  namespace: string;
  revision: string;
  updated: string;
  status: string;
  chart: string;
  app_version: string;
}

const releases = ref<HelmRelease[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const fetchReleases = async () => {
  loading.value = true;
  error.value = null;
  try {
    const ns = props.namespace === 'All Namespaces' ? '' : props.namespace;
    releases.value = await invoke<HelmRelease[]>('list_helm_releases', { namespace: ns });
  } catch (err: any) {
    error.value = err.toString();
    console.error('Failed to fetch helm releases:', err);
  } finally {
    loading.value = false;
  }
};

watch(() => props.namespace, () => {
  fetchReleases();
});

onMounted(() => {
  fetchReleases();
});
</script>

<template>
  <div class="helm-view">
    <div class="header">
      <h2>Helm Releases</h2>
      <button class="refresh-btn" @click="fetchReleases" :disabled="loading">
        {{ loading ? 'Loading...' : 'Refresh' }}
      </button>
    </div>

    <div v-if="error" class="error-banner">
      {{ error }}
    </div>

    <div class="table-container">
      <table class="helm-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Namespace</th>
            <th>Revision</th>
            <th>Updated</th>
            <th>Status</th>
            <th>Chart</th>
            <th>App Version</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="releases.length === 0 && !loading">
            <td colspan="7" class="empty-state">No Helm releases found in this namespace.</td>
          </tr>
          <tr v-for="release in releases" :key="release.name + release.namespace">
            <td class="font-bold text-accent">{{ release.name }}</td>
            <td>{{ release.namespace }}</td>
            <td>{{ release.revision }}</td>
            <td class="text-xs text-dim">{{ release.updated }}</td>
            <td>
              <span class="status-badge" :class="release.status.toLowerCase()">
                {{ release.status }}
              </span>
            </td>
            <td>{{ release.chart }}</td>
            <td>{{ release.app_version }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.helm-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: var(--space-6);
  color: #f3f4f6;
  background-color: var(--surface-dark);
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-6);
}

.header h2 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 800;
}

.refresh-btn {
  background-color: var(--surface-glass);
  border: 1px solid var(--border-dim);
  color: #d1d5db;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  font-size: 0.8rem;
  cursor: pointer;
}

.refresh-btn:hover {
  background-color: rgba(255,255,255,0.1);
}

.error-banner {
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #fca5a5;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-4);
  font-size: 0.85rem;
}

.table-container {
  flex: 1;
  overflow-y: auto;
  border: 1px solid var(--border-dim);
  border-radius: var(--radius-md);
  background-color: rgba(0, 0, 0, 0.2);
}

.helm-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
}

.helm-table th {
  position: sticky;
  top: 0;
  background-color: #1f2937;
  padding: var(--space-3) var(--space-4);
  font-size: 0.75rem;
  text-transform: uppercase;
  color: #9ca3af;
  border-bottom: 1px solid var(--border-dim);
  z-index: 1;
}

.helm-table td {
  padding: var(--space-3) var(--space-4);
  font-size: 0.85rem;
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.helm-table tr:hover {
  background-color: rgba(255,255,255,0.02);
}

.empty-state {
  text-align: center;
  padding: var(--space-8);
  color: #6b7280;
  font-style: italic;
}

.status-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  background-color: #374151;
  color: #d1d5db;
}

.status-badge.deployed {
  background-color: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.status-badge.failed {
  background-color: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.status-badge.pending-install,
.status-badge.pending-upgrade {
  background-color: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.font-bold { font-weight: 600; }
.text-accent { color: #3b82f6; }
.text-xs { font-size: 0.7rem; }
.text-dim { color: #9ca3af; }
</style>
