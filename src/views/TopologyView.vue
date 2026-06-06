<script setup lang="ts">
import { ref, onMounted, markRaw } from 'vue';
import { VueFlow, useVueFlow, MarkerType } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { invoke } from '@tauri-apps/api/core';
import dagre from 'dagre';
import { Box, Server, Globe, Cpu } from 'lucide-vue-next';

const { onPaneReady, fitView } = useVueFlow();

const nodes = ref<any[]>([]);
const edges = ref<any[]>([]);

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

const fetchTopology = async () => {
  try {
    const graph = await invoke<any>('get_namespace_topology', { contextName: 'default', namespace: 'default' });
    
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
    
    // Fit view after layout
    setTimeout(() => {
      fitView();
    }, 50);
  } catch (e) {
    console.error('Failed to fetch topology:', e);
  }
};

onPaneReady(() => {
  fitView();
});

onMounted(fetchTopology);
</script>

<template>
  <div class="topology-wrapper">
    <VueFlow :nodes="nodes" :edges="edges" :node-types="nodeTypes" :fit-view-on-init="true">
      <Background color="#111827" :gap="20" pattern-color="#374151" />
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
}

/* Custom Node Styling */
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
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  transition: all 0.2s ease;
}

.custom-node-container:hover {
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.5);
}

.node-icon {
  color: #9ca3af;
}

.node-label {
  font-size: 0.75rem;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Kind-specific colors */
.pod .node-icon { color: #10b981; }
.service .node-icon { color: #3b82f6; }
.ingress .node-icon { color: #f59e0b; }

.vue-flow__edge-path {
  stroke-width: 2 !important;
  stroke: #4b5563 !important;
}

.vue-flow__edge.animated .vue-flow__edge-path {
  stroke: #3b82f6 !important;
}

.vue-flow__edge-label {
  background: #111827;
  color: #9ca3af;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  border: 1px solid #374151;
  pointer-events: none;
}

.vue-flow__handle {
  width: 6px !important;
  height: 6px !important;
  background: #3b82f6 !important;
}
</style>
