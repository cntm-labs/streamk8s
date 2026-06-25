<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Loader2 } from 'lucide-vue-next';

const props = defineProps<{
  visible: boolean;
  resource: { contextName: string; namespace: string; name: string; kind: string } | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'open-terminal', containerName: string): void;
  (e: 'open-logs', containerName: string): void;
  (e: 'edit-yaml', resource: any): void;
  (e: 'deleted'): void;
}>();

const detailData = ref<any>(null);
const events = ref<any[]>([]);
const containerNames = ref<string[]>([]);
const selectedContainer = ref('');
const isLoading = ref(false);
const isLoadingEvents = ref(false);
const isDeleting = ref(false);

const fetchDetails = async () => {
  const res = props.resource;
  if (!res) return;
  isLoading.value = true;
  detailData.value = null;
  containerNames.value = [];
  selectedContainer.value = '';

  try {
    const kindSingular = res.kind.replace(/s$/, '');
    const dataText = await invoke<string>('get_k8s_resource_details', {
      contextName: res.contextName,
      namespace: res.namespace,
      name: res.name,
      kind: kindSingular,
    });
    
    // Parse response
    const parsed = JSON.parse(dataText);
    detailData.value = parsed;

    if (res.kind === 'Pods' && parsed.spec?.containers) {
      containerNames.value = parsed.spec.containers.map((c: any) => c.name);
      if (containerNames.value.length > 0) {
        selectedContainer.value = containerNames.value[0];
      }
    }
  } catch (e) {
    console.error('Failed to fetch resource details:', e);
  } finally {
    isLoading.value = false;
  }
};

const fetchEvents = async () => {
  const res = props.resource;
  if (!res || res.kind !== 'Pods') return;
  isLoadingEvents.value = true;
  events.value = [];

  try {
    const eventsText = await invoke<string>('get_pod_events', {
      contextName: res.contextName,
      namespace: res.namespace,
      podName: res.name,
    });
    events.value = JSON.parse(eventsText);
  } catch (e) {
    console.error('Failed to fetch pod events:', e);
  } finally {
    isLoadingEvents.value = false;
  }
};

const deleteResource = async () => {
  const res = props.resource;
  if (!res) return;
  
  const confirmed = window.confirm(`Are you sure you want to delete ${res.kind.replace(/s$/, '')} "${res.name}"?`);
  if (!confirmed) return;

  isDeleting.value = true;
  try {
    const kindSingular = res.kind.replace(/s$/, '');
    await invoke('delete_k8s_resource', {
      contextName: res.contextName,
      namespace: res.namespace,
      name: res.name,
      kind: kindSingular,
    });
    emit('deleted');
  } catch (e) {
    alert(`Failed to delete resource: ${e}`);
  } finally {
    isDeleting.value = false;
  }
};

watch(() => props.resource, () => {
  if (props.visible && props.resource) {
    fetchDetails();
    fetchEvents();
  }
}, { deep: true });

