<script setup lang="ts">
import { ref, computed } from 'vue';
import PortForwardModal from './PortForwardModal.vue';

const props = defineProps<{ 
  rows: any[],
  contextName: string,
  kind: string
}>();

const emit = defineEmits<{
  (e: 'select-resource', namespace: string, name: string, kind: string): void;
  (e: 'edit-yaml', namespace: string, name: string, kind: string): void;
}>();

const selectResource = (namespace: string, name: string) => {
  emit('select-resource', namespace || '', name, props.kind);
};

const showPfModal = ref(false);
const pfTarget = ref({ name: '', namespace: '' });

const openPfModal = (namespace: string, name: string) => {
  pfTarget.value = { namespace, name };
  showPfModal.value = true;
};

const searchQuery = ref('');

const formattedRows = computed(() => {
  return props.rows.map(r => {
    return {
      _raw: r,
      name: r.metadata?.name || 'Unknown',
      namespace: r.metadata?.namespace || '',
      creationTimestamp: r.metadata?.creationTimestamp ? new Date(r.metadata.creationTimestamp).toLocaleString() : '',
      status: r.status?.phase || r.status?.conditions?.[0]?.type || ''
    };
  });
});

const filteredRows = computed(() => {
  if (!searchQuery.value) return formattedRows.value;
  const query = searchQuery.value.toLowerCase();
  return formattedRows.value.filter(row => 
    row.name.toLowerCase().includes(query) || 
    (row.namespace && row.namespace.toLowerCase().includes(query))
  );
});

// Infer dynamic columns from spec if possible
const dynamicColumns = computed(() => {
  if (props.rows.length === 0) return [];
  const firstRow = props.rows[0];
  if (firstRow.spec && typeof firstRow.spec === 'object') {
    return Object.keys(firstRow.spec).filter(k => typeof firstRow.spec[k] === 'string' || typeof firstRow.spec[k] === 'number').slice(0, 2);
  }
  return [];
});

const getSpecValue = (row: any, col: string) => {
  if (row._raw && row._raw.spec) {
    return row._raw.spec[col];
  }
  return '';
};

</script>

<template>
  <div class="resource-table">
    <div class="list-header">
      <div class="header-left">
        <h3>{{ kind }} ({{ filteredRows.length }})</h3>
      </div>
      <input 
        v-model="searchQuery" 
        type="text" 
        :placeholder="`Filter ${kind.toLowerCase()}...`" 
        class="search-input"
      />
    </div>

    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th v-if="filteredRows.some(r => r.namespace)">Namespace</th>
            <th>Created At</th>
            <th v-for="col in dynamicColumns" :key="col">Spec: {{ col }}</th>
            <th v-if="filteredRows.some(r => r.status)">Status</th>
            <th class="col-actions">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr 
            v-for="row in filteredRows" 
            :key="row.name + row.namespace"
            @click.stop="selectResource(row.namespace, row.name)"
          >
            <td class="resource-name-cell">
              {{ row.name }}
            </td>
            <td v-if="row.namespace">{{ row.namespace }}</td>
            <td>{{ row.creationTimestamp }}</td>
            <td v-for="col in dynamicColumns" :key="col">{{ getSpecValue(row, col) }}</td>
            <td v-if="row.status">
              <span :class="['status-badge', row.status.toLowerCase()]">
                {{ row.status }}
              </span>
            </td>
            <td class="col-actions" @click.stop>
              <button 
                @click="emit('edit-yaml', row.namespace, row.name, kind)" 
                class="btn-action edit-yaml-btn"
                title="Edit YAML Manifest"
              >
                ✏️ YAML
              </button>
              <button 
                v-if="kind === 'Pod'"
                @click="openPfModal(row.namespace, row.name)" 
                class="btn-action pf-btn"
                title="Port Forward"
              >
                🔌 Forward
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    
    <PortForwardModal
      v-if="showPfModal"
      :context-name="contextName"
      :namespace="pfTarget.namespace"
      :pod-name="pfTarget.name"
      @close="showPfModal = false"
      @success="showPfModal = false"
    />
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
.status-badge {
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
}
.running, .active, .ready { background-color: rgba(6, 95, 70, 0.2); color: #34d399; border: 1px solid rgba(52, 211, 153, 0.2); }
.pending { background-color: rgba(146, 64, 14, 0.2); color: #fbbf24; border: 1px solid rgba(251, 191, 36, 0.2); }
.failed, .inactive, .error { background-color: rgba(153, 27, 27, 0.2); color: #f87171; border: 1px solid rgba(248, 113, 113, 0.2); }

.col-actions {
  width: 200px;
  text-align: center;
  display: flex;
  gap: 8px;
  justify-content: center;
}
.btn-action {
  background-color: #1e293b;
  border: 1px solid #334155;
  color: #cbd5e1;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-action:hover {
  background-color: rgba(168, 85, 247, 0.1);
  border-color: #a855f7;
  color: #c084fc;
}
</style>
