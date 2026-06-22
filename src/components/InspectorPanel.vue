<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';
import { Loader2 } from 'lucide-vue-next';

const props = defineProps<{
  selectedResource: { contextName: string, namespace: string, name: string, kind: string } | null
}>();

const emit = defineEmits<{ (e: 'close'): void; (e: 'edit', resource: any): void; }>();

const activeTab = ref('Logs');
const tabs = ['Logs', 'YAML', 'Events', 'Files', 'AI Diagnostic'];

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

// AI Diagnostic State
const aiAdvice = ref('');
const isAnalyzing = ref(false);
const apiKey = ref('');

// Container State
const containerNames = ref<string[]>([]);
const selectedContainer = ref('');

watch(() => props.selectedResource, async (newVal) => {
  if (newVal && newVal.kind === 'Pods') {
    try {
      const podDetailsText = await invoke<string>('get_k8s_resource_details', {
        contextName: newVal.contextName,
        namespace: newVal.namespace,
        name: newVal.name,
        kind: 'Pod'
      });
      const parsed = JSON.parse(podDetailsText);
      const names = parsed.spec?.containers?.map((c: any) => c.name) || [];
      containerNames.value = names;
      if (names.length > 0) {
        selectedContainer.value = names[0];
      }
    } catch (e) {
      console.error('Failed to load container list:', e);
      containerNames.value = [newVal.name];
      selectedContainer.value = newVal.name;
    }
  }
}, { immediate: true });

const clearLogs = () => {
  logs.value = [];
};

const initiateLogStream = async () => {
  if (!props.selectedResource || props.selectedResource.kind !== 'Pods') return;
  
  try {
    logs.value = ["Connecting to pod logs..."];
    await invoke('start_log_stream', {
      contextName: props.selectedResource.contextName,
      namespace: props.selectedResource.namespace,
      podName: props.selectedResource.name,
      containerName: selectedContainer.value || null
    });
  } catch (e) {
    logs.value.push(`Stream Error: ${e}`);
  }
};

const fetchYaml = async () => {
  if (!props.selectedResource) return;
  isLoadingYaml.value = true;
  try {
    yamlContent.value = await invoke('get_k8s_resource_details', {
      contextName: props.selectedResource.contextName,
      namespace: props.selectedResource.namespace,
      name: props.selectedResource.name,
      kind: props.selectedResource.kind.replace(/s$/, '')
    });
  } catch (e) {
    yamlContent.value = `Error fetching YAML: ${e}`;
  } finally {
    isLoadingYaml.value = false;
  }
};

const fetchEvents = async () => {
  if (!props.selectedResource) return;
  isLoadingEvents.value = true;
  try {
    const response: any = await invoke('get_pod_events', {
      contextName: props.selectedResource.contextName,
      namespace: props.selectedResource.namespace,
      podName: props.selectedResource.name
    });
    events.value = response.items || [];
  } catch (e) {
    console.error('Failed to fetch events:', e);
  } finally {
    isLoadingEvents.value = false;
  }
};

const readFile = async () => {
  if (!props.selectedResource) return;
  isLoadingFiles.value = true;
  try {
    fileContent.value = await invoke('read_pod_file', {
      contextName: props.selectedResource.contextName,
      namespace: props.selectedResource.namespace,
      podName: props.selectedResource.name,
      containerName: selectedContainer.value || props.selectedResource.name,
      filePath: filePath.value
    });
  } catch (e) {
    fileContent.value = `Error reading file: ${e}\n(Note: Container name might be different from pod name)`;
  } finally {
    isLoadingFiles.value = false;
  }
};

