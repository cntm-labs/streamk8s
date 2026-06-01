<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { 
  Box, Search, PackageOpen, RefreshCw, 
  FolderOpen, UploadCloud, Globe, Cpu, ShieldCheck, Zap,
  Download, CheckCircle2
} from 'lucide-vue-next';
import PluginRenderer from '../components/PluginRenderer.vue';

interface ExtensionInfo {
  id: string;
  name: string;
  description: string;
  version: string;
  author?: string;
  category?: string;
}

interface PluginManifest {
  extension: ExtensionInfo;
  ui: any;
  source: 'local' | 'online';
  url?: string;
}

const installedPlugins = ref<PluginManifest[]>([]);
const remotePlugins = ref<any[]>([]);
const selectedPluginId = ref<string | null>(null);
const isLoading = ref(true);
const searchQuerry = ref('');
const activeCategory = ref('All');

const fetchPlugins = async () => {
  isLoading.value = true;
  try {
    const [installed, remote] = await Promise.all([
      invoke<any[]>('get_installed_plugins'),
      invoke<any[]>('get_remote_registry')
    ]);

    installedPlugins.value = installed.map(p => ({ ...p, source: 'local' }));
    remotePlugins.value = remote;
  } catch (e) {
    console.error('Failed to fetch plugins:', e);
  } finally {
    isLoading.value = false;
  }
};

const allPlugins = computed(() => {
  const list: PluginManifest[] = [];
  
  // Add installed first
  installedPlugins.value.forEach(p => list.push(p));
  
  // Add remote if not already installed
  remotePlugins.value.forEach(rp => {
    if (!installedPlugins.value.some(p => p.extension.id === rp.id)) {
      list.push({
        extension: {
          id: rp.id,
          name: rp.name,
          description: rp.description,
          version: rp.version,
          author: rp.author,
          category: rp.category
        },
        ui: null,
        source: 'online',
        url: rp.url
      });
    }
  });

  return list.filter(p => {
    const matchesSearch = p.extension.name.toLowerCase().includes(searchQuerry.value.toLowerCase()) ||
                          p.extension.description.toLowerCase().includes(searchQuerry.value.toLowerCase());
    const matchesCategory = activeCategory.value === 'All' || p.extension.category === activeCategory.value;
    return matchesSearch && matchesCategory;
  });
});

const selectedPlugin = computed(() => {
  return allPlugins.value.find(p => p.extension.id === selectedPluginId.value) || null;
});

const installRemotePlugin = async (plugin: PluginManifest) => {
  if (!plugin.url) return;
  try {
    isLoading.value = true;
    await invoke('install_remote_plugin', { id: plugin.extension.id, url: plugin.url });
    await fetchPlugins();
    selectedPluginId.value = plugin.extension.id;
  } catch (e) {
    console.error('Failed to install plugin:', e);
  } finally {
    isLoading.value = false;
  }
};

const importPlugin = async () => {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: 'Select Plugin Directory'
    });

    if (selected) {
      await invoke('install_plugin', { sourcePath: selected });
      await fetchPlugins();
    }
  } catch (e) {
    console.error('Failed to import plugin:', e);
  }
};

const openPluginFolder = async () => {
  try {
    // In a real env we should use an API to get the path
    await openPath('/home/mrbt/.config/streamk8s/plugins');
  } catch (e) {
    console.error('Failed to open folder:', e);
  }
};

onMounted(fetchPlugins);
</script>

<template>
  <div class="marketplace-container">
    <div class="marketplace-sidebar">
      <div class="sidebar-section">
        <h4>CATEGORIES</h4>
        <div 
          class="cat-item" 
          :class="{ active: activeCategory === 'All' }"
          @click="activeCategory = 'All'"
        >
          <Box :size="16" /> All Extensions
        </div>
        <div 
          class="cat-item" 
          :class="{ active: activeCategory === 'Observability' }"
          @click="activeCategory = 'Observability'"
        >
          <Zap :size="16" /> Observability
        </div>
        <div 
          class="cat-item" 
          :class="{ active: activeCategory === 'Security' }"
          @click="activeCategory = 'Security'"
        >
          <ShieldCheck :size="16" /> Security
        </div>
        <div 
          class="cat-item" 
          :class="{ active: activeCategory === 'Automation' }"
          @click="activeCategory = 'Automation'"
        >
          <Cpu :size="16" /> Automation
        </div>
      </div>

      <div class="sidebar-section dev-tools">
        <h4>DEVELOPER</h4>
        <button class="side-btn" @click="importPlugin">
          <UploadCloud :size="14" /> Import Local
        </button>
        <button class="side-btn" @click="openPluginFolder">
          <FolderOpen :size="14" /> Plugin Folder
        </button>
      </div>
    </div>

    <div class="marketplace-main">
      <div class="marketplace-header">
        <div class="search-container">
          <Search :size="16" class="search-icon" />
          <input 
            v-model="searchQuerry"
            type="text" 
            placeholder="Search extensions, utilities, and analyzers..." 
          />
        </div>
        <div class="header-actions">
          <button class="icon-btn" @click="fetchPlugins" :disabled="isLoading">
            <RefreshCw :size="18" :class="{ 'spin': isLoading }" />
          </button>
        </div>
      </div>

      <div class="content-area">
        <div v-if="selectedPlugin && selectedPlugin.source === 'local'" class="plugin-view-mode">
          <div class="view-header">
            <button class="back-btn" @click="selectedPluginId = null">← Back to Catalog</button>
            <div class="plugin-title">
              <h2>{{ selectedPlugin.extension.name }}</h2>
              <span class="badge badge-local">Local</span>
            </div>
          </div>
          <PluginRenderer :manifest="selectedPlugin" :plugin-id="selectedPlugin.extension.id" />
        </div>

        <div v-else class="catalog-grid-mode">
          <div class="grid-header">
            <h3>{{ activeCategory }} Extensions</h3>
            <span class="count">{{ allPlugins.length }} found</span>
          </div>

          <div v-if="allPlugins.length > 0" class="plugin-grid">
            <div 
              v-for="plugin in allPlugins" 
              :key="plugin.extension.id"
              class="plugin-card"
              @click="plugin.source === 'local' ? selectedPluginId = plugin.extension.id : null"
            >
              <div class="card-icon" :class="plugin.source">
                <Globe v-if="plugin.source === 'online'" :size="24" />
                <Box v-else :size="24" />
              </div>
              <div class="card-content">
                <div class="card-header">
                  <h4>{{ plugin.extension.name }}</h4>
                  <span class="badge" :class="'badge-' + plugin.source">{{ plugin.source }}</span>
                </div>
                <p class="desc">{{ plugin.extension.description }}</p>
                <div class="card-footer">
                  <span class="author">by {{ plugin.extension.author || 'Anonymous' }}</span>
                  <div class="actions">
                    <button 
                      v-if="plugin.source === 'online'" 
                      class="btn-install"
                      @click.stop="installRemotePlugin(plugin)"
                    >
                      <Download :size="14" class="mr-1" /> Install
                    </button>
                    <button v-else class="btn-installed" disabled>
                      <CheckCircle2 :size="14" class="mr-1" /> Installed
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div v-else-if="!isLoading" class="empty-state">
            <PackageOpen :size="48" />
            <p>No extensions matching your criteria</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.marketplace-container {
  display: grid;
  grid-template-columns: 220px 1fr;
  height: 100%;
  background-color: #030712;
}

