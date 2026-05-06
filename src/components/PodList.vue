<script setup lang="ts">
interface Pod {
  name: string;
  namespace: string;
  status: string;
}
defineProps<{ pods: Pod[] }>();
</script>

<template>
  <div class="pod-list">
    <h3>Active Workloads</h3>
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Namespace</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="pod in pods" :key="pod.name">
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
</template>

<style scoped>
.pod-list {
  background-color: #1f2937;
  padding: 1rem;
  border-radius: 8px;
  border: 1px solid #374151;
}
h3 {
  margin-top: 0;
  font-size: 1.125rem;
  margin-bottom: 1rem;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th {
  text-align: left;
  font-size: 0.75rem;
  color: #9ca3af;
  padding-bottom: 0.5rem;
}
td {
  padding: 0.5rem 0;
  font-size: 0.875rem;
  border-top: 1px solid #374151;
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