const applyYaml = async () => {
  if (!props.selectedResource) return;
  isLoadingYaml.value = true;
  try {
    await invoke('apply_resource_manifest', {
      contextName: props.selectedResource.contextName,
      namespace: props.selectedResource.namespace,
      podName: props.selectedResource.name,
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
  if (!props.selectedResource) return;
  isLoadingFiles.value = true;
  try {
    // UTF-8 to Base64
    const contentBase64 = window.btoa(unescape(encodeURIComponent(fileContent.value)));
    await invoke('write_pod_file', {
      contextName: props.selectedResource.contextName,
      namespace: props.selectedResource.namespace,
      podName: props.selectedResource.name,
      containerName: selectedContainer.value || props.selectedResource.name,
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

const runAiAnalysis = async () => {
  if (!props.selectedResource) return;
  
  if (!apiKey.value) {
    const key = prompt('Please enter your Gemini API Key:');
    if (!key) return;
    apiKey.value = key;
  }

  isAnalyzing.value = true;
  aiAdvice.value = 'AI is analyzing the resource state and logs...';
  
  try {
    const result: string = await invoke('analyze_with_ai', {
      contextName: props.selectedResource.contextName,
      kind: props.selectedResource.kind.replace(/s$/, ''),
      namespace: props.selectedResource.namespace,
      resourceName: props.selectedResource.name
    });
    aiAdvice.value = result;
  } catch (e) {
    aiAdvice.value = `### Error during AI Analysis\n\n${e}`;
  } finally {
    isAnalyzing.value = false;
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

  if (activeTab.value === 'Logs') {
    initiateLogStream();
  }
});

onUnmounted(() => {
  if (unlisten) unlisten();
});

watch(activeTab, (newTab) => {
  if (newTab === 'Logs') initiateLogStream();
  if (newTab === 'YAML') fetchYaml();
  if (newTab === 'Events') fetchEvents();
  if (newTab === 'Files') readFile();
});

watch(() => props.selectedResource, () => {
  clearLogs();
  if (activeTab.value === 'Logs') initiateLogStream();
  if (activeTab.value === 'YAML') fetchYaml();
  if (activeTab.value === 'Events') fetchEvents();
  if (activeTab.value === 'Files') readFile();
}, { deep: true });

watch(selectedContainer, () => {
  clearLogs();
  if (activeTab.value === 'Logs') initiateLogStream();
  if (activeTab.value === 'Files') readFile();
});

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
      <div class="container-select-wrapper" v-if="props.selectedResource && props.selectedResource.kind === 'Pods'">
        <label for="container-select" class="container-label">Container:</label>
        <select id="container-select" v-model="selectedContainer" class="container-selector">
          <option v-for="c in containerNames" :key="c" :value="c">{{ c }}</option>
        </select>
      </div>
      <div class="actions">
        <button v-if="activeTab === 'Logs'" @click="clearLogs" class="action-btn">Clear</button>
        <button v-if="activeTab === 'YAML'" @click="fetchYaml" class="action-btn" :disabled="isLoadingYaml">Refresh</button>
        <button v-if="activeTab === 'YAML'" @click="applyYaml" class="action-btn save-btn" :disabled="isLoadingYaml">Apply</button>
        <button v-if="activeTab === 'Events'" @click="fetchEvents" class="action-btn" :disabled="isLoadingEvents">Refresh</button>
        <button v-if="activeTab === 'Files'" @click="saveFile" class="action-btn save-btn" :disabled="isLoadingFiles">Save</button>
        <button v-if="activeTab === 'AI Diagnostic'" @click="runAiAnalysis" class="action-btn ai-btn" :disabled="isAnalyzing">AI Analyze</button>
        
        <div class="header-divider"></div>
        <div class="header-actions">
          <button class="btn-icon" @click="emit('edit', selectedResource)" title="Edit YAML">✏️</button>
          <button class="btn-icon" @click="emit('close')" title="Close Panel">✕</button>
        </div>
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
          {{ selectedResource ? 'Streaming logs...' : 'Select a resource to start streaming logs...' }}
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
              <td colspan="4" class="empty-state">No events found for this {{ selectedResource?.kind || 'resource' }}.</td>
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

      <!-- AI DIAGNOSTIC TAB -->
      <div v-if="activeTab === 'AI Diagnostic'" class="ai-content">
        <div v-if="isAnalyzing" class="loading-overlay">
          <Loader2 class="animate-spin mr-2" :size="18" />
          <span>AI is analyzing resource context (YAML + Events)...</span>
        </div>
        <div v-if="aiAdvice" class="markdown-body" v-html="marked.parse(aiAdvice)"></div>
        <div v-else class="empty-state">
          Click "AI Analyze" to get diagnostic advice from Gemini AI.
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inspector-panel {
  height: 350px;
  background-color: var(--surface-dark);
  border-top: 1px solid var(--border-dim);
  display: flex;
  flex-direction: column;
  font-family: var(--font-ui);
}
.panel-header {
  height: 48px;
  background-color: var(--surface-card);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 var(--space-4);
  border-bottom: 1px solid var(--border-dim);
}
.container-select-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 16px;
}
.container-label {
  font-size: 0.75rem;
  color: #9ca3af;
  font-weight: 600;
}
.container-selector {
  background-color: var(--surface-dark);
  border: 1px solid var(--border-dim);
  color: #d1d5db;
  font-size: 0.75rem;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  outline: none;
  cursor: pointer;
  transition: all 0.2s;
}
.container-selector:hover {
  border-color: var(--accent-blue);
  color: white;
}
.tabs {
  display: flex;
  gap: var(--space-2);
  align-items: center;
}
.tab-btn {
  background: var(--surface-dark);
  border: 1px solid var(--border-dim);
  color: #9ca3af;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 6px 16px;
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all 0.2s;
}
.tab-btn:hover {
  color: #f3f4f6;
  border-color: var(--accent-blue);
}
.tab-btn.active {
  color: white;
  background-color: var(--accent-blue);
  border-color: var(--accent-blue);
  box-shadow: 0 0 10px var(--accent-blue-glow);
}
.actions {
  display: flex;
  gap: var(--space-2);
}
.action-btn {
  background-color: var(--surface-dark);
  border: 1px solid var(--border-dim);
  color: #d1d5db;
  font-size: 0.75rem;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.2s;
}
.action-btn:hover:not(:disabled) {
  border-color: var(--accent-blue);
  color: white;
}
.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.action-btn.save-btn {
  background-color: var(--accent-blue);
  border-color: var(--accent-blue);
  color: white;
}
.action-btn.save-btn:hover:not(:disabled) {
  background-color: #1d4ed8;
}
.action-btn.ai-btn {
  background: linear-gradient(135deg, #6366f1 0%, #a855f7 100%);
  border: none;
  color: white;
}
.action-btn.ai-btn:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.header-divider {
  width: 1px;
  height: 20px;
  background-color: var(--border-dim);
  margin: 0 4px;
}

.btn-icon {
  background: none;
  border: none;
  color: #9ca3af;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px;
  border-radius: var(--radius-sm);
  transition: all 0.2s;
}

.btn-icon:hover {
  background-color: rgba(255, 255, 255, 0.05);
  color: white;
}

.tab-content {
  flex: 1;
  overflow: hidden;
  position: relative;
  background-color: #050505;
}

/* Common Content Styles */
.log-content, .yaml-content, .events-content, .files-content, .ai-content {
  height: 100%;
  overflow-y: auto;
  padding: var(--space-4);
}

/* AI Diagnostic Styles */
.ai-content {
  color: #e5e7eb;
}
.markdown-body {
  font-size: 0.875rem;
  line-height: 1.6;
}
.markdown-body :deep(h1), .markdown-body :deep(h2), .markdown-body :deep(h3) {
  color: #a855f7;
  margin-top: 1.5rem;
  margin-bottom: 0.5rem;
}
.markdown-body :deep(code) {
  background-color: #1f2937;
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
  font-family: var(--font-code);
}
.markdown-body :deep(pre) {
  background-color: #111827;
  padding: 1rem;
  border-radius: 8px;
  overflow-x: auto;
  margin: 1rem 0;
}
.markdown-body :deep(ul), .markdown-body :deep(ol) {
  padding-left: 1.5rem;
  margin: 1rem 0;
}
.markdown-body :deep(blockquote) {
  border-left: 4px solid #4f46e5;
  padding-left: 1rem;
  color: #9ca3af;
  font-style: italic;
}

/* Logs Styles */
.log-content {
  font-family: var(--font-code);
}
.log-line {
  display: flex;
  gap: 1rem;
  font-size: 0.85rem;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
}
.line-num {
  color: #4b5563;
  min-width: 2.5rem;
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
  padding: 0;
}
.code-editor {
  flex: 1;
  background-color: transparent;
  color: #a78bfa;
  border: none;
  padding: var(--space-4);
  font-family: var(--font-code);
  font-size: 0.875rem;
  resize: none;
  outline: none;
  line-height: 1.6;
}

/* Events Table Styles */
.events-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8rem;
  color: #d1d5db;
}
.events-table th {
  text-align: left;
  background-color: var(--surface-card);
  padding: var(--space-3) var(--space-4);
  position: sticky;
  top: 0;
  color: #9ca3af;
  border-bottom: 1px solid var(--border-dim);
}
.events-table td {
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--border-dim);
  vertical-align: top;
}
.type-cell.warning { color: #f59e0b; }
.type-cell.normal { color: #10b981; }

/* Files Styles */
.files-content {
  display: flex;
  flex-direction: column;
  padding: 0;
}
.file-browser-header {
  display: flex;
  gap: 0.5rem;
  padding: var(--space-2) var(--space-4);
  background-color: var(--surface-card);
  border-bottom: 1px solid var(--border-dim);
}
.path-input {
  flex: 1;
  background-color: var(--surface-dark);
  border: 1px solid var(--border-dim);
  color: #d1d5db;
  font-size: 0.8rem;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  font-family: var(--font-code);
}

/* Shared UI */
.loading-overlay {
  position: absolute;
  inset: 0;
  background-color: rgba(0,0,0,0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  color: var(--accent-blue);
  font-size: 0.8rem;
  z-index: 10;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

.mr-2 {
  margin-right: 0.5rem;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: #4b5563;
  font-size: 0.875rem;
  font-style: italic;
  padding: 2rem;
  text-align: center;
}
</style>