watch(() => props.visible, (newVal) => {
  if (newVal && props.resource) {
    fetchDetails();
    fetchEvents();
  }
});
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click.self="emit('close')">
    <div class="resource-modal" role="dialog" aria-modal="true">
      <div v-if="resource" class="modal-container">
        <!-- Header -->
        <header class="modal-header">
          <div class="header-main">
            <span class="resource-kind-badge">{{ resource.kind.replace(/s$/, '') }} Detail</span>
            <h2 class="modal-title" :title="resource.name">{{ resource.name }}</h2>
          </div>
          <button class="btn-close" @click="emit('close')" aria-label="Close dialog">✕</button>
        </header>

        <!-- Quick Action Tools Bar -->
        <section class="quick-actions-bar">
          <button 
            v-if="resource.kind === 'Pods'"
            @click="emit('open-terminal', selectedContainer)" 
            class="action-btn terminal-btn" 
            title="Open Terminal Session"
          >
            💻 Terminal
          </button>
          <button 
            v-if="resource.kind === 'Pods'"
            @click="emit('open-logs', selectedContainer)" 
            class="action-btn logs-btn" 
            title="Stream Container Logs"
          >
            📝 Logs
          </button>
          <button 
            @click="emit('edit-yaml', resource)" 
            class="action-btn yaml-btn" 
            title="Edit YAML Manifest"
          >
            ✏️ Edit YAML
          </button>
          <button 
            @click="deleteResource" 
            class="action-btn delete-btn" 
            :disabled="isDeleting"
            title="Delete Resource"
          >
            🗑️ Delete
          </button>
        </section>

        <!-- Loader -->
        <div v-if="isLoading" class="modal-loader">
          <Loader2 class="animate-spin" :size="32" />
          <p>Loading details...</p>
        </div>

        <!-- Scrollable Content -->
        <div v-else class="modal-content">
          <!-- Metadata Info -->
          <div class="info-group">
            <h3>Overview</h3>
            <table class="details-table">
              <tbody>
                <tr>
                  <td class="label-col">Name</td>
                  <td class="value-col bold-value">{{ resource.name }}</td>
                </tr>
                <tr>
                  <td class="label-col">Namespace</td>
                  <td class="value-col">{{ resource.namespace }}</td>
                </tr>
                <tr v-if="detailData?.metadata?.creationTimestamp">
                  <td class="label-col">Age</td>
                  <td class="value-col">{{ new Date(detailData.metadata.creationTimestamp).toLocaleString() }}</td>
                </tr>
                <tr v-if="detailData?.status?.phase">
                  <td class="label-col">Status</td>
                  <td class="value-col">
                    <span :class="['status-badge', detailData.status.phase.toLowerCase()]">
                      {{ detailData.status.phase }}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Labels -->
          <div class="info-group" v-if="detailData?.metadata?.labels">
            <h3>Labels</h3>
            <div class="tags-container">
              <span v-for="(val, key) in detailData.metadata.labels" :key="key" class="tag-pill">
                {{ key }}: {{ val }}
              </span>
            </div>
          </div>

          <!-- Container Dropdown inside Detail Panel -->
          <div class="info-group" v-if="resource.kind === 'Pods' && containerNames.length > 0">
            <h3>Containers Details</h3>
            <div class="container-selection-block">
              <label for="modal-container-select" class="container-select-label">Active Container:</label>
              <select id="modal-container-select" v-model="selectedContainer" class="container-select-dropdown">
                <option v-for="c in containerNames" :key="c" :value="c">{{ c }}</option>
              </select>
            </div>
            
            <!-- Containers Status Lists -->
            <div class="containers-status-list">
              <div v-for="c in detailData?.spec?.containers" :key="c.name" class="container-status-row">
                <span class="bullet-dot"></span>
                <strong class="c-name">{{ c.name }}</strong>
                <span class="c-image" :title="c.image">{{ c.image.substring(c.image.lastIndexOf('/') + 1) }}</span>
              </div>
            </div>
          </div>

          <!-- Events -->
          <div class="info-group" v-if="resource.kind === 'Pods'">
            <h3>Recent Events</h3>
            <div v-if="isLoadingEvents" class="events-loader">Loading events...</div>
            <div v-else-if="events.length === 0" class="empty-events">No events found.</div>
            <div v-else class="modal-events-list">
              <div v-for="ev in events" :key="ev.metadata.uid" class="event-item-card">
                <div class="event-item-header">
                  <span :class="['event-badge', ev.type.toLowerCase()]">{{ ev.type }}</span>
                  <span class="event-reason">{{ ev.reason }}</span>
                </div>
                <p class="event-message">{{ ev.message }}</p>
                <div class="event-time">{{ new Date(ev.lastTimestamp || ev.eventTime).toLocaleTimeString() }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Modal overlay layout */
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
  z-index: 1000;
  display: flex;
  justify-content: center;
  align-items: center;
  animation: fadeIn 0.2s ease-out forwards;
}

/* Modal box floating in center */
.resource-modal {
  width: 760px;
  max-width: 90vw;
  height: 620px;
  max-height: 80vh;
  background-color: rgba(13, 14, 18, 0.95);
  border: 1px solid #1e293b;
  border-radius: 12px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
  z-index: 1001;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: scaleUp 0.3s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
}

.modal-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
}

/* Header */
.modal-header {
  padding: 16px 24px;
  background-color: #111827;
  border-bottom: 1px solid #1e293b;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.header-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-width: 85%;
}
.resource-kind-badge {
  font-size: 0.65rem;
  text-transform: uppercase;
  color: #3b82f6;
  font-weight: bold;
  letter-spacing: 0.05em;
}
.modal-title {
  margin: 0;
  font-size: 1.25rem;
  color: #f3f4f6;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.btn-close {
  background: none;
  border: none;
  color: #9ca3af;
  font-size: 1.25rem;
  cursor: pointer;
  padding: 4px;
  transition: all 0.2s;
}
.btn-close:hover {
  color: #f87171;
}

