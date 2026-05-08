<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import Gauge from './components/Gauge.vue';
import PodList from './components/PodList.vue';
import TrendChart from './components/TrendChart.vue';
import LogPanel from './components/LogPanel.vue';

interface Metrics {
  cpu_usage: number;
  ram_usage: number;
  gpu_usage: number | null;
  gpu_mem_usage: number | null;
}
interface Pod {
  name: string;
  namespace: string;
  status: string;
}

const metrics = ref<Metrics>({ 
  cpu_usage: 0, 
  ram_usage: 0,
  gpu_usage: null,
  gpu_mem_usage: null
});

const pods = ref<Pod[]>([]);

const selectedPod = ref<{ namespace: string, name: string } | null>(null);
const logPanelRef = ref<InstanceType<typeof LogPanel> | null>(null);

const handleSelectPod = async (namespace: string, name: string) => {
  selectedPod.value = { namespace, name };
  
  if (logPanelRef.value) {
    logPanelRef.value.clearLogs();
  }
  
  try {
    await invoke('start_log_stream', { namespace, name });
  } catch (e) {
    console.error('Failed to start log stream:', e);
  }
};

// History buffers for trend charts
const cpuHistory = ref<number[]>([]);
const ramHistory = ref<number[]>([]);
const gpuHistory = ref<number[]>([]);

onMounted(async () => {
  // Listen to hardware updates
  await listen<Metrics>('hardware-update', (event) => {
    metrics.value = event.payload;

    // Update history buffers
    const pushAndShift = (arr: number[], val: number) => {
      arr.push(val);
      if (arr.length > 60) arr.shift();
    };

    pushAndShift(cpuHistory.value, event.payload.cpu_usage);
    pushAndShift(ramHistory.value, event.payload.ram_usage);
    if (event.payload.gpu_usage !== null) {
      pushAndShift(gpuHistory.value, event.payload.gpu_usage);
    }
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
    <div class="main-layout">
      <div class="content-area">
        <section class="telemetry-panel">
          <h3>System Telemetry</h3>
          
          <div class="metric-group">
            <Gauge label="CPU Usage" :value="metrics.cpu_usage" color="#3b82f6" />
            <TrendChart :data="cpuHistory" color="#3b82f6" />
          </div>

          <div class="metric-group">
            <Gauge label="RAM Usage" :value="metrics.ram_usage" color="#10b981" />
            <TrendChart :data="ramHistory" color="#10b981" />
          </div>

          <div v-if="metrics.gpu_usage !== null" class="metric-group">
            <Gauge label="GPU Load" :value="metrics.gpu_usage" color="#f59e0b" />
            <TrendChart :data="gpuHistory" color="#f59e0b" />
            <div class="sub-metric">VRAM: {{ metrics.gpu_mem_usage?.toFixed(1) }}%</div>
          </div>
          <div v-else class="gpu-not-found">
            No NVIDIA GPU Detected
          </div>
        </section>
        <section class="workloads-panel">
          <PodList :pods="pods" @select-pod="handleSelectPod" />
        </section>
      </div>
      <footer class="footer-panel">
        <LogPanel ref="logPanelRef" />
      </footer>
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
.main-layout {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}
.content-area {
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 1.5rem;
  padding: 1.5rem;
  flex: 1;
  overflow: hidden; /* Prevent body scroll, use container scroll instead */
}
.footer-panel {
  flex-shrink: 0;
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
.metric-group {
  margin-bottom: 2rem;
  background-color: rgba(255, 255, 255, 0.02);
  padding: 0.75rem;
  border-radius: 6px;
}
.sub-metric {
  font-size: 0.75rem;
  color: #9ca3af;
  margin-top: 0.5rem;
  text-align: right;
  font-family: monospace;
}
.gpu-not-found {
  font-size: 0.75rem;
  color: #6b7280;
  text-align: center;
  padding: 1rem;
  border: 1px dashed #374151;
  border-radius: 4px;
}
</style>
