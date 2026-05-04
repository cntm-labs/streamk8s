# StreamK8s Initialization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Scaffold the core StreamK8s project structure using Rust, Tauri, and Vue, establishing the foundation for the Unified View UI.

**Architecture:** Initialize a standard Tauri v2 application utilizing Vue (TypeScript) for the frontend and Rust for the backend engine. Ensure the initial layout supports a sidebar and a main content area for the "Unified View".

**Tech Stack:** Rust, Tauri (v2), Vue 3, TypeScript, Vite.

---

### Task 1: Initialize Project Scaffolding

**Files:**
- Create: `package.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src/App.vue`
- Create: `src/main.ts`

- [ ] **Step 1: Run Tauri Create Command**

Run: `npm create tauri-app@latest . -- --rc --manager npm --template vue-ts`
Expected: PASS with project successfully scaffolded in current directory.

- [ ] **Step 2: Verify Initial Build**

Run: `npm install`
Run: `npm run tauri build`
Expected: PASS with successful compilation of Rust and Vue code.

- [ ] **Step 3: Commit**

```bash
git add .
git commit -m "chore(init): Scaffold StreamK8s with Tauri, Vue, and Rust"
```

### Task 2: Setup Unified View Layout Skeleton

**Files:**
- Modify: `src/App.vue`
- Modify: `src/style.css`

- [ ] **Step 1: Write the basic layout template**

Replace `src/App.vue` with:
```vue
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
```

- [ ] **Step 2: Clear default styles**

Replace `src/style.css` with:
```css
html, body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}
```

- [ ] **Step 3: Run dev server to verify**

Run: `npm run tauri dev`
Expected: Application opens showing the new sidebar and main content area.
*Action:* Close the application after visual confirmation.

- [ ] **Step 4: Commit**

```bash
git add src/App.vue src/style.css
git commit -m "feat(ui): Implement basic Unified View skeleton layout"
```