/* Quick Actions Bar */
.quick-actions-bar {
  padding: 12px 24px;
  background-color: #090b0f;
  border-bottom: 1px solid #1e293b;
  display: flex;
  gap: 10px;
  overflow-x: auto;
}
.action-btn {
  background-color: #1e293b;
  border: 1px solid #334155;
  color: #cbd5e1;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 6px 14px;
  border-radius: 4px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}
.action-btn:hover {
  color: white;
  border-color: #475569;
}
.action-btn.terminal-btn:hover {
  background-color: rgba(59, 130, 246, 0.1);
  border-color: #3b82f6;
  color: #60a5fa;
}
.action-btn.logs-btn:hover {
  background-color: rgba(16, 185, 129, 0.1);
  border-color: #10b981;
  color: #34d399;
}
.action-btn.yaml-btn:hover {
  background-color: rgba(168, 85, 247, 0.1);
  border-color: #a855f7;
  color: #c084fc;
}
.action-btn.delete-btn {
  background-color: #450a0a;
  border-color: #7f1d1d;
  color: #fca5a5;
  margin-left: auto;
}
.action-btn.delete-btn:hover:not(:disabled) {
  background-color: #991b1b;
  border-color: #ef4444;
  color: white;
}
.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Loaders and Empty States */
.modal-loader {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  color: #3b82f6;
  font-size: 0.85rem;
  gap: 12px;
}
.animate-spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Content Layout */
.modal-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}
.info-group h3 {
  margin-top: 0;
  margin-bottom: 12px;
  font-size: 0.85rem;
  text-transform: uppercase;
  color: #64748b;
  letter-spacing: 0.05em;
  border-bottom: 1px solid #1e293b;
  padding-bottom: 6px;
}

/* Details Table */
.details-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8rem;
}
.details-table td {
  padding: 8px 0;
}
.label-col {
  color: #94a3b8;
  width: 25%;
  font-weight: 500;
}
.value-col {
  color: #e2e8f0;
}
.bold-value {
  font-weight: 600;
  font-family: var(--font-code);
}

/* Status badge */
.status-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: bold;
}
.status-badge.running { background-color: rgba(16, 185, 129, 0.15); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.3); }
.status-badge.pending { background-color: rgba(245, 158, 11, 0.15); color: #f59e0b; border: 1px solid rgba(245, 158, 11, 0.3); }
.status-badge.failed { background-color: rgba(239, 68, 68, 0.15); color: #ef4444; border: 1px solid rgba(239, 68, 68, 0.3); }

/* Tags Labels */
.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.tag-pill {
  background-color: #1e293b;
  border: 1px solid #334155;
  color: #94a3b8;
  font-size: 0.7rem;
  padding: 2px 8px;
  border-radius: 4px;
}

/* Container Selector block */
.container-selection-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}
.container-select-label {
  font-size: 0.75rem;
  color: #94a3b8;
}
.container-select-dropdown {
  background-color: #1e293b;
  border: 1px solid #334155;
  color: #e2e8f0;
  font-size: 0.8rem;
  padding: 6px 10px;
  border-radius: 4px;
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s;
}
.container-select-dropdown:focus {
  border-color: #3b82f6;
}

.containers-status-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background-color: #090b0f;
  padding: 10px;
  border-radius: 4px;
  border: 1px solid #1e293b;
}
.container-status-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
}
.bullet-dot {
  width: 6px;
  height: 6px;
  background-color: #10b981;
  border-radius: 50%;
}
.c-name {
  color: #cbd5e1;
}
.c-image {
  color: #64748b;
  font-family: var(--font-code);
  font-size: 0.7rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 60%;
  margin-left: auto;
}

/* Events */
.events-loader, .empty-events {
  font-size: 0.75rem;
  color: #64748b;
  font-style: italic;
  padding: 8px 0;
}
.modal-events-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.event-item-card {
  background-color: #111827;
  border: 1px solid #1e293b;
  border-radius: 6px;
  padding: 8px 12px;
}
.event-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}
.event-badge {
  font-size: 0.6rem;
  font-weight: bold;
  padding: 1px 6px;
  border-radius: 4px;
}
.event-badge.warning { background-color: rgba(245, 158, 11, 0.15); color: #f59e0b; border: 1px solid rgba(245, 158, 11, 0.3); }
.event-badge.normal { background-color: rgba(16, 185, 129, 0.15); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.3); }
.event-reason {
  font-size: 0.75rem;
  color: #e2e8f0;
  font-weight: 600;
}
.event-message {
  margin: 4px 0;
  font-size: 0.75rem;
  color: #94a3b8;
  line-height: 1.4;
}
.event-time {
  font-size: 0.65rem;
  color: #64748b;
  text-align: right;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes scaleUp {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
