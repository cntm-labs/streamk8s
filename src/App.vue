<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import ActivityBar from './components/ActivityBar.vue';
import ClusterHotbar from './components/ClusterHotbar.vue';
import Sidebar from './components/Sidebar.vue';
import ResourceTree from './components/ResourceTree.vue';
import ResourceTable from './components/ResourceTable.vue';
import Gauge from './components/Gauge.vue';
import TrendChart from './components/TrendChart.vue';
import InspectorPanel from './components/InspectorPanel.vue';
import AdviceBanner from './components/AdviceBanner.vue';
import CommandPalette from './components/CommandPalette.vue';

// Views
import WelcomeView from './views/WelcomeView.vue';
import SettingsView from './views/SettingsView.vue';
import MarketplaceView from './views/MarketplaceView.vue';

interface Metrics {
  cpu_usage: number;
  ram_usage: number;
  gpu_usage: number | null;
  gpu_mem_usage: number | null;
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
const currentView = ref<'welcome' | 'cluster' | 'settings' | 'marketplace'>('welcome');
const sidebarWidth = ref(240);
const isResizing = ref(false);
const showCommandPalette = ref(false);

const handleStartResize = () => {
  isResizing.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
  document.body.style.cursor = 'col-resize';
};

const handleMouseMove = (e: MouseEvent) => {
  if (!isResizing.value) return;
  const newWidth = e.clientX - 96;
  if (newWidth > 150 && newWidth < 600) {
    sidebarWidth.value = newWidth;
  }
};

const handleMouseUp = () => {
  isResizing.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
  document.body.style.cursor = 'default';
};

const metrics = ref<Metrics>({ cpu_usage: 0, ram_usage: 0, gpu_usage: null, gpu_mem_usage: null });
const availableContexts = ref<ClusterContext[]>([]);
const selectedContextName = ref<string | null>(null);
const clusterResources = ref<Record<string, any[]>>({});
const currentAdvice = ref<Advice | null>(null);
const activeResourceKind = ref('Pods');
const selectedResource = ref<any | null>(null);
const inspectorPanelRef = ref<any | null>(null);

const handleTabChange = (id: string) => {
  activeTab.value = id;
  if (id === 'explorer') currentView.value = 'cluster';
  else if (id === 'settings') currentView.value = 'settings';
  else if (id === 'marketplace') currentView.value = 'marketplace';
  else if (id === 'ai') currentView.value = 'cluster'; // Or a dedicated AI view
};

const handleSelectResource = (resource: any) => {
  selectedResource.value = { 
    contextName: selectedContextName.value, 
    namespace: resource.namespace, 
    name: resource.name,
    kind: activeResourceKind.value 
  };
};

const handleCommandSelect = (result: any) => {
  showCommandPalette.value = false;
  if (result.kind === 'Action') {
    if (result.id === 'action_settings') handleTabChange('settings');
    else if (result.id === 'action_explorer') handleTabChange('explorer');
    else if (result.id === 'action_hardware') handleTabChange('hardware');
  } else if (result.kind === 'Cluster') {
    selectedContextName.value = result.context;
    currentView.value = 'cluster';
  }
};

const fetchResources = async (context: string, kind: string) => {
  try {
    let cmd = 'get_pods';
    if (kind === 'Deployments') cmd = 'get_deployments';
    else if (kind === 'Services') cmd = 'get_services';
    else if (kind === 'ConfigMaps') cmd = 'get_configmaps';
    else if (kind === 'Secrets') cmd = 'get_secrets';

    const data = await invoke<any[]>(cmd, { contextName: context, namespace: 'default' });
    clusterResources.value[context] = data;
  } catch (e) {
    console.error(`Failed to fetch ${kind}:`, e);
  }
};

const handleResourceTypeSelect = (kind: string) => {
  activeResourceKind.value = kind;
  if (selectedContextName.value) {
    fetchResources(selectedContextName.value, kind);
  }
};

const cpuHistory = ref<number[]>([]);
const ramHistory = ref<number[]>([]);
const gpuHistory = ref<number[]>([]);

onMounted(async () => {
  window.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'p') {
      e.preventDefault();
      showCommandPalette.value = !showCommandPalette.value;
    }
  });

  await listen<Metrics>('hardware-update', (event) => {
    metrics.value = event.payload;
    const pushAndShift = (arr: number[], val: number) => {
      arr.push(val);
      if (arr.length > 60) arr.shift();
    };
    pushAndShift(cpuHistory.value, event.payload.cpu_usage);
    pushAndShift(ramHistory.value, event.payload.ram_usage);
    if (event.payload.gpu_usage !== null) pushAndShift(gpuHistory.value, event.payload.gpu_usage);
  });

  await listen<Advice>('smart-advice', (event) => {
    currentAdvice.value = event.payload;
  });

  try {
    availableContexts.value = await invoke<ClusterContext[]>('get_available_contexts');
    const current = availableContexts.value.find(c => c.is_current) || availableContexts.value[0];
    if (current) {
      selectedContextName.value = current.name;
      fetchResources(current.name, activeResourceKind.value);
    }
  } catch (e) {
    console.error('Initial fetch failed:', e);
  }
});

