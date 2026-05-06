<script setup lang="ts">
import { ref } from 'vue';

const currentMode = ref<'dashboard' | 'terminal'>('dashboard');
</script>

<template>
  <div class="app-container">
    <nav class="sidebar">
      <div class="logo">SK8s</div>
      <button @click="currentMode = 'dashboard'">D</button>
      <button @click="currentMode = 'terminal'">T</button>
    </nav>
    <main class="content">
      <div v-if="currentMode === 'dashboard'">Dashboard Area (Placeholder)</div>
      <div v-if="currentMode === 'terminal'">Terminal Area (Placeholder)</div>
    </main>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  width: 100vw;
  background-color: #111827; /* gray-900 */
  color: white;
  font-family: sans-serif;
}
.sidebar {
  width: 64px;
  background-color: #1f2937; /* gray-800 */
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 1rem;
  border-right: 1px solid #374151;
}
.logo {
  color: #3b82f6; /* blue-500 */
  margin-bottom: 2rem;
  font-weight: bold;
}
.sidebar button {
  width: 40px;
  height: 40px;
  margin-bottom: 1rem;
  background-color: #374151; /* gray-700 */
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
}
.sidebar button:hover {
  background-color: #4b5563; /* gray-600 */
}
.content {
  flex: 1;
  padding: 1.5rem;
}
</style>
