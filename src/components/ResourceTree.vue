<script setup lang="ts">
import { ref } from 'vue';

const expandedGroups = ref<Record<string, boolean>>({
  workloads: true,
  network: false,
  configuration: false,
});

const toggleGroup = (group: string) => {
  expandedGroups.value[group] = !expandedGroups.value[group];
};

const emit = defineEmits(['select-resource-type']);

const handleItemClick = (type: string) => {
  emit('select-resource-type', type);
};
</script>

<template>
  <div class="resource-tree">
    <!-- Workloads -->
    <div class="tree-group">
      <div class="group-header" @click="toggleGroup('workloads')">
        <span class="chevron" :class="{ 'is-expanded': expandedGroups.workloads }">▶</span>
        <span class="label">Workloads</span>
      </div>
      <div v-if="expandedGroups.workloads" class="group-items">
        <div class="tree-item" @click="handleItemClick('pods')">
          <span class="icon pod-icon">P</span>
          <span class="label">Pods</span>
        </div>
        <div class="tree-item" @click="handleItemClick('deployments')">
          <span class="icon deploy-icon">D</span>
          <span class="label">Deployments</span>
        </div>
        <div class="tree-item" @click="handleItemClick('statefulsets')">
          <span class="icon ss-icon">S</span>
          <span class="label">StatefulSets</span>
        </div>
      </div>
    </div>

    <!-- Network -->
    <div class="tree-group">
      <div class="group-header" @click="toggleGroup('network')">
        <span class="chevron" :class="{ 'is-expanded': expandedGroups.network }">▶</span>
        <span class="label">Network</span>
      </div>
      <div v-if="expandedGroups.network" class="group-items">
        <div class="tree-item" @click="handleItemClick('services')">
          <span class="icon svc-icon">S</span>
          <span class="label">Services</span>
        </div>
        <div class="tree-item" @click="handleItemClick('ingresses')">
          <span class="icon ing-icon">I</span>
          <span class="label">Ingresses</span>
        </div>
      </div>
    </div>

    <!-- Configuration -->
    <div class="tree-group">
      <div class="group-header" @click="toggleGroup('configuration')">
        <span class="chevron" :class="{ 'is-expanded': expandedGroups.configuration }">▶</span>
        <span class="label">Configuration</span>
      </div>
      <div v-if="expandedGroups.configuration" class="group-items">
        <div class="tree-item" @click="handleItemClick('configmaps')">
          <span class="icon cm-icon">C</span>
          <span class="label">ConfigMaps</span>
        </div>
        <div class="tree-item" @click="handleItemClick('secrets')">
          <span class="icon sec-icon">S</span>
          <span class="label">Secrets</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.resource-tree {
  user-select: none;
  font-size: 0.85rem;
  color: #d1d5db;
}

.tree-group {
  margin-bottom: 2px;
}

.group-header {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  cursor: pointer;
  border-radius: 4px;
  font-weight: 500;
  color: #9ca3af;
}

.group-header:hover {
  background-color: #1f2937;
  color: #f3f4f6;
}

.chevron {
  font-size: 0.6rem;
  margin-right: 6px;
  transition: transform 0.2s;
  display: inline-block;
}

.chevron.is-expanded {
  transform: rotate(90deg);
}

.group-items {
  padding-left: 16px;
}

.tree-item {
  display: flex;
  align-items: center;
  padding: 4px 12px;
  cursor: pointer;
  border-radius: 4px;
}

.tree-item:hover {
  background-color: #1f2937;
  color: #3b82f6;
}

.icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.6rem;
  font-weight: bold;
  margin-right: 8px;
  border-radius: 2px;
  background-color: #374151;
  color: #9ca3af;
}

.pod-icon { color: #10b981; border: 1px solid #10b981; }
.deploy-icon { color: #3b82f6; border: 1px solid #3b82f6; }
.ss-icon { color: #8b5cf6; border: 1px solid #8b5cf6; }
.svc-icon { color: #f59e0b; border: 1px solid #f59e0b; }
.ing-icon { color: #ec4899; border: 1px solid #ec4899; }
.cm-icon { color: #6b7280; border: 1px solid #6b7280; }
.sec-icon { color: #ef4444; border: 1px solid #ef4444; }

.label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
