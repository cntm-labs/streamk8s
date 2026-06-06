<script setup lang="ts">
import { ref, onMounted, markRaw, watch } from 'vue';
import { VueFlow, useVueFlow, MarkerType } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { invoke } from '@tauri-apps/api/core';
import dagre from 'dagre';
import { Box, Server, Globe, Cpu, RefreshCw, AlertTriangle, Layers } from 'lucide-vue-next';

const props = defineProps<{
  contextName: string | null;
}>();

const { onPaneReady, fitView } = useVueFlow();

const nodes = ref<any[]>([]);
const edges = ref<any[]>([]);
const namespaces = ref<string[]>([]);
const selectedNamespace = ref('default');
const isLoading = ref(false);
const error = ref<string | null>(null);

// Custom Node component mapping
const nodeTypes = {
  custom: markRaw({
    props: ['label', 'data'],
    setup(props: any) {
      const getIcon = (kind: string) => {
        switch (kind?.toLowerCase()) {
          case 'pod': return Box;
          case 'service': return Server;
          case 'ingress': return Globe;
          default: return Cpu;
        }
      };
      return { getIcon, props };
    },
    template: `
      <div class="custom-node-container" :class="props.data.kind?.toLowerCase()">
        <component :is="getIcon(props.data.kind)" :size="16" class="node-icon" />
        <span class="node-label">{{ props.label }}</span>
      </div>
    `
  })
};

const dagreGraph = new dagre.graphlib.Graph();
dagreGraph.setDefaultEdgeLabel(() => ({}));

const getLayoutedElements = (nodes: any[], edges: any[]) => {
  if (nodes.length === 0) return [];
  
  dagreGraph.setGraph({ rankdir: 'LR', nodesep: 60, ranksep: 150 });

  nodes.forEach((node) => {
    dagreGraph.setNode(node.id, { width: 180, height: 40 });
  });

  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);
  });

  dagre.layout(dagreGraph);

  return nodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    return {
      ...node,
      position: { x: nodeWithPosition.x - 90, y: nodeWithPosition.y - 20 },
    };
  });
};

const fetchNamespaces = async () => {
  if (!props.contextName) return;
  try {
    const list = await invoke<string[]>('get_namespaces', { contextName: props.contextName });
    namespaces.value = list;
    if (!list.includes(selectedNamespace.value) && list.length > 0) {
      selectedNamespace.value = list.includes('default') ? 'default' : list[0];
    }
  } catch (e) {
    console.error('Failed to fetch namespaces:', e);
  }
};

const fetchTopology = async () => {
  if (!props.contextName) return;
  
  isLoading.value = true;
  error.value = null;
  
  try {
    const graph = await invoke<any>('get_namespace_topology', { 
      contextName: props.contextName, 
      namespace: selectedNamespace.value 
    });
    
    if (!graph.nodes || graph.nodes.length === 0) {
      nodes.value = [];
      edges.value = [];
      return;
    }

    const initialNodes = graph.nodes.map((n: any) => ({
      id: n.id,
      label: n.name,
      type: 'custom',
      data: { kind: n.kind }
    }));

    const initialEdges = graph.edges.map((e: any) => ({
      id: e.id,
      source: e.source,
      target: e.target,
      label: e.label,
      animated: true,
      markerEnd: MarkerType.ArrowClosed,
      style: { stroke: '#3b82f6' }
    }));

    nodes.value = getLayoutedElements(initialNodes, initialEdges);
    edges.value = initialEdges;
    
    setTimeout(() => {
      fitView();
    }, 100);
  } catch (e: any) {
    console.error('Failed to fetch topology:', e);
    error.value = e.toString();
  } finally {
    isLoading.value = false;
  }
};

watch(() => props.contextName, () => {
  fetchNamespaces();
  fetchTopology();
});

watch(selectedNamespace, fetchTopology);

onPaneReady(() => {
  fitView();
});

onMounted(() => {
  fetchNamespaces();
  fetchTopology();
});
</script>

<template>
  <div class="topology-wrapper">
    <!-- Overlay Actions -->
    <div class="topology-controls">
      <div class="status-indicator">
        <div class="dot" :class="{ active: props.contextName }"></div>
        <span>{{ props.contextName || 'No Context' }}</span>
      </div>
      
      <div class="ns-selector">
        <Layers :size="14" />
        <select v-model="selectedNamespace">
          <option v-for="ns in namespaces" :key="ns" :value="ns">{{ ns }}</option>
        </select>
      </div>

      <button class="refresh-btn" @click="fetchTopology" :disabled="isLoading">
        <RefreshCw :size="14" :class="{ spin: isLoading }" />
        Refresh
      </button>
    </div>

    <!-- Empty/Error States -->
    <div v-if="error" class="overlay-state error">
      <AlertTriangle :size="32" />
      <p>{{ error }}</p>
      <button @click="fetchTopology">Try Again</button>
    </div>
    
    <div v-else-if="nodes.length === 0 && !isLoading" class="overlay-state empty">
      <Box :size="48" style="opacity: 0.2" />
      <h3>No Resources Found</h3>
      <p>There are no Ingresses, Services, or Pods in the <b>{{ selectedNamespace }}</b> namespace.</p>
    </div>

    <VueFlow :nodes="nodes" :edges="edges" :node-types="nodeTypes" :fit-view-on-init="true">
      <Background color="#030712" :gap="20" pattern-color="#1f2937" />
    </VueFlow>
  </div>
</template>

<style>
@import '@vue-flow/core/dist/style.css';
@import '@vue-flow/core/dist/theme-default.css';

.topology-wrapper {
  width: 100%;
  height: 100%;
  background-color: #030712;
  position: relative;
}

.topology-controls {
  position: absolute;
  top: 1rem;
  left: 1rem;
  z-index: 10;
  display: flex;
  gap: 1rem;
  align-items: center;
  background: rgba(17, 24, 39, 0.8);
  backdrop-filter: blur(8px);
  padding: 8px 16px;
  border-radius: 8px;
  border: 1px solid #1f2937;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
  font-weight: 700;
  color: #9ca3af;
}

.ns-selector {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #0f172a;
  border: 1px solid #374151;
  padding: 2px 8px;
  border-radius: 4px;
  color: #9ca3af;
}

.ns-selector select {
  background: transparent;
  border: none;
  color: #f3f4f6;
  font-size: 0.75rem;
  font-weight: 600;
  outline: none;
  cursor: pointer;
}

.dot {
  width: 8px; height: 8px;
  background: #4b5563;
  border-radius: 50%;
}
.dot.active {
  background: #10b981;
  box-shadow: 0 0 8px #10b981;
}

.refresh-btn {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}
.refresh-btn:hover { background: #2563eb; }

.overlay-state {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 5;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.overlay-state.error { color: #ef4444; }
.overlay-state.empty { color: #4b5563; }

.custom-node-container {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: #1f2937;
  color: #f3f4f6;
  border: 1px solid #4b5563;
  border-radius: 6px;
  min-width: 180px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}

.pod .node-icon { color: #10b981; }
.service .node-icon { color: #3b82f6; }
.ingress .node-icon { color: #f59e0b; }

.vue-flow__edge-path {
  stroke-width: 2 !important;
  stroke: #4b5563 !important;
}

.vue-flow__edge-label {
  background: #111827;
  color: #9ca3af;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  border: 1px solid #374151;
}

.spin { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
