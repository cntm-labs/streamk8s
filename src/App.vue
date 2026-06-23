<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
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
import YamlEditorModal from './components/YamlEditorModal.vue';
import AdvisorToast from './components/AdvisorToast.vue';

// Views
import WelcomeView from './views/WelcomeView.vue';
import SettingsView from './views/SettingsView.vue';
import MarketplaceView from './views/MarketplaceView.vue';
import TopologyView from './views/TopologyView.vue';

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
const currentView = ref<'welcome' | 'cluster' | 'settings' | 'marketplace' | 'topology'>('welcome');
const sidebarVisible = ref(true);
const sidebarWidth = ref(240);
const isResizing = ref(false);
const showCommandPalette = ref(false);
const editingResource = ref<any | null>(null);

const navMap: Record<string, { sidebar: boolean, view: 'welcome' | 'cluster' | 'settings' | 'marketplace' | 'topology' }> = {
  explorer: { sidebar: true, view: 'cluster' },
  topology: { sidebar: false, view: 'topology' as any },
  hardware: { sidebar: true, view: 'cluster' },
  ai: { sidebar: true, view: 'cluster' },
  marketplace: { sidebar: true, view: 'marketplace' },
  settings: { sidebar: false, view: 'settings' },
};

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
const selectedNamespace = ref('default');
const availableNamespaces = ref<string[]>(['default']);
const clusterResources = ref<Record<string, any[]>>({});
const currentAdvice = ref<Advice | null>(null);
const activeResourceKind = ref('Pods');
const selectedResource = ref<any | null>(null);
const inspectorPanelRef = ref<any | null>(null);

const handleTabChange = (id: string) => {
  const config = navMap[id];
  if (!config) return;

  if (activeTab.value === id) {
    sidebarVisible.value = !sidebarVisible.value;
  } else {
    activeTab.value = id;
    currentView.value = config.view;
    sidebarVisible.value = config.sidebar;
  }
};

const showHotbar = computed(() => currentView.value === 'cluster' || currentView.value === 'marketplace');
const showSidebar = computed(() => sidebarVisible.value && (currentView.value === 'cluster' || currentView.value === 'marketplace'));

const gridColumns = computed(() => {
  const activityBarWidth = '48px';
  const hotbarWidth = showHotbar.value ? '48px' : '0px';
  const sidebarWidthVal = showSidebar.value ? `${sidebarWidth.value}px` : '0px';
  return `${activityBarWidth} ${hotbarWidth} ${sidebarWidthVal} 1fr`;
});

const handleSelectResource = (resource: any) => {
  selectedResource.value = { 
    contextName: selectedContextName.value, 
    namespace: resource.namespace, 
    name: resource.name,
    kind: activeResourceKind.value 
  };
};

const fetchNamespaces = async (context: string) => {
  try {
    availableNamespaces.value = await invoke<string[]>('get_namespaces', { contextName: context });
  } catch (e) {
    console.error('Failed to fetch namespaces:', e);
    availableNamespaces.value = ['default'];
  }
};

watch(selectedNamespace, () => {
  if (selectedContextName.value) {
    fetchResources(selectedContextName.value, activeResourceKind.value);
  }
});

