<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Terminal } from 'xterm';
import 'xterm/css/xterm.css';

const props = defineProps<{
  contextName: string | null;
  namespace: string;
  podName: string;
  containerName: string;
}>();

const terminalContainer = ref<HTMLElement | null>(null);
const sessionId = ref(`term-${Math.random().toString(36).substring(2, 9)}`);
let term: Terminal;
let unlistenStdout: UnlistenFn | null = null;
let unlistenExit: UnlistenFn | null = null;

onMounted(async () => {
  if (!terminalContainer.value) return;

  term = new Terminal({
    cursorBlink: true,
    theme: {
      background: '#0f172a'
    }
  });

  term.open(terminalContainer.value);
  term.writeln(`Connecting to ${props.podName}...`);

  term.onData((data) => {
    invoke('send_terminal_input', {
      sessionId: sessionId.value,
      data
    });
  });

  unlistenStdout = await listen<string>(`terminal-stdout-${sessionId.value}`, (event) => {
    term.write(event.payload);
  });

  unlistenExit = await listen<void>(`terminal-exit-${sessionId.value}`, () => {
    term.writeln('\r\n[Process Exited]');
  });

  try {
    await invoke('start_terminal_session', {
      contextName: props.contextName,
      namespace: props.namespace,
      podName: props.podName,
      containerName: props.containerName || '',
      sessionId: sessionId.value
    });
    term.write('\x1b[2K\r'); // Clear the "Connecting" line
  } catch (e) {
    console.error("Failed to start terminal session", e);
    term.writeln(`\r\nError: ${e}`);
  }
});

onUnmounted(async () => {
  if (unlistenStdout) unlistenStdout();
  if (unlistenExit) unlistenExit();
  if (term) term.dispose();

  try {
    await invoke('close_terminal_session', { sessionId: sessionId.value });
  } catch (e) {
    console.error("Failed to close terminal session", e);
  }
});
</script>

<template>
  <div class="pod-terminal" ref="terminalContainer"></div>
</template>

<style scoped>
.pod-terminal {
  width: 100%;
  height: 100%;
  background-color: #0f172a;
  padding: 8px;
  box-sizing: border-box;
}
:deep(.xterm) {
  height: 100%;
}
</style>
