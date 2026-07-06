<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  contextName: string | null;
}>();

const emit = defineEmits(['select-resource-type']);

interface DynamicResourceInfo {
  group: string;
  version: string;
  kind: string;
  plural: string;
  namespaced: boolean;
}

const apiGroups = ref<Record<string, DynamicResourceInfo[]>>({});
const expandedGroups = ref<Record<string, boolean>>({});

const fetchApiResources = async () => {
  if (!props.contextName) return;
  try {
    const res = await invoke<Record<string, DynamicResourceInfo[]>>('get_api_resources', {
      contextName: props.contextName
    });
    apiGroups.value = res;
    // expand core by default
    expandedGroups.value['core'] = true;
  } catch (e) {
    console.error('Failed to fetch api resources', e);
  }
};

watch(() => props.contextName, () => {
  fetchApiResources();
});

onMounted(() => {
  fetchApiResources();
});

const toggleGroup = (group: string) => {
  expandedGroups.value[group] = !expandedGroups.value[group];
};

const handleItemClick = (resource: DynamicResourceInfo) => {
  // Pass an object so we can use dynamic list
  emit('select-resource-type', resource);
};
</script>

<template>
  <div class="resource-tree">
    <div v-for="(resources, groupName) in apiGroups" :key="groupName" class="tree-group">
      <div class="group-header" @click="toggleGroup(groupName)">
        <span class="chevron" :class="{ 'is-expanded': expandedGroups[groupName] }">▶</span>
        <span class="label">{{ groupName === 'core' ? 'Core' : groupName }}</span>
      </div>
      <div v-if="expandedGroups[groupName]" class="group-items">
        <div v-for="res in resources" :key="res.kind" class="tree-item" @click="handleItemClick(res)">
          <span class="icon default-icon">{{ res.kind.charAt(0) }}</span>
          <span class="label">{{ res.kind }}</span>
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
.default-icon { color: #3b82f6; border: 1px solid #3b82f6; }
.label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
