<script setup lang="ts">
defineProps<{
  label: string;
  value: number;
  color: string;
}>();
</script>

<template>
  <div class="gauge-container">
    <div class="label">{{ label }}</div>
    <div class="bar-bg">
      <div class="bar-fill" :style="{ width: value + '%', backgroundColor: color }"></div>
    </div>
    <div class="value">{{ Math.round(value) }}%</div>
  </div>
</template>

<style scoped>
.gauge-container {
  margin-bottom: 1.5rem;
  width: 100%;
}
.label {
  font-size: 0.875rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #9ca3af;
}
.bar-bg {
  height: 8px;
  background-color: #374151;
  border-radius: 4px;
  overflow: hidden;
}
.bar-fill {
  height: 100%;
  transition: width 0.3s ease;
}
.value {
  text-align: right;
  font-size: 0.75rem;
  margin-top: 0.25rem;
  color: #6b7280;
}
</style>