const gridColumns = computed(() => {
  if (currentView.value === 'cluster') return `48px 48px ${sidebarWidth.value}px 1fr`;
  return `48px 0px 0px 1fr`; // Hide sidebars for welcome/settings
});
</script>

<template>
  <div class="ide-container" :style="{ gridTemplateColumns: gridColumns }">
    <ActivityBar :active-id="activeTab" @update:active-id="handleTabChange" />
    
    <ClusterHotbar 
      v-if="currentView === 'cluster'"
      :contexts="availableContexts" 
      :active-name="selectedContextName" 
      @select="(name) => { selectedContextName = name; fetchResources(name, activeResourceKind); }" 
    />
    
    <Sidebar v-if="currentView === 'cluster'" :title="activeTab" :width="sidebarWidth" @start-resize="handleStartResize">
      <div v-if="activeTab === 'explorer'" class="explorer-content">
        <div class="active-cluster-label" v-if="selectedContextName">
          <span class="label-icon">⎈</span>
          <span class="label-text">{{ selectedContextName }}</span>
        </div>
        <ResourceTree @select-resource-type="handleResourceTypeSelect" />
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
        <div v-else class="gpu-not-found">No NVIDIA GPU Detected</div>
      </div>
    </Sidebar>

    <main class="main-area">
      <!-- Dynamic Views -->
      <WelcomeView v-if="currentView === 'welcome'" @start="handleTabChange('explorer')" />
      <SettingsView v-else-if="currentView === 'settings'" />
      <MarketplaceView v-else-if="currentView === 'marketplace'" />
      
      <div v-else-if="currentView === 'cluster'" class="content-viewport">
        <header class="top-search">
          <div class="brand">StreamK8s | OS v0.1</div>
          <div class="search-placeholder" @click="showCommandPalette = true">Search or execute command... (Ctrl+P)</div>
          <div class="system-time">{{ new Date().toLocaleTimeString() }}</div>
        </header>

        <section class="workloads-panel">
          <AdviceBanner :advice="currentAdvice" @optimize="() => {}" />
          
          <div class="main-scroll-area">
            <div v-if="selectedContextName" class="focused-cluster-view">
              <header class="cluster-view-header">
                <div class="cluster-title">
                  <span class="k8s-icon">⎈</span>
                  <h2>{{ activeResourceKind }} in {{ selectedContextName }}</h2>
                </div>
                <button class="btn-refresh" @click="fetchResources(selectedContextName!, activeResourceKind)">Refresh</button>
              </header>
              <ResourceTable 
                :rows="clusterResources[selectedContextName] || []" 
                :context-name="selectedContextName"
                :kind="activeResourceKind"
                @select-resource="handleSelectResource" 
              />
            </div>
          </div>

          <div v-if="selectedResource" class="floating-inspector">
             <InspectorPanel 
               ref="inspectorPanelRef" 
               :selected-resource="selectedResource" 
               @close="selectedResource = null"
             />
          </div>
        </section>
      </div>
    </main>

    <CommandPalette 
      v-if="showCommandPalette" 
      :visible="showCommandPalette"
      @close="showCommandPalette = false"
      @select="handleCommandSelect"
    />
  </div>
</template>

<style>
/* Keeping existing global styles */
body { margin: 0; padding: 0; background-color: #111827; overflow: hidden; color: #f3f4f6; }
.ide-container { display: grid; height: 100vh; font-family: 'Inter', sans-serif; overflow: hidden; }
/* ... (Rest of CSS from merged master) */
</style>
