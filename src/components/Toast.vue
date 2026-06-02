<script setup lang="ts">
import { watch } from 'vue';
import { CheckCircle2, AlertCircle, X } from 'lucide-vue-next';

const props = defineProps<{
  message: string;
  type: 'success' | 'error';
  visible: boolean;
  duration?: number;
}>();

const emit = defineEmits(['update:visible']);

const close = () => {
  emit('update:visible', false);
};

watch(() => props.visible, (newVal) => {
  if (newVal) {
    setTimeout(() => {
      close();
    }, props.duration || 3000);
  }
});
</script>

<template>
  <Transition name="toast">
    <div v-if="visible" class="toast-wrapper" :class="type">
      <div class="toast-icon">
        <CheckCircle2 v-if="type === 'success'" :size="18" />
        <AlertCircle v-else :size="18" />
      </div>
      <div class="toast-content">
        {{ message }}
      </div>
      <button class="toast-close" @click="close">
        <X :size="14" />
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.toast-wrapper {
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  z-index: 1000;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  background: #111827;
  border: 1px solid #1f2937;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.2);
  min-width: 280px;
  max-width: 400px;
}

.toast-wrapper.success {
  border-left: 4px solid #10b981;
  box-shadow: 0 0 15px rgba(16, 185, 129, 0.1);
}

.toast-wrapper.error {
  border-left: 4px solid #ef4444;
  box-shadow: 0 0 15px rgba(239, 68, 68, 0.1);
}

.toast-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.success .toast-icon { color: #10b981; }
.error .toast-icon { color: #ef4444; }

.toast-content {
  flex: 1;
  font-size: 0.875rem;
  color: #f3f4f6;
  font-weight: 500;
}

.toast-close {
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.toast-close:hover {
  background: #1f2937;
  color: #f3f4f6;
}

/* Animations */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
