<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { Search, Globe, Box, Zap, Command } from 'lucide-vue-next';

interface CommandItem {
  id: string;
  label: string;
  type: 'cluster' | 'resource' | 'action';
  description?: string;
}

const emit = defineEmits(['close', 'select']);

const query = ref('');
const selectedIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

// Mock data for UI demonstration
const items: CommandItem[] = [
  { id: 'c1', label: 'docker-desktop', type: 'cluster', description: 'Switch to Docker Desktop context' },
  { id: 'c2', label: 'minikube', type: 'cluster', description: 'Switch to Minikube context' },
  { id: 'r1', label: 'Pods', type: 'resource', description: 'View all Pods' },
  { id: 'r2', label: 'Deployments', type: 'resource', description: 'View all Deployments' },
  { id: 'r3', label: 'Services', type: 'resource', description: 'View all Services' },
  { id: 'a1', label: 'Optimize Resources', type: 'action', description: 'Run smart optimization' },
  { id: 'a2', label: 'Settings', type: 'action', description: 'Open application settings' },
];

const filteredItems = ref<CommandItem[]>(items);

const filterItems = () => {
  if (!query.value) {
    filteredItems.value = items;
  } else {
    const q = query.value.toLowerCase();
    filteredItems.value = items.filter(item => 
      item.label.toLowerCase().includes(q) || 
      item.description?.toLowerCase().includes(q)
    );
  }
  selectedIndex.value = 0;
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowDown') {
    selectedIndex.value = (selectedIndex.value + 1) % filteredItems.value.length;
    e.preventDefault();
  } else if (e.key === 'ArrowUp') {
    selectedIndex.value = (selectedIndex.value - 1 + filteredItems.value.length) % filteredItems.value.length;
    e.preventDefault();
  } else if (e.key === 'Enter') {
    if (filteredItems.value[selectedIndex.value]) {
      handleSelect(filteredItems.value[selectedIndex.value]);
    }
  } else if (e.key === 'Escape') {
    emit('close');
  }
};

const handleSelect = (item: CommandItem) => {
  emit('select', item);
  emit('close');
};

const closeOnOutsideClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (target.classList.contains('palette-overlay')) {
    emit('close');
  }
};

onMounted(() => {
  nextTick(() => {
    inputRef.value?.focus();
  });
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <div class="palette-overlay" @click="closeOnOutsideClick">
    <div class="palette-container">
      <div class="palette-search">
        <Search class="search-icon" :size="20" />
        <input 
          ref="inputRef"
          v-model="query" 
          type="text" 
          placeholder="Type a command or search..." 
          @input="filterItems"
        />
        <div class="palette-shortcut">ESC</div>
      </div>
      
      <div class="palette-results" v-if="filteredItems.length > 0">
        <div 
          v-for="(item, index) in filteredItems" 
          :key="item.id"
          class="result-item"
          :class="{ selected: index === selectedIndex }"
          @mouseenter="selectedIndex = index"
          @click="handleSelect(item)"
        >
          <div class="item-icon">
            <Globe v-if="item.type === 'cluster'" :size="16" />
            <Box v-else-if="item.type === 'resource'" :size="16" />
            <Zap v-else-if="item.type === 'action'" :size="16" />
            <Command v-else :size="16" />
          </div>
          <div class="item-info">
            <div class="item-label">{{ item.label }}</div>
            <div class="item-description" v-if="item.description">{{ item.description }}</div>
          </div>
          <div class="item-type-badge">{{ item.type }}</div>
        </div>
      </div>
      
      <div class="palette-no-results" v-else>
        No results found for "{{ query }}"
      </div>
      
      <div class="palette-footer">
        <div class="footer-hint">
          <span><kbd>↑↓</kbd> to navigate</span>
          <span><kbd>↵</kbd> to select</span>
          <span><kbd>ESC</kbd> to close</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.palette-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 100px;
  z-index: 9999;
  backdrop-filter: blur(4px);
}

.palette-container {
  width: 600px;
  max-width: 90vw;
  background-color: #1f2937;
  border: 1px solid #374151;
  border-radius: 12px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 10px 10px -5px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  animation: slide-down 0.2s ease-out;
}

@keyframes slide-down {
  from { opacity: 0; transform: translateY(-20px); }
  to { opacity: 1; transform: translateY(0); }
}

.palette-search {
  display: flex;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #374151;
  gap: 12px;
}

.search-icon {
  color: #9ca3af;
}

.palette-search input {
  flex: 1;
  background: none;
  border: none;
  color: #f3f4f6;
  font-size: 1.1rem;
  outline: none;
}

.palette-shortcut {
  background-color: #374151;
  color: #9ca3af;
  font-size: 0.7rem;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: bold;
}

.palette-results {
  max-height: 400px;
  overflow-y: auto;
  padding: 8px;
}

.result-item {
  display: flex;
  align-items: center;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  gap: 12px;
  transition: background-color 0.15s;
}

.result-item.selected {
  background-color: #3b82f6;
}

.item-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #374151;
  border-radius: 6px;
  color: #3b82f6;
}

.result-item.selected .item-icon {
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
}

.item-info {
  flex: 1;
}

.item-label {
  font-weight: 600;
  font-size: 0.95rem;
  color: #f3f4f6;
}

.item-description {
  font-size: 0.8rem;
  color: #9ca3af;
}

.result-item.selected .item-label,
.result-item.selected .item-description {
  color: white;
}

.item-type-badge {
  font-size: 0.7rem;
  text-transform: uppercase;
  color: #6b7280;
  font-weight: 700;
  background-color: #111827;
  padding: 2px 8px;
  border-radius: 10px;
}

.result-item.selected .item-type-badge {
  background-color: rgba(0, 0, 0, 0.2);
  color: rgba(255, 255, 255, 0.8);
}

.palette-no-results {
  padding: 32px;
  text-align: center;
  color: #9ca3af;
}

.palette-footer {
  padding: 12px 16px;
  background-color: #111827;
  border-top: 1px solid #374151;
}

.footer-hint {
  display: flex;
  gap: 16px;
  font-size: 0.75rem;
  color: #6b7280;
}

.footer-hint kbd {
  background-color: #374151;
  color: #d1d5db;
  padding: 1px 4px;
  border-radius: 3px;
  font-family: monospace;
}
</style>
