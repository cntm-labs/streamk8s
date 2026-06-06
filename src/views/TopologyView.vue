<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { VueFlow, useVueFlow } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { invoke } from '@tauri-apps/api/core';

const { onPaneReady, fitView } = useVueFlow();

const nodes = ref<any[]>([]);
const edges = ref<any[]>([]);

const fetchTopology = async () => {
  try {
    const graph = await invoke<any>('get_namespace_topology', { contextName: 'default', namespace: 'default' });
    
    nodes.value = graph.nodes.map((n: any, idx: number) => ({
      id: n.id,
      label: n.name,
      type: 'default',
      position: { x: idx * 250, y: 100 }, // Temporary horizontal layout before dagre
      data: { kind: n.kind }
    }));

    edges.value = graph.edges.map((e: any) => ({
      id: e.id,
      source: e.source,
      target: e.target,
      animated: true,
      style: { stroke: '#3b82f6' }
    }));
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
    <VueFlow :nodes="nodes" :edges="edges">
      <Background color="#1f2937" :gap="20" />
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

/* Customize Vue Flow Theme */
.vue-flow__node-default {
  background: #1f2937 !important;
  color: #f3f4f6 !important;
  border: 1px solid #374151 !important;
  border-radius: 8px !important;
  font-size: 0.8rem !important;
  font-weight: 600 !important;
}
.vue-flow__edge-path {
  stroke-width: 2 !important;
}
</style>
