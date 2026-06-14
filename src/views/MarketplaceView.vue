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
import Toast from '../components/Toast.vue';

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

// Task 3: Marketplace UI Polish
const installingPlugins = ref<Map<string, number>>(new Map());
const toast = ref({
  message: '',
  type: 'success' as 'success' | 'error',
  visible: false
});

const showToast = (message: string, type: 'success' | 'error' = 'success') => {
  toast.value = { message, type, visible: true };
};

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
  if (!plugin.url || installingPlugins.value.has(plugin.extension.id)) return;
  
  const pluginId = plugin.extension.id;
  installingPlugins.value.set(pluginId, 0);

  try {
    // Simulate progress updates
    const interval = setInterval(() => {
      const current = installingPlugins.value.get(pluginId) || 0;
      if (current < 90) {
        installingPlugins.value.set(pluginId, current + Math.floor(Math.random() * 15));
      }
    }, 400);

    await invoke('install_remote_plugin', { id: pluginId, url: plugin.url });
    
    clearInterval(interval);
    installingPlugins.value.set(pluginId, 100);
    
    // Smooth transition before refreshing
    setTimeout(async () => {
      await fetchPlugins();
      installingPlugins.value.delete(pluginId);
      showToast(`Successfully installed ${plugin.extension.name}`);
      selectedPluginId.value = pluginId;
    }, 500);

  } catch (e) {
    console.error('Failed to install plugin:', e);
    installingPlugins.value.delete(pluginId);
    showToast(`Failed to install ${plugin.extension.name}`, 'error');
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

          <div v-if="allPlugins.length > 0">
            <TransitionGroup name="list" tag="div" class="plugin-grid">
              <div 
                v-for="plugin in allPlugins" 
                :key="plugin.extension.id"
                class="plugin-card"
                :class="{ 'is-installing': installingPlugins.has(plugin.extension.id) }"
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
                  
                  <!-- Progress Bar for installation -->
                  <div v-if="installingPlugins.has(plugin.extension.id)" class="install-progress-container">
                    <div class="progress-info">
                      <span>Installing...</span>
                      <span>{{ installingPlugins.get(plugin.extension.id) }}%</span>
                    </div>
                    <div class="progress-bar-bg">
                      <div 
                        class="progress-bar-fill" 
                        :style="{ width: installingPlugins.get(plugin.extension.id) + '%' }"
                      ></div>
                    </div>
                  </div>

                  <div v-else class="card-footer">
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
            </TransitionGroup>
          </div>

          <div v-else-if="!isLoading" class="empty-state">
            <PackageOpen :size="48" />
            <p>No extensions matching your criteria</p>
          </div>
        </div>
      </div>
    </div>
    
    <Toast 
      v-model:visible="toast.visible" 
      :message="toast.message" 
      :type="toast.type" 
    />
  </div>
</template>

<style scoped>
.marketplace-container {
  display: grid;
  grid-template-columns: 240px 1fr;
  height: 100%;
  background-color: var(--surface-dark);
  font-family: var(--font-ui);
}

.marketplace-sidebar {
  background-color: var(--surface-dark);
  border-right: 1px solid var(--border-dim);
  padding: var(--space-6) var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-8);
}

.sidebar-section h4 {
  font-size: 0.7rem;
  font-weight: 900;
  color: #4b5563;
  margin-bottom: var(--space-3);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.cat-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  color: #9ca3af;
  cursor: pointer;
  transition: all 0.2s;
}

.cat-item:hover { color: #f3f4f6; background: rgba(255, 255, 255, 0.03); }
.cat-item.active { color: var(--accent-blue); background: rgba(59, 130, 246, 0.08); font-weight: 600; }

.side-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  background: transparent;
  border: 1px solid var(--border-dim);
  color: #d1d5db;
  padding: var(--space-2);
  border-radius: var(--radius-md);
  font-size: 0.75rem;
  cursor: pointer;
  margin-bottom: var(--space-2);
  transition: all 0.2s;
}

.side-btn:hover { border-color: var(--accent-blue); color: white; background: rgba(59, 130, 246, 0.05); }

