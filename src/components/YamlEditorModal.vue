<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  resource: any;
  onClose: () => void;
}>();

const yamlContent = ref('');
const isApplying = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);

onMounted(async () => {
  try {
    yamlContent.value = await invoke('get_k8s_resource_details', {
      contextName: props.resource.contextName,
      kind: props.resource.kind,
      namespace: props.resource.namespace,
      name: props.resource.name,
    });
  } catch (e: any) {
    error.value = e.toString();
  }
});

const applyChanges = async () => {
  isApplying.value = true;
  error.value = null;
  success.value = null;
  try {
    // 1. Dry run
    await invoke('apply_k8s_resource', {
      contextName: props.resource.contextName,
      kind: props.resource.kind,
      namespace: props.resource.namespace,
      name: props.resource.name,
      yaml: yamlContent.value,
      dryRun: true
    });
    
    // 2. Real apply
    const result = await invoke<string>('apply_k8s_resource', {
      contextName: props.resource.contextName,
      kind: props.resource.kind,
      namespace: props.resource.namespace,
      name: props.resource.name,
      yaml: yamlContent.value,
      dryRun: false
    });
    
    success.value = result;
    setTimeout(() => {
        props.onClose();
    }, 1500);
  } catch (e: any) {
    error.value = e.toString();
  } finally {
    isApplying.value = false;
  }
};
</script>

<template>
  <div class="modal-overlay">
    <div class="modal-content">
      <div class="modal-header">
        <h2>Edit {{ resource.kind }} / {{ resource.name }}</h2>
        <button @click="onClose" class="btn-close">Close</button>
      </div>
      <div v-if="error" class="error-banner">{{ error }}</div>
      <div v-if="success" class="success-banner">{{ success }}</div>
      <textarea v-model="yamlContent" class="yaml-editor" spellcheck="false"></textarea>
      <div class="modal-actions">
        <button @click="applyChanges" :disabled="isApplying" class="btn-apply">
          {{ isApplying ? 'Applying...' : 'Apply Changes' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-content { background: #111827; width: 80%; height: 80%; border-radius: 8px; display: flex; flex-direction: column; overflow: hidden; border: 1px solid #374151; }
.modal-header { padding: 1rem; border-bottom: 1px solid #1f2937; display: flex; justify-content: space-between; align-items: center; }
.modal-header h2 { margin: 0; font-size: 1.1rem; color: #f3f4f6; }
.yaml-editor { flex: 1; background: #030712; color: #60a5fa; font-family: monospace; padding: 1rem; border: none; outline: none; resize: none; }
.modal-actions { padding: 1rem; border-top: 1px solid #1f2937; display: flex; justify-content: flex-end; }
.error-banner { background: rgba(239, 68, 68, 0.1); color: #ef4444; padding: 0.5rem 1rem; font-size: 0.8rem; border-bottom: 1px solid rgba(239, 68, 68, 0.2); }
.success-banner { background: rgba(16, 185, 129, 0.1); color: #10b981; padding: 0.5rem 1rem; font-size: 0.8rem; border-bottom: 1px solid rgba(16, 185, 129, 0.2); }
.btn-apply { background: #3b82f6; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; font-weight: bold; }
.btn-apply:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-close { background: transparent; color: #9ca3af; border: 1px solid #374151; padding: 4px 12px; border-radius: 4px; cursor: pointer; }
.btn-close:hover { color: white; border-color: #9ca3af; }
</style>