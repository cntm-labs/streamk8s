<script setup lang="ts">
defineProps<{
  title: string;
  width: number;
}>();
defineEmits(['start-resize']);
</script>

<template>
  <aside class="sidebar" :style="{ width: width + 'px' }">
    <div class="sidebar-header">
      <h2>{{ title }}</h2>
    </div>
    <div class="sidebar-content">
      <slot></slot>
    </div>
    <div class="resizer" @mousedown="$emit('start-resize', $event)"></div>
  </aside>
</template>

<style scoped>
.sidebar {
  position: relative;
  background-color: var(--surface-glass);
  backdrop-filter: var(--glass-blur);
  border-right: var(--glass-border);
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
  overflow: hidden;
  z-index: 5;
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
  padding: 8px 4px;
}

.resizer {
  position: absolute;
  top: 0;
  right: 0;
  width: 2px;
  height: 100%;
  cursor: col-resize;
  z-index: 100;
  background-color: transparent;
  transition: var(--transition-smooth);
}

.resizer:hover, .resizer:active {
  background-color: var(--accent-blue);
  box-shadow: 0 0 8px var(--accent-blue);
}
</style>
