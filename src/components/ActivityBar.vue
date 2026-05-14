<script setup lang="ts">
import { 
  LayoutGrid, 
  Bot, 
  ShoppingBag, 
  Activity 
} from 'lucide-vue-next';

defineProps<{
  activeId: string;
}>();

const emit = defineEmits<{
  (e: 'update:activeId', id: string): void;
}>();

const items = [
  { id: 'explorer', icon: LayoutGrid, title: 'Explorer' },
  { id: 'ai', icon: Bot, title: 'AI Assistant' },
  { id: 'marketplace', icon: ShoppingBag, title: 'Marketplace' },
  { id: 'hardware', icon: Activity, title: 'Hardware' },
];

const selectItem = (id: string) => {
  emit('update:activeId', id);
};
</script>

<template>
  <nav class="activity-bar">
    <div 
      v-for="item in items" 
      :key="item.id"
      class="activity-item"
      :class="{ active: activeId === item.id }"
      :title="item.title"
      @click="selectItem(item.id)"
    >
      <component :is="item.icon" :size="24" stroke-width="1.5" />
    </div>
  </nav>
</template>

<style scoped>
.activity-bar {
  width: 48px;
  background-color: #1f2937;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 12px;
  border-right: 1px solid #374151;
  height: 100%;
}

.activity-item {
  width: 100%;
  height: 48px;
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  color: #9ca3af;
  font-size: 0.75rem;
  font-weight: bold;
  transition: all 0.2s;
  border-left: 2px solid transparent;
}

.activity-item:hover {
  color: #f3f4f6;
  background-color: rgba(255, 255, 255, 0.05);
}

.activity-item.active {
  color: #3b82f6;
  border-left-color: #3b82f6;
  background-color: rgba(59, 130, 246, 0.1);
}
</style>
