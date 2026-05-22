<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ShoppingBag, Box, Search, PackageOpen } from 'lucide-vue-next';
import PluginRenderer from '../components/PluginRenderer.vue';

interface ExtensionInfo {
  id: string;
  name: string;
  description: string;
  version: string;
}

interface PluginManifest {
  extension: ExtensionInfo;
  ui: any;
}

const installedPlugins = ref<PluginManifest[]>([]);
const selectedPlugin = ref<PluginManifest | null>(null);
const isLoading = ref(true);

const fetchPlugins = async () => {
  isLoading.value = true;
  try {
    const plugins = await invoke<PluginManifest[]>('get_installed_plugins');
    installedPlugins.value = plugins;
    if (plugins.length > 0) {
      selectedPlugin.value = plugins[0];
    }
  } catch (e) {
    console.error('Failed to fetch plugins:', e);
  } finally {
    isLoading.value = false;
  }
};

onMounted(fetchPlugins);
</script>

<template>
  <div class="marketplace-container">
    <div class="marketplace-sidebar">
      <div class="sidebar-header">
        <div class="title-row">
          <ShoppingBag :size="18" class="mr-2" />
          <h3>Marketplace</h3>
        </div>
        <div class="search-box">
          <Search :size="14" class="search-icon" />
          <input type="text" placeholder="Search extensions..." />
        </div>
      </div>

      <div class="plugin-list">
        <div 
          v-for="plugin in installedPlugins" 
          :key="plugin.extension.id"
          class="plugin-item"
          :class="{ active: selectedPlugin?.extension.id === plugin.extension.id }"
          @click="selectedPlugin = plugin"
        >
          <div class="plugin-icon">
            <Box :size="20" />
          </div>
          <div class="plugin-info">
            <div class="name">{{ plugin.extension.name }}</div>
            <div class="version">v{{ plugin.extension.version }}</div>
          </div>
        </div>

        <div v-if="installedPlugins.length === 0 && !isLoading" class="empty-list">
          <PackageOpen :size="32" />
          <p>No extensions installed</p>
        </div>
      </div>
    </div>

    <div class="marketplace-main">
      <div v-if="selectedPlugin" class="renderer-wrapper">
        <PluginRenderer :manifest="selectedPlugin" :plugin-id="selectedPlugin.extension.id" />
      </div>
      <div v-else class="welcome-marketplace">
        <ShoppingBag :size="48" class="mb-4" />
        <h2>StreamK8s Marketplace</h2>
        <p>Enhance your Kubernetes management experience with custom WASM-powered extensions.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.marketplace-container {
  display: grid;
  grid-template-columns: 260px 1fr;
  height: 100%;
  background-color: #030712;
  border-radius: 8px;
  overflow: hidden;
}

.marketplace-sidebar {
  background-color: #0f172a;
  border-right: 1px solid #1f2937;
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 1.5rem 1rem;
  border-bottom: 1px solid #1f2937;
}

.title-row {
  display: flex;
  align-items: center;
  margin-bottom: 1rem;
}

.title-row h3 { margin: 0; font-size: 1rem; color: #f3f4f6; }

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 8px;
  color: #6b7280;
}

.search-box input {
  width: 100%;
  background-color: #020617;
  border: 1px solid #1f2937;
  border-radius: 6px;
  padding: 6px 8px 6px 30px;
  color: white;
  font-size: 0.8rem;
  outline: none;
}

.plugin-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.plugin-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 4px;
}

.plugin-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.plugin-item.active {
  background-color: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.plugin-icon {
  width: 40px;
  height: 40px;
  background-color: #1e293b;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #3b82f6;
}

.plugin-info .name {
  font-size: 0.85rem;
  font-weight: 600;
  color: #e5e7eb;
}

.plugin-info .version {
  font-size: 0.7rem;
  color: #6b7280;
}

.marketplace-main {
  flex: 1;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
}

.renderer-wrapper {
  height: 100%;
}

.welcome-marketplace {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #4b5563;
  text-align: center;
}

.empty-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-top: 4rem;
  color: #374151;
}

.mr-2 { margin-right: 0.5rem; }
.mb-4 { margin-bottom: 1rem; }
</style>
