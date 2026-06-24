<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch, computed } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';
import { Loader2 } from 'lucide-vue-next';
import { Terminal } from 'xterm';
import 'xterm/css/xterm.css';

const props = defineProps<{
  selectedResource: { contextName: string, namespace: string, name: string, kind: string } | null
}>();

const emit = defineEmits<{ (e: 'close'): void; (e: 'edit', resource: any): void; }>();

const activeTab = ref('YAML'); // Set default to YAML since it is universally supported

const filteredTabs = computed(() => {
  const isPod = props.selectedResource?.kind === 'Pods';
  if (isPod) {
    return [
      { id: 'Logs', label: '📝 Logs' },
      { id: 'Terminal', label: '💻 Terminal' },
      { id: 'YAML', label: '⚙️ YAML' },
      { id: 'Events', label: '🔔 Events' },
      { id: 'Files', label: '📂 Files' },
      { id: 'AI Diagnostic', label: '🤖 AI Diagnostic' }
    ];
  } else {
    return [
      { id: 'YAML', label: '⚙️ YAML' },
      { id: 'Events', label: '🔔 Events' },
      { id: 'AI Diagnostic', label: '🤖 AI Diagnostic' }
    ];
  }
});

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

// Files State (Pod File Explorer Upgrade)
const fileContent = ref('');
const filePath = ref('/'); // Default to root
const fileList = ref<Array<{ name: string; is_dir: boolean }>>([]);
const selectedFile = ref<string | null>(null);
const isLoadingFiles = ref(false);
const isListingFiles = ref(false);

// AI Diagnostic State
const aiAdvice = ref('');
const isAnalyzing = ref(false);
const apiKey = ref('');
const isApiKeySaved = ref(false);
const showKeyInput = ref(false);

// Container State
const containerNames = ref<string[]>([]);
const selectedContainer = ref('');

// Load API Key natively from Tauri config
const loadApiKey = async () => {
  try {
    const config = await invoke<any>('get_config');
    if (config.api_key) {
      apiKey.value = config.api_key;
      isApiKeySaved.value = true;
    } else {
      isApiKeySaved.value = false;
    }
  } catch (e) {
    console.error('Failed to load config:', e);
  }
};

const saveApiKey = async () => {
  if (!apiKey.value) return;
  try {
    const config = await invoke<any>('get_config');
    config.api_key = apiKey.value;
    await invoke('save_config', { config });
    isApiKeySaved.value = true;
    showKeyInput.value = false;
  } catch (e) {
    alert('Failed to save API Key: ' + e);
  }
};

const listFiles = async () => {
  const res = props.selectedResource;
  if (!res || res.kind !== 'Pods') return;
  isListingFiles.value = true;
  try {
    const files: any = await invoke('list_pod_files', {
      contextName: res.contextName,
      namespace: res.namespace,
      podName: res.name,
      containerName: selectedContainer.value || res.name,
      dirPath: filePath.value
    });
    // Add parent option if not in root
    if (filePath.value !== '/' && filePath.value !== '') {
      fileList.value = [{ name: '..', is_dir: true }, ...files];
    } else {
      fileList.value = files;
    }
  } catch (e) {
    console.error('Failed to list files:', e);
    fileList.value = [];
  } finally {
    isListingFiles.value = false;
  }
};

const handleItemClick = async (item: { name: string; is_dir: boolean }) => {
  let currentPath = filePath.value;
  if (!currentPath.endsWith('/') && currentPath !== '') {
    currentPath += '/';
  }
  
  if (item.is_dir) {
    if (item.name === '..') {
      const parts = currentPath.split('/').filter(Boolean);
      parts.pop();
      filePath.value = '/' + parts.join('/');
    } else {
      filePath.value = currentPath + item.name;
    }
    await listFiles();
  } else {
    selectedFile.value = currentPath + item.name;
    isLoadingFiles.value = true;
    try {
      fileContent.value = await invoke('read_pod_file', {
        contextName: props.selectedResource?.contextName,
        namespace: props.selectedResource?.namespace,
        podName: props.selectedResource?.name,
        containerName: selectedContainer.value || props.selectedResource?.name,
        filePath: selectedFile.value
      });
    } catch (e) {
      fileContent.value = `Error reading file: ${e}`;
    } finally {
      isLoadingFiles.value = false;
    }
  }
};

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

