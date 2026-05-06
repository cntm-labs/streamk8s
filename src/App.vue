<script setup lang="ts">
import { ref } from "vue";

const activeTab = ref("unified-view");
</script>

<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <img src="/tauri.svg" class="logo tauri" alt="StreamK8s logo" />
        <span>StreamK8s</span>
      </div>
      <nav class="sidebar-nav">
        <button 
          :class="{ active: activeTab === 'unified-view' }" 
          @click="activeTab = 'unified-view'"
        >
          Unified View
        </button>
        <button 
          :class="{ active: activeTab === 'marketplace' }" 
          @click="activeTab = 'marketplace'"
        >
          Marketplace
        </button>
        <button 
          :class="{ active: activeTab === 'settings' }" 
          @click="activeTab = 'settings'"
        >
          Settings
        </button>
      </nav>
    </aside>
    <main class="main-content">
      <header class="content-header">
        <h1>{{ activeTab === 'unified-view' ? 'Unified View' : activeTab.charAt(0).toUpperCase() + activeTab.slice(1) }}</h1>
      </header>
      <section class="content-body">
        <div v-if="activeTab === 'unified-view'" class="unified-view">
          <p>Welcome to the Unified View. Management of desktop apps and Kubernetes resources starts here.</p>
          <div class="placeholder-grid">
            <div class="card">Desktop Applications</div>
            <div class="card">Kubernetes Pods</div>
            <div class="card">Resource Allocation</div>
          </div>
        </div>
        <div v-else>
          <p>Content for {{ activeTab }} is coming soon.</p>
        </div>
      </section>
    </main>
  </div>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #ffffff;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  display: flex;
  place-items: center;
  min-width: 320px;
  min-height: 100vh;
}

#app {
  width: 100%;
  height: 100vh;
}

.app-layout {
  display: flex;
  width: 100%;
  height: 100%;
}

.sidebar {
  width: 260px;
  background-color: #f6f6f6;
  border-right: 1px solid #e2e2e2;
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 1.5rem;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-weight: bold;
  font-size: 1.2rem;
}

.sidebar-header .logo {
  height: 32px;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  padding: 0.5rem;
  gap: 0.25rem;
}

.sidebar-nav button {
  text-align: left;
  padding: 0.75rem 1rem;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1rem;
  color: #444;
  transition: background 0.2s;
}

.sidebar-nav button:hover {
  background-color: #eeeeee;
}

.sidebar-nav button.active {
  background-color: #e2e2e2;
  color: #000;
  font-weight: 600;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content-header {
  padding: 1rem 2rem;
  border-bottom: 1px solid #e2e2e2;
}

.content-header h1 {
  margin: 0;
  font-size: 1.5rem;
}

.content-body {
  padding: 2rem;
  overflow-y: auto;
  flex: 1;
}

.placeholder-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-top: 2rem;
}

.card {
  padding: 2rem;
  background-color: #f9f9f9;
  border: 1px solid #e2e2e2;
  border-radius: 12px;
  text-align: center;
  font-weight: 500;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #1a1a1a;
  }

  .sidebar {
    background-color: #242424;
    border-right-color: #333;
  }

  .sidebar-nav button {
    color: #ccc;
  }

  .sidebar-nav button:hover {
    background-color: #333;
  }

  .sidebar-nav button.active {
    background-color: #444;
    color: #fff;
  }

  .content-header {
    border-bottom-color: #333;
  }

  .card {
    background-color: #2a2a2a;
    border-color: #444;
  }
}
</style>