.marketplace-sidebar {
  background-color: #030712;
  border-right: 1px solid #1f2937;
  padding: 1.5rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.sidebar-section h4 {
  font-size: 0.65rem;
  font-weight: 900;
  color: #4b5563;
  margin-bottom: 0.75rem;
  letter-spacing: 0.05em;
}

.cat-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.85rem;
  color: #9ca3af;
  cursor: pointer;
  transition: all 0.2s;
}

.cat-item:hover { color: #f3f4f6; background: #111827; }
.cat-item.active { color: #3b82f6; background: rgba(59, 130, 246, 0.1); font-weight: 600; }

.side-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  background: transparent;
  border: 1px solid #1f2937;
  color: #d1d5db;
  padding: 8px;
  border-radius: 6px;
  font-size: 0.75rem;
  cursor: pointer;
  margin-bottom: 8px;
}

.side-btn:hover { border-color: #3b82f6; color: white; }

.marketplace-main {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.marketplace-header {
  height: 60px;
  display: flex;
  align-items: center;
  padding: 0 2rem;
  gap: 2rem;
  border-bottom: 1px solid #1f2937;
  background-color: #030712;
}

.search-container {
  flex: 1;
  position: relative;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: #4b5563;
}

.search-container input {
  width: 100%;
  background: #0f172a;
  border: 1px solid #1f2937;
  border-radius: 8px;
  padding: 10px 10px 10px 40px;
  color: white;
  font-size: 0.9rem;
  outline: none;
}

.search-container input:focus { border-color: #3b82f6; }

.content-area {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}

.grid-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 1.5rem;
}

.grid-header h3 { font-size: 1.25rem; margin: 0; }
.grid-header .count { font-size: 0.8rem; color: #4b5563; }

.plugin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1.5rem;
}

.plugin-card {
  background: #111827;
  border: 1px solid #1f2937;
  border-radius: 12px;
  padding: 1.25rem;
  display: flex;
  gap: 1.25rem;
  cursor: pointer;
  transition: all 0.2s;
}

.plugin-card:hover { border-color: #3b82f6; transform: translateY(-2px); }

.card-icon {
  width: 54px;
  height: 54px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.card-icon.online { background: rgba(59, 130, 246, 0.1); color: #3b82f6; }
.card-icon.local { background: rgba(16, 185, 129, 0.1); color: #10b981; }

.card-content { flex: 1; min-width: 0; }
.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem; }
.card-header h4 { margin: 0; font-size: 1rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

.badge { font-size: 0.6rem; font-weight: 800; padding: 2px 6px; border-radius: 4px; text-transform: uppercase; }
.badge-online { background: rgba(59, 130, 246, 0.2); color: #60a5fa; }
.badge-local { background: rgba(16, 185, 129, 0.2); color: #34d399; }

.desc { font-size: 0.8rem; color: #9ca3af; line-height: 1.5; margin-bottom: 1rem; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }

.card-footer { display: flex; justify-content: space-between; align-items: center; }
.author { font-size: 0.7rem; color: #4b5563; font-style: italic; }

.btn-install {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
}

.btn-installed {
  background: #1f2937;
  color: #6b7280;
  border: 1px solid #374151;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 0.75rem;
}

.plugin-view-mode { height: 100%; display: flex; flex-direction: column; }
.view-header { display: flex; align-items: center; gap: 2rem; margin-bottom: 2rem; }
.back-btn { background: none; border: none; color: #3b82f6; cursor: pointer; font-size: 0.9rem; font-weight: 600; }
.plugin-title { display: flex; align-items: center; gap: 1rem; }
.plugin-title h2 { margin: 0; }

.empty-state { text-align: center; margin-top: 5rem; color: #1e293b; }
.empty-state p { margin-top: 1rem; color: #4b5563; }

.spin { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.mr-1 { margin-right: 0.25rem; }
</style>