// Terminal State & Logic
const terminalDiv = ref<HTMLElement | null>(null);
let term: Terminal | null = null;
const sessionId = ref('');
let stdoutListener: UnlistenFn | null = null;
let exitListener: UnlistenFn | null = null;

const initTerminal = async () => {
  const res = props.selectedResource;
  if (!res || res.kind !== 'Pods') return;
  
  destroyTerminal();
  
  sessionId.value = Math.random().toString(36).substring(7);
  
  term = new Terminal({
    cols: 120,
    rows: 35,
    theme: {
      background: '#0a0a0a',
      foreground: '#f8f8f2',
      cursor: '#f8f8f0',
    }
  });

  nextTick(async () => {
    if (!terminalDiv.value || !term) return;
    term.open(terminalDiv.value);
    term.write('Connecting to pod terminal...\r\n');

    try {
      await invoke('start_terminal_session', {
        contextName: res.contextName,
        namespace: res.namespace,
        podName: res.name,
        containerName: selectedContainer.value || res.name,
        sessionId: sessionId.value
      });

      stdoutListener = await listen<string>(`terminal-stdout-${sessionId.value}`, (event) => {
        term?.write(event.payload);
      });

      exitListener = await listen<void>(`terminal-exit-${sessionId.value}`, () => {
        term?.write('\r\n[Connection Closed]\r\n');
      });

      if (term) {
        term.onData((data) => {
          invoke('send_terminal_input', {
            sessionId: sessionId.value,
            data
          });
        });
      }
    } catch (e) {
      if (term) {
        term.write(`\r\nError launching terminal: ${e}\r\n`);
      }
    }
  });
};

