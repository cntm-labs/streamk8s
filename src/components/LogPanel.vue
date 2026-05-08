<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

const logs = ref<string[]>([]);
const logContainer = ref<HTMLElement | null>(null);
let unlisten: UnlistenFn;

onMounted(async () => {
  unlisten = await listen<string>('pod-log-line', (event) => {
    logs.value.push(event.payload);
    if (logs.value.length > 500) logs.value.shift();
    
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight;
      }
    });
  });
});

onUnmounted(() => {
  if (unlisten) unlisten();
});

const clearLogs = () => {
  logs.value = [];
};

defineExpose({ clearLogs });
</script>

<template>
  <div class="log-panel">
    <div class="panel-header">
      <div class="title">TERMINAL LOGS</div>
      <button @click="clearLogs" class="clear-btn">Clear</button>
    </div>
    <div ref="logContainer" class="log-content">
      <div v-for="(line, i) in logs" :key="i" class="log-line">
        <span class="line-num">{{ i + 1 }}</span>
        <span class="line-text">{{ line }}</span>
      </div>
      <div v-if="logs.length === 0" class="empty-state">
        Select a pod to start streaming logs...
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-panel {
  height: 250px;
  background-color: #000;
  border-top: 1px solid #374151;
  display: flex;
  flex-direction: column;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}
.panel-header {
  height: 32px;
  background-color: #1f2937;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 1rem;
  border-bottom: 1px solid #374151;
}
.title {
  font-size: 0.7rem;
  font-weight: 700;
  color: #9ca3af;
  letter-spacing: 1px;
}
.clear-btn {
  background: none;
  border: 1px solid #4b5563;
  color: #9ca3af;
  font-size: 0.65rem;
  padding: 2px 8px;
  border-radius: 3px;
  cursor: pointer;
}
.clear-btn:hover {
  background-color: #374151;
  color: white;
}
.log-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  background-color: #0a0a0a;
}
.log-line {
  display: flex;
  gap: 0.75rem;
  font-size: 0.8rem;
  line-height: 1.4;
  white-space: pre-wrap;
  word-break: break-all;
}
.line-num {
  color: #4b5563;
  min-width: 2rem;
  text-align: right;
  user-select: none;
}
.line-text {
  color: #d1d5db;
}
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: #4b5563;
  font-size: 0.8rem;
  font-style: italic;
}
</style>