const handleCommandSelect = async (result: any) => {
  showCommandPalette.value = false;
  
  if (result.kind === 'Action') {
    // Navigate between tabs/views
    if (result.id === 'action_settings') handleTabChange('settings');
    else if (result.id === 'action_explorer') handleTabChange('explorer');
    else if (result.id === 'action_hardware') handleTabChange('hardware');
  } else if (result.kind === 'Cluster') {
    // Switch Kubernetes Context
    if (result.context) {
      selectedContextName.value = result.context;
      activeTab.value = 'explorer';
      currentView.value = 'cluster';
      sidebarVisible.value = true;
      await fetchNamespaces(result.context);
      fetchResources(result.context, activeResourceKind.value);
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

    const data = await invoke<any[]>(cmd, { contextName: context, namespace: selectedNamespace.value });
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

const handleOptimize = async () => {
  if (!currentAdvice.value) return;

  const action = currentAdvice.value.action;
  const namespace = selectedNamespace.value;
  const contextName = selectedContextName.value;

  try {
    if (action === 'Suspend') {
      await invoke('suspend_namespace', { contextName, namespace });
      currentAdvice.value = {
        action: 'Resume',
        reason: `Namespace '${namespace}' was suspended to save resources.`
      };
    } else if (action === 'Resume') {
      await invoke('resume_namespace', { contextName, namespace });
      currentAdvice.value = null;
    }
    
    // Refresh resources to update replicas on table
    if (contextName) {
      fetchResources(contextName, activeResourceKind.value);
    }
  } catch (e) {
    console.error('Optimization action failed:', e);
  }
};

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
      await fetchNamespaces(current.name);
      fetchResources(current.name, activeResourceKind.value);
    }
  } catch (e) {
    console.error('Initial fetch failed:', e);
  }
});
</script>

