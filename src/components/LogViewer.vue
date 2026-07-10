<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

const props = defineProps<{
  contextName: string | null;
  namespace: string;
  podName: string;
  containerName: string | null;
}>();

const logs = ref<string[]>([]);
const logContainer = ref<HTMLElement | null>(null);
let unlisten: UnlistenFn | null = null;

const scrollToBottom = () => {
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight;
  }
};

onMounted(async () => {
  try {
    unlisten = await listen<string>('pod-log-line', (event) => {
      logs.value.push(event.payload);
      if (logs.value.length > 2000) {
        logs.value.shift();
      }
      nextTick(scrollToBottom);
    });

    await invoke('start_log_stream', {
      contextName: props.contextName,
      namespace: props.namespace,
      podName: props.podName,
      containerName: props.containerName
    });
  } catch (e) {
    console.error("Failed to start log stream", e);
    logs.value.push(`Error: ${e}`);
  }
});

onUnmounted(() => {
  if (unlisten) unlisten();
});
</script>

<template>
  <div class="log-viewer" ref="logContainer">
    <div v-if="logs.length === 0" class="log-line empty">Loading logs...</div>
    <div v-for="(line, idx) in logs" :key="idx" class="log-line">{{ line }}</div>
  </div>
</template>

<style scoped>
.log-viewer {
  background-color: #0f172a;
  color: #e2e8f0;
  font-family: var(--font-code, monospace);
  font-size: 0.8rem;
  padding: 12px;
  height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
}
.log-line {
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.4;
}
.log-line.empty {
  color: #94a3b8;
  font-style: italic;
}
</style>
