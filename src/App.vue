<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import Gauge from './components/Gauge.vue';
import PodList from './components/PodList.vue';

interface Metrics {
  cpu_usage: number;
  ram_usage: number;
}
interface Pod {
  name: string;
  namespace: string;
  status: string;
}

const metrics = ref<Metrics>({ cpu_usage: 0, ram_usage: 0 });
const pods = ref<Pod[]>([]);

onMounted(async () => {
  // Listen to hardware updates
  await listen<Metrics>('hardware-update', (event) => {
    metrics.value = event.payload;
  });

  // Fetch initial pods
  try {
    pods.value = await invoke<Pod[]>('get_pods');
  } catch (e) {
    console.error('Failed to fetch pods:', e);
  }
});
</script>

<template>
  <div class="dashboard-os">
    <header class="top-bar">
      <div class="brand">StreamK8s | OS v0.1</div>
      <div class="system-time">{{ new Date().toLocaleTimeString() }}</div>
    </header>
    <div class="content-area">
      <section class="telemetry-panel">
        <h3>System Telemetry</h3>
        <Gauge label="CPU Usage" :value="metrics.cpu_usage" color="#3b82f6" />
        <Gauge label="RAM Usage" :value="metrics.ram_usage" color="#10b981" />
      </section>
      <section class="workloads-panel">
        <PodList :pods="pods" />
      </section>
    </div>
  </div>
</template>

<style>
body { margin: 0; padding: 0; background-color: #111827; }
.dashboard-os {
  display: flex;
  flex-direction: column;
  height: 100vh;
  color: #f3f4f6;
  font-family: 'Inter', system-ui, sans-serif;
}
.top-bar {
  height: 48px;
  background-color: #1f2937;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 1.5rem;
  border-bottom: 1px solid #374151;
}
.brand { font-weight: 700; color: #3b82f6; letter-spacing: 1px; }
.system-time { font-family: monospace; color: #9ca3af; }
.content-area {
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 1.5rem;
  padding: 1.5rem;
  flex: 1;
  overflow: hidden; /* Prevent body scroll, use container scroll instead */
}
.telemetry-panel {
  background-color: #1f2937;
  padding: 1.5rem;
  border-radius: 8px;
  border: 1px solid #374151;
  align-self: start;
}
.workloads-panel {
  height: 100%;
  overflow: hidden;
}
.telemetry-panel h3 { margin-top: 0; margin-bottom: 1.5rem; }
</style>
