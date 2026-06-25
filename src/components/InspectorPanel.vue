<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { Terminal } from 'xterm';
import 'xterm/css/xterm.css';

const props = defineProps<{
  selectedResource: { contextName: string, namespace: string, name: string, kind: string } | null,
  containerName?: string
}>();

const emit = defineEmits<{ (e: 'close'): void }>();

const activeTab = ref('Logs');
const tabs = ['Logs', 'Terminal'];

// Logs State
const logs = ref<string[]>([]);
const logContainer = ref<HTMLElement | null>(null);
let unlisten: UnlistenFn;

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
        containerName: props.containerName || res.name,
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
  const res = props.selectedResource;
  if (!res || res.kind !== 'Pods') return;
  
  try {
    logs.value = ["Connecting to pod logs..."];
    await invoke('start_log_stream', {
      contextName: res.contextName,
      namespace: res.namespace,
      podName: res.name,
      containerName: props.containerName || null
    });
  } catch (e) {
    logs.value.push(`Stream Error: ${e}`);
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
  destroyTerminal();
});

watch(activeTab, (newTab, oldTab) => {
  if (oldTab === 'Terminal') destroyTerminal();
  if (newTab === 'Terminal') initTerminal();
  if (newTab === 'Logs') initiateLogStream();
});

watch(() => props.selectedResource, () => {
  clearLogs();
  destroyTerminal();
  if (activeTab.value === 'Terminal') initTerminal();
  if (activeTab.value === 'Logs') initiateLogStream();
}, { deep: true });

watch(() => props.containerName, () => {
  clearLogs();
  destroyTerminal();
  if (activeTab.value === 'Terminal') initTerminal();
  if (activeTab.value === 'Logs') initiateLogStream();
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
          {{ t === 'Logs' ? '📝 Logs' : '💻 Terminal' }}
        </button>
      </div>
      <div class="actions">
        <button v-if="activeTab === 'Logs'" @click="clearLogs" class="action-btn">Clear</button>
        <div class="header-divider"></div>
        <button class="btn-close" @click="emit('close')" title="Close Diagnostics Panel">✕</button>
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
    </div>
  </div>
</template>

<style scoped>
.inspector-panel {
  height: 280px;
  background-color: #0c0c0c;
  border-top: 1px solid #1e293b;
  display: flex;
  flex-direction: column;
  font-family: var(--font-ui);
}
.panel-header {
  height: 38px;
  background-color: #111827;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid #1e293b;
}
.tabs {
  display: flex;
  gap: 8px;
  align-items: center;
}
.tab-btn {
  background: transparent;
  border: none;
  color: #9ca3af;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 6px 16px;
  cursor: pointer;
  transition: all 0.2s;
  border-bottom: 2px solid transparent;
}
.tab-btn:hover {
  color: #f3f4f6;
}
.tab-btn.active {
  color: #3b82f6;
  border-bottom: 2px solid #3b82f6;
}
.actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.action-btn {
  background-color: #1f2937;
  border: 1px solid #374151;
  color: #d1d5db;
  font-size: 0.75rem;
  padding: 3px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}
.action-btn:hover {
  border-color: #3b82f6;
  color: white;
}
.header-divider {
  width: 1px;
  height: 16px;
  background-color: #374151;
  margin: 0 4px;
}
.btn-close {
  background: none;
  border: none;
  color: #9ca3af;
  cursor: pointer;
  padding: 4px 8px;
  font-size: 0.85rem;
  transition: all 0.2s;
}
.btn-close:hover {
  color: #f87171;
}

.tab-content {
  flex: 1;
  overflow: hidden;
  position: relative;
  background-color: #050505;
}
.log-content {
  height: 100%;
  overflow-y: auto;
  padding: 12px 16px;
  font-family: var(--font-code);
}
.log-line {
  display: flex;
  gap: 12px;
  font-size: 0.8rem;
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
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: #4b5563;
  font-size: 0.8rem;
  font-style: italic;
}
.terminal-content {
  height: 100%;
  padding: 8px;
  background-color: #0a0a0a;
}
.terminal-panel-body {
  height: 100%;
  width: 100%;
}
</style>
