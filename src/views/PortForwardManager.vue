<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface ForwardSession {
  id: string;
  context: string;
  namespace: string;
  pod: string;
  local_port: number;
  remote_port: number;
}

const activeForwards = ref<ForwardSession[]>([]);
let refreshInterval: number;

const fetchForwards = async () => {
  try {
    activeForwards.value = await invoke<ForwardSession[]>('list_active_forwards');
  } catch (e) {
    console.error('Failed to fetch forwards:', e);
  }
};

const stopForward = async (id: string) => {
  try {
    await invoke('stop_port_forward', { sessionId: id });
    await fetchForwards();
  } catch (e) {
    console.error('Failed to stop forward:', e);
    alert(`Failed to stop: ${e}`);
  }
};

onMounted(() => {
  fetchForwards();
  refreshInterval = window.setInterval(fetchForwards, 3000);
});

onUnmounted(() => {
  clearInterval(refreshInterval);
});
</script>

<template>
  <div class="port-forward-manager">
    <header class="manager-header">
      <h2>Active Port Forwards</h2>
      <button class="btn-refresh" @click="fetchForwards">Refresh</button>
    </header>

    <div v-if="activeForwards.length === 0" class="empty-state">
      No active port forwards. Right-click a Pod to start forwarding.
    </div>

    <div v-else class="forward-list">
      <div v-for="session in activeForwards" :key="session.id" class="forward-card">
        <div class="forward-info">
          <div class="forward-title">
            <span class="context-label">{{ session.context }}</span>
            <span class="pod-name">{{ session.namespace }} / {{ session.pod }}</span>
          </div>
          <div class="forward-ports">
            <a :href="`http://localhost:${session.local_port}`" target="_blank" class="port-link">
              localhost:{{ session.local_port }}
            </a>
            <span class="arrow">→</span>
            <span class="remote-port">Port {{ session.remote_port }}</span>
          </div>
        </div>
        <button class="btn-stop" @click="stopForward(session.id)">Stop</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.port-forward-manager {
  padding: var(--space-6);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-6);
}

.manager-header h2 {
  margin: 0;
  font-size: 1.5rem;
}

.btn-refresh {
  background-color: #1f2937;
  border: 1px solid #374151;
  color: #d1d5db;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.empty-state {
  text-align: center;
  padding: var(--space-10);
  color: #9ca3af;
  border: 1px dashed #374151;
  border-radius: var(--radius-md);
}

.forward-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  overflow-y: auto;
}

.forward-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-4);
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-dim);
  border-radius: var(--radius-md);
}

.context-label {
  font-size: 0.7rem;
  padding: 2px 6px;
  background-color: #374151;
  border-radius: 4px;
  margin-right: 8px;
}

.pod-name {
  font-weight: 600;
}

.forward-ports {
  margin-top: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-code);
  font-size: 0.85rem;
}

.port-link {
  color: #3b82f6;
  text-decoration: none;
}

.port-link:hover {
  text-decoration: underline;
}

.arrow {
  color: #6b7280;
}

.btn-stop {
  background-color: #ef4444;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: 600;
}
.btn-stop:hover {
  background-color: #dc2626;
}
</style>
