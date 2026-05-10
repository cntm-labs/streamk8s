<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  selectedPod: { contextName: string, namespace: string, name: string } | null
}>();

const activeTab = ref('Logs');
const tabs = ['Logs', 'YAML', 'Events', 'Files'];

// Logs State
const logs = ref<string[]>([]);
const logContainer = ref<HTMLElement | null>(null);
let unlisten: UnlistenFn;

// YAML State
const yamlContent = ref('');
const isLoadingYaml = ref(false);

// Events State
const events = ref<any[]>([]);
const isLoadingEvents = ref(false);

// Files State
const fileContent = ref('');
const filePath = ref('/etc/hosts'); // Default example
const isLoadingFiles = ref(false);

const clearLogs = () => {
  logs.value = [];
};

const fetchYaml = async () => {
  if (!props.selectedPod) return;
  isLoadingYaml.value = true;
  try {
    yamlContent.value = await invoke('get_resource_manifest', {
      contextName: props.selectedPod.contextName,
      namespace: props.selectedPod.namespace,
      podName: props.selectedPod.name
    });
  } catch (e) {
    yamlContent.value = `Error fetching YAML: ${e}`;
  } finally {
    isLoadingYaml.value = false;
  }
};

const fetchEvents = async () => {
  if (!props.selectedPod) return;
  isLoadingEvents.value = true;
  try {
    const response: any = await invoke('get_pod_events', {
      contextName: props.selectedPod.contextName,
      namespace: props.selectedPod.namespace,
      podName: props.selectedPod.name
    });
    events.value = response.items || [];
  } catch (e) {
    console.error('Failed to fetch events:', e);
  } finally {
    isLoadingEvents.value = false;
  }
};

const readFile = async () => {
  if (!props.selectedPod) return;
  isLoadingFiles.value = true;
  try {
    // Note: container_name is required. For simplicity, we might need to fetch container names first.
    // For now, let's assume the first container name matches the pod name prefix or just try to get it.
    // In a real app, we'd have a container selector.
    fileContent.value = await invoke('read_pod_file', {
      contextName: props.selectedPod.contextName,
      namespace: props.selectedPod.namespace,
      podName: props.selectedPod.name,
      containerName: props.selectedPod.name, // Temporary assumption
      filePath: filePath.value
    });
  } catch (e) {
    fileContent.value = `Error reading file: ${e}\n(Note: Container name might be different from pod name)`;
  } finally {
    isLoadingFiles.value = false;
  }
};

const applyYaml = async () => {
  if (!props.selectedPod) return;
  isLoadingYaml.value = true;
  try {
    await invoke('apply_resource_manifest', {
      contextName: props.selectedPod.contextName,
      namespace: props.selectedPod.namespace,
      podName: props.selectedPod.name,
      yamlContent: yamlContent.value
    });
    alert('Manifest applied successfully!');
  } catch (e) {
    alert(`Failed to apply manifest: ${e}`);
  } finally {
    isLoadingYaml.value = false;
  }
};

const saveFile = async () => {
  if (!props.selectedPod) return;
  isLoadingFiles.value = true;
  try {
    // UTF-8 to Base64
    const contentBase64 = window.btoa(unescape(encodeURIComponent(fileContent.value)));
    await invoke('write_pod_file', {
      contextName: props.selectedPod.contextName,
      namespace: props.selectedPod.namespace,
      podName: props.selectedPod.name,
      containerName: props.selectedPod.name, // Assumption matches readFile
      filePath: filePath.value,
      contentBase64
    });
    alert('File saved successfully!');
  } catch (e) {
    alert(`Failed to save file: ${e}`);
  } finally {
    isLoadingFiles.value = false;
  }
};

onMounted(async () => {
  unlisten = await listen<string>('pod-log-line', (event) => {
    logs.value.push(event.payload);
    if (logs.value.length > 500) logs.value.shift();
    
    if (activeTab.value === 'Logs') {
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = logContainer.value.scrollHeight;
        }
      });
    }
  });
});

onUnmounted(() => {
  if (unlisten) unlisten();
});

watch(activeTab, (newTab) => {
  if (newTab === 'YAML') fetchYaml();
  if (newTab === 'Events') fetchEvents();
  if (newTab === 'Files') readFile();
});

watch(() => props.selectedPod, () => {
  clearLogs();
  if (activeTab.value === 'YAML') fetchYaml();
  if (activeTab.value === 'Events') fetchEvents();
  if (activeTab.value === 'Files') readFile();
}, { deep: true });

defineExpose({ clearLogs });
</script>

