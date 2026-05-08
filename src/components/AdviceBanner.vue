<script setup lang="ts">
defineProps<{
  advice: { action: string, reason: string } | null;
}>();
defineEmits(['optimize']);
</script>

<template>
  <div v-if="advice" class="advice-banner">
    <div class="icon">🤖 AI ADVICE</div>
    <div class="content">
      <strong>{{ advice.action }}:</strong> {{ advice.reason }}
    </div>
    <button @click="$emit('optimize')" class="optimize-btn">
      OPTIMIZE NOW
    </button>
  </div>
</template>

<style scoped>
.advice-banner {
  background: linear-gradient(90deg, #1e3a8a, #1e40af);
  border: 1px solid #3b82f6;
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1.5rem;
  animation: slide-in 0.5s ease-out;
}
.icon { font-weight: 800; font-size: 0.7rem; color: #60a5fa; }
.content { flex: 1; font-size: 0.85rem; color: #e5e7eb; }
.optimize-btn {
  background-color: #3b82f6;
  color: white;
  border: none;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
}
.optimize-btn:hover { background-color: #2563eb; transform: scale(1.05); }

@keyframes slide-in {
  from { transform: translateY(-20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}
</style>
