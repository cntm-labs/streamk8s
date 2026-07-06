<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  contextName: string;
  namespace: string;
  podName: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'success'): void;
}>();

const localPort = ref(8080);
const remotePort = ref(80);
const isForwarding = ref(false);
const error = ref<string | null>(null);

const startForward = async () => {
  if (!localPort.value || !remotePort.value) return;
  isForwarding.value = true;
  error.value = null;

  try {
    await invoke('start_port_forward', {
      context: props.contextName,
      namespace: props.namespace,
      pod: props.podName,
      localPort: Number(localPort.value),
      remotePort: Number(remotePort.value)
    });
    emit('success');
    emit('close');
  } catch (e) {
    error.value = String(e);
  } finally {
    isForwarding.value = false;
  }
};
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content">
      <header class="modal-header">
        <h3>Port Forward: {{ podName }}</h3>
        <button class="btn-close" @click="$emit('close')">✕</button>
      </header>
      
      <div class="modal-body">
        <div v-if="error" class="error-msg">{{ error }}</div>
        
        <div class="form-group">
          <label>Local Port</label>
          <input type="number" v-model="localPort" placeholder="e.g. 8080" />
        </div>
        
        <div class="form-group">
          <label>Remote Port (Pod)</label>
          <input type="number" v-model="remotePort" placeholder="e.g. 80" />
        </div>
      </div>
      
      <footer class="modal-footer">
        <button class="btn-cancel" @click="$emit('close')">Cancel</button>
        <button class="btn-primary" :disabled="isForwarding" @click="startForward">
          {{ isForwarding ? 'Starting...' : 'Start Forwarding' }}
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background-color: var(--surface-dark);
  border: 1px solid var(--border-dim);
  border-radius: var(--radius-md);
  width: 400px;
  max-width: 90vw;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-dim);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h3 {
  margin: 0;
  font-size: 1.1rem;
}

.btn-close {
  background: transparent;
  border: none;
  color: #9ca3af;
  cursor: pointer;
  font-size: 1.2rem;
}
.btn-close:hover {
  color: #f3f4f6;
}

.modal-body {
  padding: var(--space-4);
}

.form-group {
  margin-bottom: var(--space-4);
}

.form-group label {
  display: block;
  margin-bottom: 4px;
  font-size: 0.85rem;
  color: #d1d5db;
}

.form-group input {
  width: 100%;
  box-sizing: border-box;
  padding: 8px;
  background-color: #1f2937;
  border: 1px solid #374151;
  color: white;
  border-radius: var(--radius-sm);
}

.form-group input:focus {
  border-color: #3b82f6;
  outline: none;
}

.error-msg {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  padding: 8px;
  border-radius: var(--radius-sm);
  margin-bottom: var(--space-4);
  font-size: 0.85rem;
}

.modal-footer {
  padding: var(--space-4);
  border-top: 1px solid var(--border-dim);
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
}

.btn-cancel {
  background-color: transparent;
  color: #d1d5db;
  border: 1px solid #374151;
  padding: 8px 16px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}
.btn-cancel:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: 600;
}
.btn-primary:hover:not(:disabled) {
  background-color: #2563eb;
}
.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
