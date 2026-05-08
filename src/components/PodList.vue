<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface Pod {
  name: string;
  namespace: string;
  status: string;
}
const props = defineProps<{ pods: Pod[] }>();

const emit = defineEmits<{
  (e: 'select-pod', namespace: string, name: string): void
}>();

const selectPod = (namespace: string, name: string) => {
  emit('select-pod', namespace, name);
};

const searchQuery = ref('');
const selectedNames = ref<string[]>([]);

const filteredPods = computed(() => {
  if (!searchQuery.value) return props.pods;
  const query = searchQuery.value.toLowerCase();
  return props.pods.filter(pod => 
    pod.name.toLowerCase().includes(query) || 
    pod.namespace.toLowerCase().includes(query)
  );
});

const isSelected = (name: string) => selectedNames.value.includes(name);

const toggleSelection = (name: string) => {
  const index = selectedNames.value.indexOf(name);
  if (index > -1) {
    selectedNames.value.splice(index, 1);
  } else {
    selectedNames.value.push(name);
  }
};

const isAllSelected = computed(() => {
  return filteredPods.value.length > 0 && filteredPods.value.every(p => isSelected(p.name));
});

const toggleSelectAll = () => {
  const filteredNames = filteredPods.value.map(p => p.name);
  if (isAllSelected.value) {
    selectedNames.value = selectedNames.value.filter(name => !filteredNames.includes(name));
  } else {
    const newSelection = [...new Set([...selectedNames.value, ...filteredNames])];
    selectedNames.value = newSelection;
  }
};

const bulkScale = async (replicas: number) => {
  const selectedPods = props.pods.filter(p => selectedNames.value.includes(p.name));
  const promises = selectedPods.map(p => 
    invoke('scale_workload', { namespace: p.namespace, name: p.name, replicas })
      .catch(err => console.error(`Failed to scale ${p.name}:`, err))
  );
  await Promise.all(promises);
  // Optional: clear selection after action
  selectedNames.value = [];
};
</script>

<template>
  <div class="pod-list">
    <div class="list-header">
      <div class="header-left">
        <h3>Active Workloads ({{ filteredPods.length }})</h3>
        <span v-if="selectedNames.length > 0" class="selection-count">
          {{ selectedNames.length }} selected
        </span>
      </div>
      <input 
        v-model="searchQuery" 
        type="text" 
        placeholder="Filter pods..." 
        class="search-input"
      />
    </div>

    <div v-if="selectedNames.length > 0" class="bulk-action-bar">
      <div class="bulk-buttons">
        <button @click="bulkScale(0)" class="btn-bulk suspend">
          <span class="icon">⏸</span> Suspend
        </button>
        <button @click="bulkScale(1)" class="btn-bulk resume">
          <span class="icon">▶</span> Resume
        </button>
      </div>
      <button @click="selectedNames = []" class="btn-clear">Clear Selection</button>
    </div>

    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th class="col-check">
              <input 
                type="checkbox" 
                :checked="isAllSelected" 
                @change="toggleSelectAll"
              />
            </th>
            <th>Name</th>
            <th>Namespace</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          <tr 
            v-for="pod in filteredPods" 
            :key="pod.name"
            :class="{ 'row-selected': isSelected(pod.name) }"
            @click="toggleSelection(pod.name)"
          >
            <td class="col-check" @click.stop>
              <input 
                type="checkbox" 
                :checked="isSelected(pod.name)" 
                @change="toggleSelection(pod.name)"
              />
            </td>
            <td class="pod-name-cell" @click.stop="selectPod(pod.namespace, pod.name)">
              {{ pod.name }}
            </td>
            <td>{{ pod.namespace }}</td>
            <td>
              <span :class="['status-badge', pod.status.toLowerCase()]">
                {{ pod.status }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.pod-list {
  background-color: #1f2937;
  padding: 1rem;
  border-radius: 8px;
  border: 1px solid #374151;
  display: flex;
  flex-direction: column;
  height: 100%;
  box-sizing: border-box;
  position: relative;
}
.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 1rem;
}
.selection-count {
  font-size: 0.875rem;
  color: #3b82f6;
  background-color: rgba(59, 130, 246, 0.1);
  padding: 2px 8px;
  border-radius: 12px;
}
.search-input {
  background-color: #374151;
  border: 1px solid #4b5563;
  color: white;
  padding: 0.4rem 0.8rem;
  border-radius: 4px;
  font-size: 0.875rem;
  outline: none;
}
.search-input:focus {
  border-color: #3b82f6;
}
h3 {
  margin: 0;
  font-size: 1.125rem;
}

.bulk-action-bar {
  background-color: #111827;
  border: 1px solid #3b82f6;
  padding: 0.75rem 1rem;
  border-radius: 6px;
  margin-bottom: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  animation: slideDown 0.2s ease-out;
}
@keyframes slideDown {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.bulk-buttons {
  display: flex;
  gap: 0.5rem;
}
.btn-bulk {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.4rem 0.8rem;
  border-radius: 4px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  border: none;
  color: white;
  transition: background-color 0.2s;
}
.btn-bulk.suspend { background-color: #92400e; }
.btn-bulk.suspend:hover { background-color: #b45309; }
.btn-bulk.resume { background-color: #065f46; }
.btn-bulk.resume:hover { background-color: #047857; }

.btn-clear {
  background: none;
  border: none;
  color: #9ca3af;
  font-size: 0.75rem;
  cursor: pointer;
  text-decoration: underline;
}
.btn-clear:hover { color: #d1d5db; }

.table-container {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #374151;
  border-radius: 4px;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th {
  position: sticky;
  top: 0;
  background-color: #1f2937;
  text-align: left;
  font-size: 0.75rem;
  color: #9ca3af;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #374151;
  z-index: 10;
}
td {
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  border-top: 1px solid #374151;
}
.col-check {
  width: 40px;
  text-align: center;
}
.pod-name-cell {
  color: #3b82f6;
  font-weight: 500;
  cursor: pointer;
}
.pod-name-cell:hover {
  text-decoration: underline;
}
tr {
  cursor: pointer;
  transition: background-color 0.1s;
}
tr:hover {
  background-color: #2d3748;
}
tr.row-selected {
  background-color: rgba(59, 130, 246, 0.1);
}
.status-badge {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.75rem;
}
.running { background-color: #065f46; color: #34d399; }
.pending { background-color: #92400e; color: #fbbf24; }
.failed { background-color: #991b1b; color: #f87171; }
</style>
