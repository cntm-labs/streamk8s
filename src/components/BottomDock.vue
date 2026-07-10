<script setup lang="ts">
import { ref } from 'vue';
import LogViewer from './LogViewer.vue';
import PodTerminal from './PodTerminal.vue';

defineProps<{
  isOpen: boolean;
  mode: 'logs' | 'terminal';
  contextName: string | null;
  namespace: string;
  podName: string;
  containerName: string | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const dockHeight = ref(300);
const isResizing = ref(false);

const startResize = () => {
  isResizing.value = true;
  document.addEventListener('mousemove', resize);
  document.addEventListener('mouseup', stopResize);
  document.body.style.cursor = 'row-resize';
};

const resize = (e: MouseEvent) => {
  if (!isResizing.value) return;
  const newHeight = window.innerHeight - e.clientY;
  if (newHeight > 100 && newHeight < window.innerHeight * 0.8) {
    dockHeight.value = newHeight;
  }
};

const stopResize = () => {
  isResizing.value = false;
  document.removeEventListener('mousemove', resize);
  document.removeEventListener('mouseup', stopResize);
  document.body.style.cursor = 'default';
};
</script>

<template>
  <div v-if="isOpen" class="bottom-dock" :style="{ height: dockHeight + 'px' }">
    <div class="dock-resizer" @mousedown="startResize"></div>
    <div class="dock-header">
      <div class="dock-tabs">
        <span class="dock-tab active">
          {{ mode === 'logs' ? 'Logs' : 'Terminal' }}: {{ podName }}
        </span>
      </div>
      <div class="dock-actions">
        <button class="close-btn" @click="emit('close')">✖</button>
      </div>
    </div>
    <div class="dock-content">
      <LogViewer 
        v-if="mode === 'logs'" 
        :context-name="contextName"
        :namespace="namespace"
        :pod-name="podName"
        :container-name="containerName"
      />
      <PodTerminal 
        v-else-if="mode === 'terminal'"
        :context-name="contextName"
        :namespace="namespace"
        :pod-name="podName"
        :container-name="containerName || ''"
      />
    </div>
  </div>
</template>

<style scoped>
.bottom-dock {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background-color: var(--surface-dark, #111827);
  border-top: 1px solid var(--border-dim, #374151);
  display: flex;
  flex-direction: column;
  z-index: 50;
  box-shadow: 0 -4px 12px rgba(0,0,0,0.3);
}
.dock-resizer {
  height: 4px;
  cursor: row-resize;
  background-color: transparent;
  position: absolute;
  top: -2px;
  left: 0;
  right: 0;
  z-index: 10;
}
.dock-resizer:hover {
  background-color: var(--accent-blue, #3b82f6);
}
.dock-header {
  height: 36px;
  background-color: var(--surface-card, #1f2937);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid var(--border-dim, #374151);
}
.dock-tabs {
  display: flex;
  gap: 8px;
}
.dock-tab {
  font-size: 0.8rem;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: 4px;
  background-color: var(--surface-dark, #111827);
  color: #d1d5db;
  border: 1px solid var(--border-dim, #374151);
}
.dock-tab.active {
  color: var(--accent-blue, #3b82f6);
  border-color: var(--accent-blue, #3b82f6);
}
.dock-actions {
  display: flex;
  align-items: center;
}
.close-btn {
  background: none;
  border: none;
  color: #9ca3af;
  cursor: pointer;
  font-size: 1rem;
  padding: 4px;
}
.close-btn:hover {
  color: #f3f4f6;
}
.dock-content {
  flex: 1;
  overflow: hidden;
  position: relative;
}
</style>
