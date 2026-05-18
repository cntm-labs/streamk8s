<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Play, Terminal } from 'lucide-vue-next';

interface UiComponent {
  id: string;
  type: string;
  label: string;
  placeholder?: string;
  action?: string;
}

const props = defineProps<{ 
  manifest: { 
    extension: { name: string, description: string },
    ui: { components: UiComponent[] } 
  }, 
  pluginId: string 
}>();

const formData = ref<Record<string, string>>({});
const output = ref<string[]>([]);
const isRunning = ref(false);

const callAction = async (action: string) => {
  isRunning.value = true;
  output.value.push(`> Executing ${action}...`);
  try {
    const result = await invoke<string>('call_plugin_action', { 
       pluginId: props.pluginId, 
       actionName: action, 
       payload: JSON.stringify(formData.value) 
    });
    output.value.push(result);
  } catch (e) {
    output.value.push(`Error: ${e}`);
  } finally {
    isRunning.value = false;
  }
};
</script>

<template>
  <div class="plugin-container">
    <header class="plugin-header">
      <h2>{{ manifest.extension.name }}</h2>
      <p>{{ manifest.extension.description }}</p>
    </header>

    <div class="plugin-body">
      <div v-for="comp in manifest.ui.components" :key="comp.id" class="comp-item">
        <label v-if="comp.label">{{ comp.label }}</label>
        
        <input 
          v-if="comp.type === 'input'" 
          v-model="formData[comp.id]" 
          :placeholder="comp.placeholder"
          class="plugin-input"
        />

        <button 
          v-if="comp.type === 'button'" 
          @click="callAction(comp.action || '')"
          class="plugin-btn"
          :disabled="isRunning"
        >
          <Play :size="14" class="mr-2" />
          {{ comp.label }}
        </button>
      </div>
    </div>

    <div class="plugin-terminal">
      <div class="terminal-header">
        <Terminal :size="12" class="mr-2" />
        OUTPUT
      </div>
      <div class="terminal-content">
        <div v-for="(line, i) in output" :key="i" class="term-line">{{ line }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.plugin-container {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  padding: 1.5rem;
  background-color: #0a0a0a;
  border-radius: 12px;
  border: 1px solid #1f2937;
  height: 100%;
}

.plugin-header h2 { margin: 0; color: #3b82f6; font-size: 1.5rem; }
.plugin-header p { color: #9ca3af; margin: 0.25rem 0 0 0; font-size: 0.9rem; }

.plugin-body {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.comp-item {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.comp-item label {
  font-size: 0.8rem;
  font-weight: 700;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.plugin-input {
  background-color: #111827;
  border: 1px solid #374151;
  color: white;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 0.9rem;
  outline: none;
}

.plugin-btn {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
  border: none;
  padding: 10px 16px;
  border-radius: 6px;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.plugin-btn:hover { opacity: 0.9; transform: translateY(-1px); }

.plugin-terminal {
  flex: 1;
  background-color: #000;
  border: 1px solid #1f2937;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: 'JetBrains Mono', monospace;
}

.terminal-header {
  background-color: #111827;
  padding: 4px 10px;
  font-size: 0.65rem;
  font-weight: 800;
  color: #4b5563;
  display: flex;
  align-items: center;
}

.terminal-content {
  padding: 10px;
  font-size: 0.8rem;
  color: #34d399;
  overflow-y: auto;
}

.mr-2 { margin-right: 0.5rem; }
</style>
