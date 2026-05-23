<script setup lang="ts">
import { ref, onMounted, computed, nextTick } from 'vue';
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
  else if (id === 'ai') currentView.value = 'cluster';
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
    if (selectedContextName.value) {
      fetchResources(selectedContextName.value, activeResourceKind.value);
    }
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
  if (currentView.value === 'cluster') {
    return `48px 48px ${sidebarWidth.value}px 1fr`;
  }
  // Use 0px tracks for hidden sidebars to keep 4-pane structure consistent in DOM
  return `48px 0px 0px 1fr`;
});
</script>

<template>
  <div class="ide-container" :style="{ gridTemplateColumns: gridColumns }">
    <!-- Pane 1: Activity Bar -->
    <ActivityBar :active-id="activeTab" @update:active-id="handleTabChange" />
    
    <!-- Pane 2: Cluster Hotbar -->
    <ClusterHotbar 
      v-show="currentView === 'cluster'"
      :contexts="availableContexts" 
      :active-name="selectedContextName" 
      @select="(name) => { selectedContextName = name; fetchResources(name, activeResourceKind); }" 
    />
    
    <!-- Pane 3: Sidebar -->
    <Sidebar v-show="currentView === 'cluster'" :title="activeTab" :width="sidebarWidth" @start-resize="handleStartResize">
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

    <!-- Pane 4: Main Area -->
    <main class="main-area">
      <!-- Standardized Persistent Header -->
      <header class="main-header">
        <div class="header-left">
          <span class="brand-text">StreamK8s</span>
          <span class="view-indicator" v-if="currentView !== 'welcome'">/ {{ currentView }}</span>
        </div>
        <div class="header-center">
          <div class="search-box" @click="showCommandPalette = true">
            <span class="search-text">Search clusters, resources or actions...</span>
            <span class="search-shortcut">Ctrl+P</span>
          </div>
        </div>
        <div class="header-right">
          <div class="cluster-status" v-if="selectedContextName">
            <span class="status-dot"></span>
            {{ selectedContextName }}
          </div>
          <div class="time-display">{{ new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}</div>
        </div>
      </header>

      <div class="content-viewport">
        <!-- View Switcher -->
        <WelcomeView v-if="currentView === 'welcome'" @start="handleTabChange('explorer')" />
        <SettingsView v-else-if="currentView === 'settings'" />
        <MarketplaceView v-else-if="currentView === 'marketplace'" />
        
        <section v-else-if="currentView === 'cluster'" class="workloads-panel">
          <AdviceBanner :advice="currentAdvice" @optimize="() => {}" />
          
          <div class="main-scroll-area">
            <div v-if="selectedContextName" class="focused-cluster-view">
              <header class="view-title-bar">
                <h2>{{ activeResourceKind }}</h2>
                <button class="btn-refresh" @click="fetchResources(selectedContextName!, activeResourceKind)">Refresh</button>
              </header>
              <ResourceTable 
                :rows="clusterResources[selectedContextName] || []" 
                :context-name="selectedContextName"
                :kind="activeResourceKind"
                @select-resource="handleSelectResource" 
              />
            </div>
            <div v-else class="no-cluster-selected">
              <div class="empty-state">
                <span class="empty-icon">⎈</span>
                <p>Select a cluster from the hotbar to begin</p>
              </div>
            </div>
          </div>

          <!-- Bottom Panel Integration -->
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
/* Global Styles */
body { margin: 0; padding: 0; background-color: #030712; color: #f3f4f6; font-family: 'Inter', system-ui, sans-serif; overflow: hidden; }

.ide-container {
  display: grid;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.main-area {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background-color: #030712;
}

/* New Standardized Header */
.main-header {
  height: 40px;
  background-color: #111827;
  border-bottom: 1px solid #1f2937;
  display: flex;
  align-items: center;
  padding: 0 1rem;
  justify-content: space-between;
  z-index: 100;
}

.header-left { display: flex; align-items: center; gap: 10px; }
.brand-text { font-weight: 900; font-size: 0.85rem; color: #3b82f6; letter-spacing: 0.5px; }
.view-indicator { color: #6b7280; font-size: 0.7rem; text-transform: uppercase; font-weight: 600; }

.header-center { flex: 1; display: flex; justify-content: center; }
.search-box {
  width: 400px;
  max-width: 50%;
  height: 24px;
  background-color: #030712;
  border: 1px solid #374151;
  border-radius: 4px;
  display: flex;
  align-items: center;
  padding: 0 10px;
  justify-content: space-between;
  cursor: pointer;
  transition: all 0.2s;
}
.search-box:hover { border-color: #3b82f6; background-color: #0a0f1e; }
.search-text { font-size: 0.7rem; color: #6b7280; }
.search-shortcut { font-size: 0.65rem; color: #4b5563; font-weight: 700; background: #111827; padding: 1px 4px; border-radius: 2px; }

.header-right { display: flex; align-items: center; gap: 1.5rem; }
.cluster-status { font-size: 0.75rem; color: #9ca3af; display: flex; align-items: center; gap: 8px; font-weight: 600; }
.status-dot { width: 8px; height: 8px; background-color: #10b981; border-radius: 50%; box-shadow: 0 0 8px #10b981; }
.time-display { font-family: monospace; font-size: 0.75rem; color: #6b7280; }

.content-viewport {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.workloads-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.main-scroll-area {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.view-title-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}
.view-title-bar h2 { margin: 0; font-size: 1.5rem; font-weight: 800; letter-spacing: -0.025em; }

.floating-inspector {
  height: 350px;
  border-top: 1px solid #1f2937;
}

.btn-refresh {
  background-color: #1f2937;
  border: 1px solid #374151;
  color: #d1d5db;
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-refresh:hover { background-color: #374151; color: white; }

.no-cluster-selected {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
.empty-state { text-align: center; color: #4b5563; }
.empty-icon { font-size: 4rem; display: block; margin-bottom: 1rem; opacity: 0.5; }

/* Scrollbar Customization */
::-webkit-scrollbar { width: 10px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #1f2937; border-radius: 5px; border: 2px solid #030712; }
::-webkit-scrollbar-thumb:hover { background: #374151; }
</style>