.marketplace-main {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.marketplace-header {
  height: 72px;
  display: flex;
  align-items: center;
  padding: 0 var(--space-8);
  gap: var(--space-8);
  border-bottom: 1px solid var(--border-dim);
  background-color: var(--surface-dark);
}

.search-container {
  flex: 1;
  position: relative;
}

.search-icon {
  position: absolute;
  left: 16px;
  top: 50%;
  transform: translateY(-50%);
  color: #4b5563;
}

.search-container input {
  width: 100%;
  background: var(--surface-card);
  border: 1px solid var(--border-dim);
  border-radius: var(--radius-md);
  padding: 12px 12px 12px 48px;
  color: white;
  font-size: 0.9rem;
  outline: none;
  transition: all 0.2s;
}

.search-container input:focus { border-color: var(--accent-blue); box-shadow: 0 0 0 2px var(--accent-blue-glow); }

.content-area {
  flex: 1;
  padding: var(--space-8);
  overflow-y: auto;
}

.grid-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: var(--space-6);
}

.grid-header h3 { font-size: 1.5rem; margin: 0; font-weight: 700; }
.grid-header .count { font-size: 0.875rem; color: #4b5563; }

.plugin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: var(--space-6);
}

.plugin-card {
  background: var(--surface-card);
  border: 1px solid var(--border-dim);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  display: flex;
  gap: var(--space-4);
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.plugin-card:hover { 
  border-color: var(--accent-blue); 
  transform: translateY(-4px);
  box-shadow: 0 10px 20px -10px rgba(0,0,0,0.5), 0 0 15px var(--accent-blue-glow);
}
.plugin-card.is-installing { border-color: var(--accent-blue); background: rgba(59, 130, 246, 0.05); cursor: default; transform: none; box-shadow: none; }

.install-progress-container {
  margin-top: var(--space-2);
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--accent-blue);
  margin-bottom: var(--space-1);
  font-weight: 700;
}

.progress-bar-bg {
  height: 6px;
  background: var(--surface-dark);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: var(--accent-blue);
  box-shadow: 0 0 10px var(--accent-blue-glow);
  transition: width 0.3s ease;
}

.card-icon {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.card-icon.online { background: rgba(59, 130, 246, 0.1); color: var(--accent-blue); }
.card-icon.local { background: rgba(16, 185, 129, 0.1); color: #10b981; }

.card-content { flex: 1; min-width: 0; }
.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-2); }
.card-header h4 { margin: 0; font-size: 1.125rem; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: #f3f4f6; }

.badge { font-size: 0.65rem; font-weight: 800; padding: 2px 8px; border-radius: var(--radius-sm); text-transform: uppercase; letter-spacing: 0.05em; }
.badge-online { background: rgba(59, 130, 246, 0.2); color: #60a5fa; }
.badge-local { background: rgba(16, 185, 129, 0.2); color: #34d399; }

.desc { font-size: 0.875rem; color: #9ca3af; line-height: 1.6; margin-bottom: var(--space-4); display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }

.card-footer { display: flex; justify-content: space-between; align-items: center; }
.author { font-size: 0.75rem; color: #4b5563; font-style: italic; }

.btn-install {
  background: var(--accent-blue);
  color: white;
  border: none;
  padding: 6px 16px;
  border-radius: var(--radius-md);
  font-size: 0.8rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-install:hover {
  filter: brightness(1.1);
  box-shadow: 0 0 12px var(--accent-blue-glow);
}

.btn-installed {
  background: var(--surface-dark);
  color: #6b7280;
  border: 1px solid var(--border-dim);
  padding: 6px 16px;
  border-radius: var(--radius-md);
  font-size: 0.8rem;
  font-weight: 600;
}

.plugin-view-mode { height: 100%; display: flex; flex-direction: column; }
.view-header { display: flex; align-items: center; gap: var(--space-6); margin-bottom: var(--space-8); }
.back-btn { background: none; border: none; color: var(--accent-blue); cursor: pointer; font-size: 0.9rem; font-weight: 700; }
.plugin-title { display: flex; align-items: center; gap: var(--space-4); }
.plugin-title h2 { margin: 0; font-size: 2rem; font-weight: 800; }

.empty-state { text-align: center; margin-top: 8rem; color: #1e293b; }
.empty-state p { margin-top: var(--space-4); color: #4b5563; font-size: 1.125rem; }

.spin { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.mr-1 { margin-right: var(--space-1); }

/* List Transitions */
.list-enter-active,
.list-leave-active {
  transition: all 0.4s ease;
}
.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: scale(0.9);
}
.list-move {
  transition: transform 0.4s ease;
}
</style>
