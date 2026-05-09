<script setup lang="ts">
import { ref } from 'vue';
import PodList from './PodList.vue';

interface Pod {
  name: string;
  namespace: string;
  status: string;
}

const props = defineProps<{
  contextName: string;
  isCurrent: boolean;
  pods: Pod[];
}>();

const emit = defineEmits<{
  (e: 'select-pod', contextName: string, namespace: string, name: string): void
}>();

const isOpen = ref(props.isCurrent);

const toggle = () => {
  isOpen.value = !isOpen.value;
};

const handleSelectPod = (namespace: string, name: string) => {
  emit('select-pod', props.contextName, namespace, name);
};
</script>

<template>
  <div class="cluster-accordion" :class="{ 'is-current': isCurrent }">
    <div class="accordion-header" @click="toggle">
      <div class="header-left">
        <span class="chevron" :class="{ 'is-open': isOpen }">▶</span>
        <span class="context-name">{{ contextName }}</span>
        <span v-if="isCurrent" class="current-badge">Current</span>
      </div>
      <div class="header-right">
        <span class="pod-count">{{ pods.length }} Pods</span>
      </div>
    </div>
    <div v-if="isOpen" class="accordion-body">
      <PodList 
        :pods="pods" 
        :context-name="contextName"
        @select-pod="handleSelectPod" 
      />
    </div>
  </div>
</template>

<style scoped>
.cluster-accordion {
  margin-bottom: 0.5rem;
  border: 1px solid #374151;
  border-radius: 6px;
  overflow: hidden;
  background-color: #1f2937;
}
.cluster-accordion.is-current {
  border-color: #3b82f6;
}
.accordion-header {
  padding: 0.75rem 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  background-color: #1f2937;
  user-select: none;
}
.accordion-header:hover {
  background-color: #2d3748;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.chevron {
  font-size: 0.75rem;
  transition: transform 0.2s;
  color: #9ca3af;
}
.chevron.is-open {
  transform: rotate(90deg);
}
.context-name {
  font-weight: 600;
  font-family: monospace;
}
.current-badge {
  font-size: 0.7rem;
  background-color: #3b82f6;
  color: white;
  padding: 1px 6px;
  border-radius: 4px;
  text-transform: uppercase;
}
.pod-count {
  font-size: 0.875rem;
  color: #9ca3af;
}
.accordion-body {
  border-top: 1px solid #374151;
  background-color: #111827;
}
</style>
