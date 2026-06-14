<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface ResourceRow {
  name: string;
  namespace: string;
  status?: string;
  [key: string]: any;
}

const props = defineProps<{ 
  rows: ResourceRow[],
  contextName: string,
  kind: string
}>();

const emit = defineEmits<{
  (e: 'select-resource', namespace: string, name: string, kind: string): void
}>();

const selectResource = (namespace: string, name: string) => {
  emit('select-resource', namespace, name, props.kind);
};

const searchQuery = ref('');
const selectedNames = ref<string[]>([]);

const filteredRows = computed(() => {
  if (!searchQuery.value) return props.rows;
  const query = searchQuery.value.toLowerCase();
  return props.rows.filter(row => 
    row.name.toLowerCase().includes(query) || 
    (row.namespace && row.namespace.toLowerCase().includes(query))
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
  return filteredRows.value.length > 0 && filteredRows.value.every(r => isSelected(r.name));
});

const toggleSelectAll = () => {
  const filteredNames = filteredRows.value.map(r => r.name);
  if (isAllSelected.value) {
    selectedNames.value = selectedNames.value.filter(name => !filteredNames.includes(name));
  } else {
    const newSelection = [...new Set([...selectedNames.value, ...filteredNames])];
    selectedNames.value = newSelection;
  }
};

const bulkScale = async (replicas: number) => {
  // Only applicable for certain kinds (Deployment, StatefulSet)
  if (!['Pod', 'Deployment', 'StatefulSet'].includes(props.kind)) {
     console.warn(`Scaling is not supported for ${props.kind}`);
     return;
  }

  const selectedRows = props.rows.filter(r => selectedNames.value.includes(r.name));
  const promises = selectedRows.map(r => 
    invoke('scale_workload', { 
      contextName: props.contextName,
      namespace: r.namespace, 
      name: r.name, 
      replicas 
    })
      .catch(err => console.error(`Failed to scale ${r.name}:`, err))
  );
  await Promise.all(promises);
  selectedNames.value = [];
};

const isScaleSupported = computed(() => ['Pod', 'Deployment', 'StatefulSet'].includes(props.kind));
</script>

<template>
  <div class="resource-table">
    <div class="list-header">
      <div class="header-left">
        <h3>{{ kind }}s ({{ filteredRows.length }})</h3>
        <span v-if="selectedNames.length > 0" class="selection-count">
          {{ selectedNames.length }} selected
        </span>
      </div>
      <input 
        v-model="searchQuery" 
        type="text" 
        :placeholder="`Filter ${kind.toLowerCase()}s...`" 
        class="search-input"
      />
    </div>

    <div v-if="selectedNames.length > 0 && isScaleSupported" class="bulk-action-bar">
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
            <th v-if="filteredRows.some(r => r.namespace)">Namespace</th>
            <th v-if="filteredRows.some(r => r.status)">Status</th>
          </tr>
        </thead>
        <tbody>
          <tr 
            v-for="row in filteredRows" 
            :key="row.name"
            :class="{ 'row-selected': isSelected(row.name) }"
            @click="toggleSelection(row.name)"
          >
            <td class="col-check" @click.stop>
              <input 
                type="checkbox" 
                :checked="isSelected(row.name)" 
                @change="toggleSelection(row.name)"
              />
            </td>
            <td class="resource-name-cell" @click.stop="selectResource(row.namespace, row.name)">
              {{ row.name }}
            </td>
            <td v-if="row.namespace">{{ row.namespace }}</td>
            <td v-if="row.status">
              <span :class="['status-badge', row.status.toLowerCase()]">
                {{ row.status }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.resource-table {
  background-color: var(--surface-card);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-dim);
  display: flex;
  flex-direction: column;
  height: 100%;
  box-sizing: border-box;
  position: relative;
  font-family: var(--font-ui);
}
.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-4);
}
.header-left {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}
.selection-count {
  font-size: 0.875rem;
  color: var(--accent-blue);
  background-color: rgba(59, 130, 246, 0.1);
  padding: 2px 8px;
  border-radius: 12px;
}
.search-input {
  background-color: var(--surface-dark);
  border: 1px solid var(--border-dim);
  color: white;
  padding: 0.5rem 1rem;
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  outline: none;
  transition: border-color 0.2s;
}
.search-input:focus {
  border-color: var(--accent-blue);
}
h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
}

.bulk-action-bar {
  background-color: var(--surface-dark);
  border: 1px solid var(--accent-blue);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-4);
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
  gap: var(--space-2);
}
.btn-bulk {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 1rem;
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  font-weight: 600;
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
  border: 1px solid var(--border-dim);
  border-radius: var(--radius-sm);
}
table {
  width: 100%;
  border-collapse: collapse;
}
th {
  position: sticky;
  top: 0;
  background-color: var(--surface-card);
  text-align: left;
  font-size: 0.75rem;
  font-weight: 700;
  color: #9ca3af;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--border-dim);
  z-index: 10;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
td {
  padding: var(--space-4);
  font-size: 0.875rem;
  border-top: 1px solid var(--border-dim);
}
.col-check {
  width: 48px;
  text-align: center;
}
.resource-name-cell {
  color: var(--accent-blue);
  font-weight: 600;
  cursor: pointer;
  font-family: var(--font-code);
}
.resource-name-cell:hover {
  text-decoration: underline;
}
tr {
  cursor: pointer;
  transition: all 0.2s;
}
tr:hover {
  background-color: rgba(255, 255, 255, 0.03);
  box-shadow: inset 0 0 12px var(--accent-blue-glow);
}
tr.row-selected {
  background-color: rgba(59, 130, 246, 0.08);
}
.status-badge {
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
}
/* Standard Pod statuses */
.running { background-color: rgba(6, 95, 70, 0.2); color: #34d399; border: 1px solid rgba(52, 211, 153, 0.2); }
.pending { background-color: rgba(146, 64, 14, 0.2); color: #fbbf24; border: 1px solid rgba(251, 191, 36, 0.2); }
.failed { background-color: rgba(153, 27, 27, 0.2); color: #f87171; border: 1px solid rgba(248, 113, 113, 0.2); }
/* Generic statuses */
.active, .ready { background-color: rgba(6, 95, 70, 0.2); color: #34d399; border: 1px solid rgba(52, 211, 153, 0.2); }
.inactive, .error { background-color: rgba(153, 27, 27, 0.2); color: #f87171; border: 1px solid rgba(248, 113, 113, 0.2); }
</style>
