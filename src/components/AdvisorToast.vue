<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{ contextName: string | null }>();

const visible = ref(false);
const detectedApp = ref('');
const isSuspended = ref(false);

onMounted(async () => {
  await listen<string>('heavy-app-detected', (event) => {
    if (!isSuspended.value && !visible.value) {
      detectedApp.value = event.payload;
      visible.value = true;
    }
  });
});

const suspend = async () => {
  try {
    await invoke('suspend_namespace', { contextName: props.contextName, namespace: 'default' });
    isSuspended.value = true;
    visible.value = false;
  } catch(e) { console.error(e); }
};

const resume = async () => {
  try {
    await invoke('resume_namespace', { contextName: props.contextName, namespace: 'default' });
    isSuspended.value = false;
  } catch(e) { console.error(e); }
};
</script>

<template>
  <div class="advisor-container">
    <div v-if="visible" class="toast-dialog">
      <div class="toast-title">🎮 Heavy App Detected</div>
      <div class="toast-body">'{{ detectedApp }}' is using high resources. Suspend 'default' namespace?</div>
      <div class="toast-buttons">
        <button class="btn-primary" @click="suspend">Suspend K8s</button>
        <button class="btn-secondary" @click="visible = false">Ignore</button>
      </div>
    </div>
    <div v-if="isSuspended" class="resume-banner">
      K8s Suspended to save resources. <button @click="resume" class="btn-resume">Resume Now</button>
    </div>
  </div>
</template>

<style scoped>
.advisor-container { position: fixed; bottom: 20px; right: 20px; z-index: 9999; display: flex; flex-direction: column; gap: 10px; }
.toast-dialog { background: #1f2937; border: 1px solid #3b82f6; border-radius: 8px; padding: 1rem; box-shadow: 0 10px 25px rgba(0,0,0,0.5); width: 300px; }
.toast-title { font-weight: bold; font-size: 0.9rem; margin-bottom: 0.5rem; color: #f3f4f6; }
.toast-body { font-size: 0.8rem; color: #d1d5db; margin-bottom: 1rem; }
.toast-buttons { display: flex; gap: 8px; }
.btn-primary { background: #3b82f6; color: white; border: none; padding: 6px 12px; border-radius: 4px; font-size: 0.75rem; cursor: pointer; flex: 1; font-weight: bold; }
.btn-secondary { background: transparent; color: #9ca3af; border: 1px solid #4b5563; padding: 6px 12px; border-radius: 4px; font-size: 0.75rem; cursor: pointer; flex: 1; }
.resume-banner { background: #064e3b; border: 1px solid #059669; color: #34d399; padding: 10px 16px; border-radius: 8px; font-size: 0.8rem; font-weight: 600; display: flex; align-items: center; justify-content: space-between; gap: 1rem; box-shadow: 0 10px 20px rgba(0,0,0,0.5); }
.btn-resume { background: #10b981; color: #064e3b; border: none; font-weight: bold; padding: 4px 10px; border-radius: 4px; cursor: pointer; }
</style>