<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Play, Terminal, Info, Cpu, Hash } from 'lucide-vue-next';

interface UiComponent {
  id: string;
  type: string;
  label: string;
  placeholder?: string;
  action?: string;
}

const props = defineProps<{ 
  manifest: { 
    extension: { name: string, description: string, version: string, id: string },
    ui: { components: UiComponent[] } 
  }, 
  pluginId: string 
}>();

const formData = ref<Record<string, string>>({});
const output = ref<string[]>([]);
const isRunning = ref(false);
const terminalRef = ref<HTMLElement | null>(null);

const scrollToBottom = async () => {
  await nextTick();
  if (terminalRef.value) {
    terminalRef.value.scrollTop = terminalRef.value.scrollHeight;
  }
};

watch(output, scrollToBottom, { deep: true });

const callAction = async (action: string) => {
  isRunning.value = true;
  output.value.push(`> Executing action: ${action}`);
  try {
    const result = await invoke<string>('call_plugin_action', { 
       pluginId: props.pluginId, 
       actionName: action, 
       payload: JSON.stringify(formData.value) 
    });
    output.value.push(result);
  } catch (e) {
    output.value.push(`[ERROR] ${e}`);
  } finally {
    isRunning.value = false;
  }
};

const clearTerminal = () => {
  output.value = [];
};
</script>

<template>
  <div class="plugin-container">
    <header class="plugin-header">
      <div class="header-main">
        <div class="title-group">
          <h2>{{ manifest.extension.name }}</h2>
          <div class="badge-row">
            <span class="badge id-badge">
              <Hash :size="10" class="mr-1" />
              {{ manifest.extension.id }}
            </span>
            <span class="badge version-badge">
              v{{ manifest.extension.version }}
            </span>
          </div>
        </div>
        <p class="description">{{ manifest.extension.description }}</p>
      </div>
    </header>

    <div class="plugin-layout">
      <div class="plugin-controls">
        <div class="section-header">
          <Cpu :size="14" class="mr-2" />
          CONTROLS
        </div>
        <div class="controls-content">
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
      </div>

      <div class="plugin-terminal">
        <div class="terminal-header">
          <div class="header-left">
            <Terminal :size="12" class="mr-2" />
            TERMINAL OUTPUT
          </div>
          <button class="clear-btn" @click="clearTerminal">Clear</button>
        </div>
        <div class="terminal-content" ref="terminalRef">
          <div v-if="output.length === 0" class="terminal-empty">
            <Info :size="16" class="mb-2 opacity-50" />
            <p>Ready to execute WASM actions.</p>
          </div>
          <div v-for="(line, i) in output" :key="i" class="term-line">
            <span class="line-time">[{{ new Date().toLocaleTimeString() }}]</span>
            <span class="line-content" :class="{ 'error': line.startsWith('[ERROR]') }">{{ line }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.plugin-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #030712;
  color: #e5e7eb;
}

.plugin-header {
  padding: 0 0 1.5rem 0;
  border-bottom: 1px solid #1f2937;
  margin-bottom: 1.5rem;
}

.title-group {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
}

.plugin-header h2 { 
  margin: 0; 
  color: #3b82f6; 
  font-size: 1.25rem; 
  font-weight: 700;
}

.badge-row {
  display: flex;
  gap: 0.5rem;
}

.badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 9999px;
  font-size: 0.65rem;
  font-weight: 600;
  letter-spacing: 0.025em;
}

.id-badge {
  background-color: #1e293b;
  color: #94a3b8;
  border: 1px solid #334155;
}

.version-badge {
  background-color: rgba(59, 130, 246, 0.1);
  color: #60a5fa;
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.description { 
  color: #9ca3af; 
  margin: 0; 
  font-size: 0.85rem; 
  line-height: 1.5;
}

.plugin-layout {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: 1.5rem;
  flex: 1;
  min-height: 0;
}

.section-header {
  font-size: 0.7rem;
  font-weight: 700;
  color: #4b5563;
  letter-spacing: 0.1em;
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
}

.plugin-controls {
  display: flex;
  flex-direction: column;
  background-color: #0f172a;
  border: 1px solid #1f2937;
  border-radius: 8px;
  padding: 1.25rem;
  overflow-y: auto;
}

.controls-content {
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
  font-size: 0.75rem;
  font-weight: 600;
  color: #94a3b8;
}

.plugin-input {
  background-color: #020617;
  border: 1px solid #334155;
  color: #f8fafc;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.85rem;
  outline: none;
  transition: border-color 0.2s;
}

.plugin-input:focus {
  border-color: #3b82f6;
}

.plugin-btn {
  background: #2563eb;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.plugin-btn:hover:not(:disabled) { background-color: #3b82f6; }
.plugin-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.plugin-terminal {
  background-color: #020617;
  border: 1px solid #1f2937;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.terminal-header {
  background-color: #0f172a;
  padding: 6px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #1f2937;
}

.header-left {
  font-size: 0.65rem;
  font-weight: 700;
  color: #64748b;
  display: flex;
  align-items: center;
}

.clear-btn {
  background: none;
  border: none;
  color: #475569;
  font-size: 0.65rem;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 3px;
}

.clear-btn:hover {
  color: #94a3b8;
  background-color: rgba(255, 255, 255, 0.05);
}

.terminal-content {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
  font-size: 0.8rem;
  color: #10b981;
}

.terminal-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #334155;
  font-size: 0.75rem;
}

.term-line {
  margin-bottom: 0.25rem;
  line-height: 1.4;
  word-break: break-all;
}

.line-time {
  color: #475569;
  margin-right: 0.75rem;
  user-select: none;
}

.line-content.error {
  color: #ef4444;
}

.mr-1 { margin-right: 0.25rem; }
.mr-2 { margin-right: 0.5rem; }
.mb-2 { margin-bottom: 0.5rem; }
.opacity-50 { opacity: 0.5; }
</style>