<template>
  <div class="inspector-panel">
    <div class="panel-header">
      <div class="tabs">
        <button 
          v-for="t in tabs" 
          :key="t" 
          @click="activeTab = t" 
          :class="['tab-btn', { active: activeTab === t }]"
        >
          {{ t }}
        </button>
      </div>
      <div class="actions">
        <button v-if="activeTab === 'Logs'" @click="clearLogs" class="action-btn">Clear</button>
        <button v-if="activeTab === 'YAML'" @click="fetchYaml" class="action-btn" :disabled="isLoadingYaml">Refresh</button>
        <button v-if="activeTab === 'YAML'" @click="applyYaml" class="action-btn save-btn" :disabled="isLoadingYaml">Apply</button>
        <button v-if="activeTab === 'Events'" @click="fetchEvents" class="action-btn" :disabled="isLoadingEvents">Refresh</button>
        <button v-if="activeTab === 'Files'" @click="saveFile" class="action-btn save-btn" :disabled="isLoadingFiles">Save</button>
      </div>
    </div>

    <div class="tab-content">
      <!-- LOGS TAB -->
      <div v-if="activeTab === 'Logs'" ref="logContainer" class="log-content">
        <div v-for="(line, i) in logs" :key="i" class="log-line">
          <span class="line-num">{{ i + 1 }}</span>
          <span class="line-text">{{ line }}</span>
        </div>
        <div v-if="logs.length === 0" class="empty-state">
          {{ selectedPod ? 'Streaming logs...' : 'Select a pod to start streaming logs...' }}
        </div>
      </div>

      <!-- YAML TAB -->
      <div v-if="activeTab === 'YAML'" class="yaml-content">
        <div v-if="isLoadingYaml" class="loading-overlay">Loading manifest...</div>
        <textarea v-model="yamlContent" class="code-editor"></textarea>
      </div>

      <!-- EVENTS TAB -->
      <div v-if="activeTab === 'Events'" class="events-content">
        <div v-if="isLoadingEvents" class="loading-overlay">Loading events...</div>
        <table v-else class="events-table">
          <thead>
            <tr>
              <th>Type</th>
              <th>Reason</th>
              <th>Message</th>
              <th>Last Seen</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="event in events" :key="event.metadata.uid">
              <td :class="['type-cell', event.type.toLowerCase()]">{{ event.type }}</td>
              <td>{{ event.reason }}</td>
              <td>{{ event.message }}</td>
              <td>{{ new Date(event.lastTimestamp || event.eventTime).toLocaleString() }}</td>
            </tr>
            <tr v-if="events.length === 0">
              <td colspan="4" class="empty-state">No events found for this pod.</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- FILES TAB -->
      <div v-if="activeTab === 'Files'" class="files-content">
        <div class="file-browser-header">
          <input v-model="filePath" class="path-input" @keyup.enter="readFile" />
          <button @click="readFile" class="action-btn" :disabled="isLoadingFiles">Read</button>
        </div>
        <div class="file-editor-area">
          <div v-if="isLoadingFiles" class="loading-overlay">Reading file...</div>
          <textarea v-model="fileContent" class="code-editor" placeholder="File content will appear here..."></textarea>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inspector-panel {
  height: 300px;
  background-color: #000;
  border-top: 1px solid #374151;
  display: flex;
  flex-direction: column;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}
.panel-header {
  height: 36px;
  background-color: #1f2937;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 0.5rem;
  border-bottom: 1px solid #374151;
}
.tabs {
  display: flex;
  gap: 2px;
  height: 100%;
}
.tab-btn {
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: #9ca3af;
  font-size: 0.7rem;
  font-weight: 600;
  padding: 0 1rem;
  cursor: pointer;
  transition: all 0.2s;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.tab-btn:hover {
  color: #d1d5db;
  background-color: rgba(255,255,255,0.05);
}
.tab-btn.active {
  color: #3b82f6;
  border-bottom-color: #3b82f6;
  background-color: rgba(59, 130, 246, 0.1);
}
.actions {
  display: flex;
  gap: 0.5rem;
}
.action-btn {
  background-color: #374151;
  border: 1px solid #4b5563;
  color: #d1d5db;
  font-size: 0.65rem;
  padding: 2px 10px;
  border-radius: 4px;
  cursor: pointer;
}
.action-btn:hover:not(:disabled) {
  background-color: #4b5563;
  color: white;
}
.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.action-btn.save-btn {
  background-color: #2563eb;
  border-color: #3b82f6;
  color: white;
}
.action-btn.save-btn:hover:not(:disabled) {
  background-color: #1d4ed8;
}

.tab-content {
  flex: 1;
  overflow: hidden;
  position: relative;
  background-color: #0a0a0a;
}

/* Common Content Styles */
.log-content, .yaml-content, .events-content, .files-content {
  height: 100%;
  overflow-y: auto;
}

/* Logs Styles */
.log-content {
  padding: 0.5rem;
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

/* YAML & Code Editor Styles */
.yaml-content, .file-editor-area {
  height: 100%;
  display: flex;
}
.code-editor {
  flex: 1;
  background-color: #0a0a0a;
  color: #a78bfa; /* Light purple for YAML/Code */
  border: none;
  padding: 1rem;
  font-family: inherit;
  font-size: 0.8rem;
  resize: none;
  outline: none;
  line-height: 1.5;
}

/* Events Table Styles */
.events-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.75rem;
  color: #d1d5db;
}
.events-table th {
  text-align: left;
  background-color: #111827;
  padding: 0.5rem;
  position: sticky;
  top: 0;
  color: #9ca3af;
  border-bottom: 1px solid #374151;
}
.events-table td {
  padding: 0.5rem;
  border-bottom: 1px solid #1f2937;
  vertical-align: top;
}
.type-cell.warning { color: #f59e0b; }
.type-cell.normal { color: #10b981; }

/* Files Styles */
.files-content {
  display: flex;
  flex-direction: column;
}
.file-browser-header {
  display: flex;
  gap: 0.5rem;
  padding: 0.5rem;
  background-color: #111827;
  border-bottom: 1px solid #374151;
}
.path-input {
  flex: 1;
  background-color: #1f2937;
  border: 1px solid #374151;
  color: #d1d5db;
  font-size: 0.75rem;
  padding: 2px 8px;
  border-radius: 4px;
  font-family: inherit;
}

/* Shared UI */
.loading-overlay {
  position: absolute;
  inset: 0;
  background-color: rgba(0,0,0,0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  color: #3b82f6;
  font-size: 0.8rem;
  z-index: 10;
}
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: #4b5563;
  font-size: 0.8rem;
  font-style: italic;
  padding: 2rem;
  text-align: center;
}
</style>
