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

const CATEGORY_MAP: Record<string, string> = {
  'Pod': 'Workloads',
  'Deployment': 'Workloads',
  'DaemonSet': 'Workloads',
  'StatefulSet': 'Workloads',
  'ReplicaSet': 'Workloads',
  'Job': 'Workloads',
  'CronJob': 'Workloads',
  'Service': 'Network',
  'Endpoints': 'Network',
  'Ingress': 'Network',
  'IngressClass': 'Network',
  'NetworkPolicy': 'Network',
  'ConfigMap': 'Config',
  'Secret': 'Config',
  'ResourceQuota': 'Config',
  'LimitRange': 'Config',
  'HorizontalPodAutoscaler': 'Config',
  'PodDisruptionBudget': 'Config',
  'PersistentVolumeClaim': 'Storage',
  'PersistentVolume': 'Storage',
  'StorageClass': 'Storage',
  'ServiceAccount': 'Access Control',
  'ClusterRole': 'Access Control',
  'ClusterRoleBinding': 'Access Control',
  'Role': 'Access Control',
  'RoleBinding': 'Access Control',
};

const GROUP_ICONS: Record<string, string> = {
  'Workloads': '📦',
  'Network': '🌐',
  'Config': '⚙️',
  'Storage': '💾',
  'Access Control': '🔐',
  'Custom Resources': '🧩',
};

const logicalGroups = ref<Record<string, DynamicResourceInfo[]>>({});
const expandedGroups = ref<Record<string, boolean>>({});

const fetchApiResources = async () => {
  if (!props.contextName) return;
  try {
    const res = await invoke<Record<string, DynamicResourceInfo[]>>('get_api_resources', {
      contextName: props.contextName
    });
    
    const newGroups: Record<string, DynamicResourceInfo[]> = {
      'Workloads': [],
      'Network': [],
      'Config': [],
      'Storage': [],
      'Access Control': [],
      'Custom Resources': [],
    };

    for (const group in res) {
      for (const resource of res[group]) {
        const category = CATEGORY_MAP[resource.kind] || 'Custom Resources';
        newGroups[category].push(resource);
      }
    }

    logicalGroups.value = newGroups;
    expandedGroups.value['Workloads'] = true;
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
  emit('select-resource-type', resource);
};
</script>

<template>
  <div class="resource-tree">
    <template v-for="(resources, groupName) in logicalGroups" :key="groupName">
      <div v-if="resources.length > 0" class="tree-group">
        <div class="group-header" @click="toggleGroup(groupName as string)">
          <span class="chevron" :class="{ 'is-expanded': expandedGroups[groupName as string] }">▶</span>
          <span class="icon group-icon">{{ GROUP_ICONS[groupName as string] || '🧩' }}</span>
          <span class="label">{{ groupName }}</span>
        </div>
        <div v-if="expandedGroups[groupName as string]" class="group-items">
          <div v-for="res in resources" :key="res.kind" class="tree-item" @click="handleItemClick(res)">
            <span class="label">{{ res.kind }}</span>
          </div>
        </div>
      </div>
    </template>
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
  padding-left: 24px;
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
  font-size: 0.8rem;
  margin-right: 8px;
}

.label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
