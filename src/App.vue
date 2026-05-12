<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import ActivityBar from './components/ActivityBar.vue';
import Sidebar from './components/Sidebar.vue';
import ResourceTree from './components/ResourceTree.vue';
import Gauge from './components/Gauge.vue';
import ClusterAccordion from './components/ClusterAccordion.vue';
import TrendChart from './components/TrendChart.vue';
import InspectorPanel from './components/InspectorPanel.vue';
import AdviceBanner from './components/AdviceBanner.vue';

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

interface ClusterContext {
  name: string;
  is_current: boolean;
}

interface Advice {
  action: string;
  reason: string;
}

const activeTab = ref('explorer');
const metrics = ref<Metrics>({ 
  cpu_usage: 0, 
  ram_usage: 0,
  gpu_usage: null,
  gpu_mem_usage: null
});

const availableContexts = ref<ClusterContext[]>([]);
const clusterPods = ref<Record<string, Pod[]>>({});
const currentAdvice = ref<Advice | null>(null);

const selectedPod = ref<{ contextName: string, namespace: string, name: string } | null>(null);
const inspectorPanelRef = ref<InstanceType<typeof InspectorPanel> | null>(null);

const handleSelectPod = async (contextName: string, namespace: string, name: string) => {
  selectedPod.value = { contextName, namespace, name };
  
  if (inspectorPanelRef.value) {
    inspectorPanelRef.value.clearLogs();
  }
  
  try {
    await invoke('start_log_stream', { contextName, namespace, podName: name });
  } catch (e) {
    console.error('Failed to start log stream:', e);
  }
};

const applyOptimization = async () => {
  if (!currentAdvice.value) return;
  
  try {
    // Scale all pods to 0 as suggested in the task
    for (const contextName in clusterPods.value) {
      for (const pod of clusterPods.value[contextName]) {
        await invoke('scale_workload', { 
          contextName,
          namespace: pod.namespace, 
          name: pod.name, 
          replicas: 0 
        });
      }
    }
    currentAdvice.value = null;
    
    // Refresh all pod lists
    for (const context of availableContexts.value) {
      clusterPods.value[context.name] = await invoke<Pod[]>('get_pods', { contextName: context.name });
    }
  } catch (e) {
    console.error('Failed to apply optimization:', e);
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

  // Listen to AI advice
  await listen<Advice>('smart-advice', (event) => {
    currentAdvice.value = event.payload;
  });

  // Fetch initial contexts and pods
  try {
    availableContexts.value = await invoke<ClusterContext[]>('get_available_contexts');
    for (const context of availableContexts.value) {
      clusterPods.value[context.name] = await invoke<Pod[]>('get_pods', { contextName: context.name });
    }
  } catch (e) {
    console.error('Failed to fetch contexts or pods:', e);
  }
});
</script>

<template>
  <div class="ide-container">
    <ActivityBar v-model:activeId="activeTab" />
    
    <Sidebar :title="activeTab">
      <div v-if="activeTab === 'explorer'" class="explorer-content">
        <ResourceTree @select="(type) => console.log('Selected resource type:', type)" />
        
        <div class="cluster-divider">Clusters</div>
        <div class="cluster-list">
          <ClusterAccordion 
            v-for="context in availableContexts" 
            :key="context.name"
            :context-name="context.name"
            :is-current="context.is_current"
            :pods="clusterPods[context.name] || []"
            @select-pod="handleSelectPod"
          />
        </div>
      </div>
      <div v-else-if="activeTab === 'hardware'" class="telemetry-panel">
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
      </div>
      <div v-else class="placeholder-content">
        {{ activeTab }} content coming soon...
      </div>
    </Sidebar>

    <main class="main-area">
      <header class="top-search">
        <div class="brand">StreamK8s | OS v0.1</div>
        <div class="search-placeholder">Search or execute command...</div>
        <div class="system-time">{{ new Date().toLocaleTimeString() }}</div>
      </header>
      
      <div class="content-viewport">
        <section class="workloads-panel">
          <AdviceBanner :advice="currentAdvice" @optimize="applyOptimization" />
          <div class="welcome-screen" v-if="!selectedPod">
            <h2>StreamK8s Orchestrator</h2>
            <p>Select a pod from the explorer to view logs and details.</p>
          </div>
          <InspectorPanel v-show="selectedPod" ref="inspectorPanelRef" :selected-pod="selectedPod" />
        </section>
      </div>
    </main>
  </div>
</template>

<style>
body { margin: 0; padding: 0; background-color: #111827; overflow: hidden; }

.ide-container {
  display: grid;
  grid-template-columns: 48px 260px 1fr;
  height: 100vh;
  color: #f3f4f6;
  font-family: 'Inter', system-ui, sans-serif;
}

.main-area {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background-color: #030712;
}

.top-search {
  height: 48px;
  background-color: #1f2937;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 1.5rem;
  border-bottom: 1px solid #374151;
}

.brand { font-weight: 700; color: #3b82f6; letter-spacing: 1px; }
.search-placeholder { 
  background-color: #111827;
  border: 1px solid #374151;
  border-radius: 4px;
  padding: 4px 12px;
  font-size: 0.8rem;
  color: #6b7280;
  width: 40%;
  text-align: center;
}
.system-time { font-family: monospace; color: #9ca3af; }

.content-viewport {
  flex: 1;
  padding: 1rem;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.workloads-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.explorer-content, .telemetry-panel {
  padding: 4px 8px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.cluster-divider {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
  padding: 4px 8px;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  color: #6b7280;
  border-top: 1px solid #1f2937;
}

.cluster-list {
  flex: 1;
  overflow-y: auto;
}

.telemetry-panel h3 { margin-top: 0; margin-bottom: 1rem; font-size: 1rem; }
.metric-group {
  margin-bottom: 1rem;
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

.welcome-screen {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #4b5563;
}

.placeholder-content {
  padding: 20px;
  color: #6b7280;
  text-align: center;
}
</style>

