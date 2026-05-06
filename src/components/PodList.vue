<script setup lang="ts">
import { ref, computed } from 'vue';

interface Pod {
  name: string;
  namespace: string;
  status: string;
}
const props = defineProps<{ pods: Pod[] }>();

const searchQuery = ref('');

const filteredPods = computed(() => {
  if (!searchQuery.value) return props.pods;
  const query = searchQuery.value.toLowerCase();
  return props.pods.filter(pod => 
    pod.name.toLowerCase().includes(query) || 
    pod.namespace.toLowerCase().includes(query)
  );
});
</script>

<template>
  <div class="pod-list">
    <div class="list-header">
      <h3>Active Workloads ({{ filteredPods.length }})</h3>
      <input 
        v-model="searchQuery" 
        type="text" 
        placeholder="Filter pods..." 
        class="search-input"
      />
    </div>
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Namespace</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="pod in filteredPods" :key="pod.name">
            <td>{{ pod.name }}</td>
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
}
.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
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
tr:hover {
  background-color: #2d3748;
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
