<script setup lang="ts">
defineProps<{ data: number[], color: string }>();
</script>

<template>
  <div class="trend-container">
    <svg viewBox="0 0 100 20" preserveAspectRatio="none">
      <rect 
        v-for="(v, i) in data" 
        :key="i"
        :x="i * (100 / 60)" 
        :y="20 - (v * 0.2)"
        :width="1" 
        :height="v * 0.2"
        :fill="color" 
        opacity="0.5" 
      />
    </svg>
  </div>
</template>

<style scoped>
.trend-container {
  height: 24px;
  width: 100%;
  margin-top: 4px;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
  overflow: hidden;
}
svg {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
