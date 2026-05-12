<script setup lang="ts">
defineProps<{
  title: string;
}>();
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <h2>{{ title }}</h2>
    </div>
    <div class="sidebar-content">
      <slot></slot>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 260px;
  background-color: #111827;
  border-right: 1px solid #374151;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.sidebar-header {
  height: 35px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  text-transform: uppercase;
  font-size: 0.7rem;
  color: #9ca3af;
  letter-spacing: 0.5px;
}

.sidebar-header h2 {
  margin: 0;
  font-weight: 600;
  font-size: inherit;
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
}
</style>