<template>
  <div class="ide-container" :style="{ gridTemplateColumns: gridColumns }">
    <!-- pane 1: Activity Bar -->
    <ActivityBar :active-id="activeTab" @update:active-id="handleTabChange" />
    
    <!-- pane 2: Cluster Hotbar (Persistent for cluster/marketplace) -->
    <ClusterHotbar 
      v-if="showHotbar"
      :contexts="availableContexts" 
      :active-name="selectedContextName" 
      @select="async (name) => { selectedContextName = name; await fetchNamespaces(name); fetchResources(name, activeResourceKind); }" 
    />
    
    <!-- pane 3: Sidebar (Independent Toggle) -->
    <Sidebar 
      v-if="currentView === 'cluster' || currentView === 'marketplace'"
      v-show="sidebarVisible"
      :title="activeTab" 
      :width="sidebarWidth" 
      @start-resize="handleStartResize"
    >
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

    <!-- pane 4: Main Workspace (Persistent across views) -->
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
          <select 
            v-if="currentView === 'cluster' || currentView === 'topology'" 
            v-model="selectedNamespace" 
            class="namespace-selector"
          >
            <option v-for="ns in availableNamespaces" :key="ns" :value="ns">{{ ns }}</option>
          </select>
          <div class="cluster-status" v-if="selectedContextName && currentView === 'cluster'">
            <span class="status-dot online"></span>
            {{ selectedContextName }}
          </div>
          <div class="time-display">{{ new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}</div>
        </div>
      </header>

      <div class="content-viewport">
        <!-- Views -->
        <WelcomeView v-if="currentView === 'welcome'" @start="handleTabChange('explorer')" />
        <SettingsView v-else-if="currentView === 'settings'" />
        <MarketplaceView v-else-if="currentView === 'marketplace'" />
        <TopologyView 
          v-else-if="currentView === 'topology'" 
          :context-name="selectedContextName"
          namespace="default"
        />
        
        <section v-else-if="currentView === 'cluster'" class="workloads-panel">
          <AdviceBanner :advice="currentAdvice" @optimize="handleOptimize" />
          
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

          <!-- Bottom Inspector Panel -->
          <div v-if="selectedResource" class="floating-inspector">
             <InspectorPanel 
               ref="inspectorPanelRef"
               :selected-resource="selectedResource" 
               @close="selectedResource = null"
               @edit="(res) => editingResource = res"
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

    <YamlEditorModal 
      v-if="editingResource" 
      :resource="editingResource" 
      :onClose="() => editingResource = null" 
    />
    
    <AdvisorToast :context-name="selectedContextName" :namespace="selectedNamespace" />
  </div>
</template>

<style>
/* Global Styles refined via Design Tokens */
body { 
  margin: 0; 
  padding: 0; 
  background-color: var(--surface-dark); 
  color: #f3f4f6; 
  font-family: var(--font-ui); 
  overflow: hidden; 
}

.ide-container {
  display: grid;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  transition: var(--transition-smooth);
}

/* Explicit Grid Assignments for Shell Stability */
.activity-bar { grid-column: 1; }
.cluster-hotbar { grid-column: 2; }
.sidebar { grid-column: 3; }
.main-area { 
  grid-column: 4;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background-color: var(--surface-dark);
}

/* Persistent Header Styling */
.main-header {
  height: 40px;
  background-color: var(--surface-glass);
  backdrop-filter: var(--glass-blur);
  border-bottom: var(--glass-border);
  display: flex;
  align-items: center;
  padding: 0 var(--space-4);
  justify-content: space-between;
  flex-shrink: 0;
  width: 100%;
  z-index: 10;
}

.header-left { display: flex; align-items: center; gap: var(--space-2); min-width: 150px; }
.brand-text { font-weight: 900; font-size: 0.85rem; color: var(--accent-blue); letter-spacing: 0.5px; }
.view-indicator { color: #6b7280; font-size: 0.7rem; text-transform: uppercase; font-weight: 600; }

.header-center { flex: 1; display: flex; justify-content: center; }
.search-box {
  width: 400px;
  max-width: 80%;
  height: 24px;
  background-color: var(--surface-dark);
  border: 1px solid #374151;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  padding: 0 var(--space-2);
  justify-content: space-between;
  cursor: pointer;
}
.search-box:hover { border-color: var(--accent-blue); }
.search-text { font-size: 0.7rem; color: #6b7280; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.search-shortcut { font-size: 0.6rem; color: #4b5563; font-weight: 700; border: 1px solid #374151; padding: 0 3px; border-radius: 2px; }

.header-right { display: flex; align-items: center; gap: var(--space-6); min-width: 150px; justify-content: flex-end; }
.namespace-selector {
  background-color: var(--surface-dark);
  border: 1px solid var(--border-dim);
  color: #d1d5db;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  outline: none;
}
.cluster-status { font-size: 0.7rem; color: #9ca3af; display: flex; align-items: center; gap: var(--space-1); font-weight: 600; }
.status-dot { width: 6px; height: 6px; background-color: #10b981; border-radius: 50%; }
.time-display { font-family: var(--font-code); font-size: 0.75rem; color: #6b7280; }

.content-viewport {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.workloads-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-scroll-area {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
}

.view-title-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-6);
}
.view-title-bar h2 { margin: 0; font-size: 1.5rem; font-weight: 800; }

.floating-inspector {
  height: 350px;
  border-top: 1px solid var(--border-dim);
  flex-shrink: 0;
}

.explorer-content, .telemetry-panel {
  padding: var(--space-4) var(--space-2);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.active-cluster-label {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background-color: rgba(59, 130, 246, 0.05);
  border: 1px solid rgba(59, 130, 246, 0.1);
  margin-bottom: var(--space-3);
  border-radius: var(--radius-md);
}
.label-text { font-size: 0.8rem; font-weight: 700; color: #9ca3af; }

.btn-refresh {
  background-color: #1f2937;
  border: 1px solid #374151;
  color: #d1d5db;
  padding: 6px var(--space-3);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
}

.metric-group {
  margin-bottom: var(--space-6);
  background-color: rgba(255, 255, 255, 0.02);
  padding: var(--space-4);
  border-radius: var(--radius-md);
}
.sub-metric { font-size: 0.7rem; color: #9ca3af; margin-top: var(--space-2); text-align: right; }
.gpu-not-found { font-size: 0.7rem; color: #6b7280; text-align: center; padding: var(--space-4); border: 1px dashed #374151; border-radius: var(--radius-sm); }

::-webkit-scrollbar { width: 8px; }
::-webkit-scrollbar-thumb { background: var(--border-dim); border-radius: var(--radius-sm); }
</style>
