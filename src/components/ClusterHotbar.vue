<script setup lang="ts">
interface ClusterContext {
  name: string;
  is_current: boolean;
}
defineProps<{
  contexts: ClusterContext[];
  activeName: string | null;
}>();
defineEmits(['select']);
</script>

<template>
  <div class="cluster-hotbar">
    <div 
      v-for="c in contexts" 
      :key="c.name"
      :class="['cluster-icon', { active: activeName === c.name, current: c.is_current }]"
      @click="$emit('select', c.name)"
      :title="c.name"
    >
      <div class="icon-text">{{ c.name.substring(0, 2).toUpperCase() }}</div>
    </div>
  </div>
</template>

<style scoped>
.cluster-hotbar {
  background-color: #111827; /* Gray-900 */
  border-right: 1px solid #374151; /* Gray-700 */
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 0;
  gap: 12px;
  overflow-y: auto;
}

.cluster-icon {
  width: 36px;
  height: 36px;
  background-color: #1f2937;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  border: 2px solid transparent;
}

.cluster-icon:hover {
  background-color: #374151;
  border-radius: 12px;
}

.cluster-icon.active {
  background-color: #1e3a8a;
  border-color: #3b82f6;
  border-radius: 12px;
}

.cluster-icon.current::before {
  content: '';
  position: absolute;
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 20px;
  background-color: #3b82f6;
  border-radius: 0 4px 4px 0;
}

.icon-text {
  font-size: 0.75rem;
  font-weight: 700;
  color: #d1d5db;
}

.cluster-icon.active .icon-text {
  color: white;
}
</style>