const destroyTerminal = () => {
  if (stdoutListener) {
    stdoutListener();
    stdoutListener = null;
  }
  if (exitListener) {
    exitListener();
    exitListener = null;
  }
  if (sessionId.value) {
    invoke('close_terminal_session', { sessionId: sessionId.value }).catch(() => {});
    sessionId.value = '';
  }
  if (term) {
    term.dispose();
    term = null;
  }
};

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
  if (filePath.value.endsWith('/') || filePath.value === '' || filePath.value === '/') {
    await listFiles();
  } else {
    selectedFile.value = filePath.value;
    isLoadingFiles.value = true;
    try {
      fileContent.value = await invoke('read_pod_file', {
        contextName: props.selectedResource.contextName,
        namespace: props.selectedResource.namespace,
        podName: props.selectedResource.name,
        containerName: selectedContainer.value || props.selectedResource.name,
        filePath: selectedFile.value
      });
    } catch (e) {
      fileContent.value = `Error reading file: ${e}\n(Note: Container name might be different from pod name)`;
    } finally {
      isLoadingFiles.value = false;
    }
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
  if (!props.selectedResource || !selectedFile.value) return;
  isLoadingFiles.value = true;
  try {
    const contentBase64 = window.btoa(unescape(encodeURIComponent(fileContent.value)));
    await invoke('write_pod_file', {
      contextName: props.selectedResource.contextName,
      namespace: props.selectedResource.namespace,
      podName: props.selectedResource.name,
      containerName: selectedContainer.value || props.selectedResource.name,
      filePath: selectedFile.value,
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
    await loadApiKey();
    if (!apiKey.value) return;
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
  loadApiKey();
  
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

  const isPod = props.selectedResource?.kind === 'Pods';
  if (!isPod) {
    activeTab.value = 'YAML';
  } else {
    activeTab.value = 'Logs';
  }

  if (activeTab.value === 'Logs') {
    initiateLogStream();
  }
});

onUnmounted(() => {
  if (unlisten) unlisten();
  destroyTerminal();
});

watch(activeTab, (newTab, oldTab) => {
  if (oldTab === 'Terminal') destroyTerminal();
  if (newTab === 'Terminal') initTerminal();
  if (newTab === 'Logs') initiateLogStream();
  if (newTab === 'YAML') fetchYaml();
  if (newTab === 'Events') fetchEvents();
  if (newTab === 'Files') readFile();
});

watch(() => props.selectedResource, (newVal) => {
  clearLogs();
  destroyTerminal();
  
  if (newVal && newVal.kind !== 'Pods') {
    if (['Logs', 'Terminal', 'Files'].includes(activeTab.value)) {
      activeTab.value = 'YAML';
    }
  } else if (newVal && newVal.kind === 'Pods' && activeTab.value === 'YAML') {
    activeTab.value = 'Logs';
  }
  
  if (activeTab.value === 'Terminal') initTerminal();
  if (activeTab.value === 'Logs') initiateLogStream();
  if (activeTab.value === 'YAML') fetchYaml();
  if (activeTab.value === 'Events') fetchEvents();
  if (activeTab.value === 'Files') {
    filePath.value = '/';
    readFile();
  }
}, { deep: true });

watch(selectedContainer, () => {
  clearLogs();
  if (activeTab.value === 'Terminal') initTerminal();
  if (activeTab.value === 'Logs') initiateLogStream();
  if (activeTab.value === 'Files') {
    filePath.value = '/';
    readFile();
  }
});

defineExpose({ clearLogs });
</script>

<template>
  <div class="inspector-panel">
    <div class="panel-header">
      <div class="tabs">
        <button 
          v-for="t in filteredTabs" 
          :key="t.id" 
          @click="activeTab = t.id" 
          :class="['tab-btn', { active: activeTab === t.id }]"
        >
          {{ t.label }}
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
        <button v-if="activeTab === 'AI Diagnostic' && isApiKeySaved && !showKeyInput" @click="runAiAnalysis" class="action-btn ai-btn" :disabled="isAnalyzing">AI Analyze</button>
        
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

      <!-- TERMINAL TAB -->
      <div v-if="activeTab === 'Terminal'" class="terminal-content">
        <div ref="terminalDiv" class="terminal-panel-body"></div>
      </div>

      <!-- YAML TAB -->
      <div v-if="activeTab === 'YAML'" class="yaml-content">
        <div v-if="isLoadingYaml" class="loading-overlay">Loading manifest...</div>
        <textarea v-model="yamlContent" class="code-editor" placeholder="YAML manifest content..."></textarea>
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
              <td class="type-cell">
                <span :class="['severity-badge', event.type.toLowerCase()]">
                  {{ event.type === 'Warning' ? '⚠️ ' + event.type : '✅ ' + event.type }}
                </span>
              </td>
              <td class="bold-cell">{{ event.reason }}</td>
              <td>{{ event.message }}</td>
              <td class="time-cell">{{ new Date(event.lastTimestamp || event.eventTime).toLocaleString() }}</td>
            </tr>
            <tr v-if="events.length === 0">
              <td colspan="4" class="empty-state">No events found for this {{ selectedResource?.kind || 'resource' }}.</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- FILES TAB (Pod File Explorer Split-Pane Redesign) -->
      <div v-if="activeTab === 'Files'" class="files-content-wrapper">
        <div class="file-browser-header">
          <button @click="handleItemClick({ name: '..', is_dir: true })" class="action-btn back-btn" :disabled="filePath === '/'">⬆️ Up</button>
          <input v-model="filePath" class="path-input" @keyup.enter="readFile" />
          <button @click="readFile" class="action-btn" :disabled="isLoadingFiles || isListingFiles">Go</button>
        </div>
        
        <div class="split-pane">
          <!-- Left Directory Listing -->
          <div class="directory-pane">
            <div v-if="isListingFiles" class="pane-loading">Listing...</div>
            <div v-else class="items-list">
              <div 
                v-for="item in fileList" 
                :key="item.name" 
                @click="handleItemClick(item)"
                :class="['file-item', item.is_dir ? 'directory' : 'file', { selected: selectedFile?.endsWith('/' + item.name) }]"
              >
                <span class="item-icon">{{ item.is_dir ? '📁' : '📄' }}</span>
                <span class="item-name">{{ item.name }}</span>
              </div>
              <div v-if="fileList.length === 0" class="empty-directory">Empty directory or permission denied.</div>
            </div>
          </div>
          
          <!-- Right File Editor -->
          <div class="editor-pane">
            <div class="editor-header-bar" v-if="selectedFile">
              <span class="editing-label">Editing: <strong>{{ selectedFile.substring(selectedFile.lastIndexOf('/') + 1) }}</strong></span>
              <button @click="saveFile" class="action-btn save-btn" :disabled="isLoadingFiles">💾 Save File</button>
            </div>
            <div class="editor-body">
              <div v-if="isLoadingFiles" class="loading-overlay">Reading file...</div>
              <textarea v-model="fileContent" class="code-editor" placeholder="Select a file from the explorer or enter a path above to edit..."></textarea>
            </div>
          </div>
        </div>
      </div>

      <!-- AI DIAGNOSTIC TAB -->
      <div v-if="activeTab === 'AI Diagnostic'" class="ai-content">
        <div v-if="!isApiKeySaved || showKeyInput" class="api-key-setup">
          <h3>🤖 AI Diagnostic Setup</h3>
          <p>Please enter your Gemini API Key to enable AI-powered Kubernetes diagnostics.</p>
          <div class="api-key-form">
            <input v-model="apiKey" type="password" placeholder="Gemini API Key..." class="api-input" />
            <button @click="saveApiKey" class="action-btn save-btn">Save Key</button>
            <button v-if="isApiKeySaved" @click="showKeyInput = false" class="action-btn">Cancel</button>
          </div>
        </div>
        <div v-else class="ai-running-view">
          <div class="ai-header-bar">
            <button @click="runAiAnalysis" class="action-btn ai-btn" :disabled="isAnalyzing">🤖 Run AI Analysis</button>
            <button @click="showKeyInput = true" class="action-btn secondary-btn">⚙️ Change API Key</button>
          </div>
          <div class="ai-output-area">
            <div v-if="isAnalyzing" class="loading-overlay">
              <Loader2 class="animate-spin mr-2" :size="18" />
              <span>AI is analyzing resource context (YAML + Events)...</span>
            </div>
            <div v-if="aiAdvice" class="markdown-body" v-html="marked.parse(aiAdvice)"></div>
            <div v-else class="empty-state">
              Click "Run AI Analysis" to get diagnostic advice from Gemini AI.
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Split Pane layout */
.files-content-wrapper {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.split-pane {
  flex: 1;
  display: flex;
  overflow: hidden;
  background-color: #030303;
  border-top: 1px solid var(--border-dim);
}
.directory-pane {
  width: 240px;
  border-right: 1px solid var(--border-dim);
  overflow-y: auto;
  padding: 8px;
}
.editor-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.editor-header-bar {
  height: 36px;
  background-color: var(--surface-card);
  border-bottom: 1px solid var(--border-dim);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
}
.editing-label {
  font-size: 0.75rem;
  color: #9ca3af;
}
.editor-body {
  flex: 1;
  position: relative;
  overflow: hidden;
  display: flex;
}

/* File items */
.items-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.15s;
  font-size: 0.75rem;
  color: #cbd5e1;
}
.file-item:hover {
  background-color: rgba(255,255,255,0.04);
  color: white;
}
.file-item.selected {
  background-color: rgba(99, 102, 241, 0.15);
  border-left: 2px solid #6366f1;
  color: white;
}
.item-icon {
  font-size: 0.9rem;
}
.item-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.empty-directory {
  color: #4b5563;
  font-size: 0.7rem;
  text-align: center;
  padding: 16px;
  font-style: italic;
}
.pane-loading {
  color: var(--accent-blue);
  font-size: 0.75rem;
  text-align: center;
  padding: 16px;
}

/* API Key setup view */
.api-key-setup {
  max-width: 400px;
  margin: 40px auto;
  background-color: var(--surface-card);
  padding: 24px;
  border-radius: 8px;
  border: 1px solid var(--border-dim);
  text-align: center;
}
.api-key-setup h3 {
  margin-top: 0;
  color: #a855f7;
  font-size: 1rem;
}
.api-key-setup p {
  font-size: 0.8rem;
  color: #9ca3af;
  margin-bottom: 20px;
  line-height: 1.4;
}
.api-key-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.api-input {
  background-color: var(--surface-dark);
  border: 1px solid var(--border-dim);
  color: white;
  font-size: 0.8rem;
  padding: 8px 12px;
  border-radius: 4px;
  outline: none;
}
.api-input:focus {
  border-color: #a855f7;
}

/* Severity Badges for Events */
.severity-badge {
  padding: 2px 8px;
  border-radius: 20px;
  font-size: 0.7rem;
  font-weight: bold;
  display: inline-block;
}
.severity-badge.warning {
  background-color: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.3);
}
.severity-badge.normal {
  background-color: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.3);
}
.bold-cell {
  font-weight: 600;
}
.time-cell {
  color: #9ca3af;
}

/* AI Diagnostics */
.ai-header-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}
.ai-running-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.ai-output-area {
  flex: 1;
  overflow-y: auto;
  position: relative;
}
.secondary-btn {
  background-color: transparent;
  border: 1px solid var(--border-dim);
  color: #9ca3af;
}

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
.terminal-content {
  height: 100%;
  padding: var(--space-2);
  background-color: #0a0a0a;
}
.terminal-panel-body {
  height: 100%;
  width: 100%;
}
</style>
