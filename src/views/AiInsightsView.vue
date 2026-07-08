<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  contextName: string | null;
}>();

interface Recommendation {
  target_name: string;
  target_kind: string;
  namespace: string;
  reason: string;
  action: string;
}

const recommendations = ref<Recommendation[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const fetchRecommendations = async () => {
  if (!props.contextName) return;
  loading.value = true;
  error.value = null;
  try {
    recommendations.value = await invoke<Recommendation[]>('analyze_workloads', { 
      contextName: props.contextName 
    });
  } catch (e: any) {
    console.error('Failed to analyze workloads:', e);
    error.value = e.toString();
  } finally {
    loading.value = false;
  }
};

watch(() => props.contextName, fetchRecommendations);

onMounted(() => {
  fetchRecommendations();
});

const applyRecommendation = async (rec: Recommendation) => {
  // Try to suspend the namespace or scale down depending on action, as MVP.
  try {
    if (rec.action === 'Suspend') {
      await invoke('suspend_namespace', { contextName: props.contextName, namespace: rec.namespace });
      alert(`Suspended namespace ${rec.namespace} successfully.`);
    } else {
      alert(`Action ${rec.action} on ${rec.target_name} is not fully implemented in MVP.`);
    }
  } catch (e: any) {
    alert(`Failed to apply action: ${e}`);
  }
};
</script>

<template>
  <div class="ai-insights-view">
    <header class="view-header">
      <h2>AI Profiling Insights</h2>
      <p>Smart recommendations to optimize your Kubernetes workloads based on historical telemetry and heuristics.</p>
      <button class="btn-primary" @click="fetchRecommendations" :disabled="loading">
        {{ loading ? 'Analyzing...' : 'Run Analysis' }}
      </button>
    </header>

    <div v-if="error" class="error-banner">
      {{ error }}
    </div>

    <div class="recommendations-list" v-if="recommendations.length > 0">
      <div v-for="(rec, idx) in recommendations" :key="idx" class="recommendation-card">
        <div class="card-header">
          <span class="kind-badge">{{ rec.target_kind }}</span>
          <h3>{{ rec.target_name }} <span class="namespace-text">in {{ rec.namespace }}</span></h3>
        </div>
        <div class="card-body">
          <p class="reason-text">{{ rec.reason }}</p>
        </div>
        <div class="card-actions">
          <button class="btn-action" @click="applyRecommendation(rec)">
            <span class="action-icon">✨</span>
            {{ rec.action === 'Suspend' ? 'Suspend Namespace' : 'Apply ' + rec.action }}
          </button>
        </div>
      </div>
    </div>
    
    <div v-else-if="!loading && recommendations.length === 0" class="empty-state">
      <span class="empty-icon">🤖</span>
      <p>No optimization recommendations found at this time.</p>
    </div>
    
    <div v-if="loading" class="loading-state">
      <span class="spinner"></span>
      <p>Analyzing workloads and metrics...</p>
    </div>
  </div>
</template>

<style scoped>
.ai-insights-view {
  padding: var(--space-6);
  color: #f3f4f6;
  height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
  background-color: var(--surface-dark);
}

.view-header {
  margin-bottom: var(--space-6);
  padding-bottom: var(--space-4);
  border-bottom: 1px solid var(--border-dim);
}

.view-header h2 {
  margin: 0 0 var(--space-2) 0;
  font-size: 1.8rem;
  font-weight: 800;
  background: linear-gradient(90deg, #3b82f6, #8b5cf6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.view-header p {
  color: #9ca3af;
  margin: 0 0 var(--space-4) 0;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2563eb;
}

.btn-primary:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.error-banner {
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #ef4444;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-4);
}

.recommendations-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.recommendation-card {
  background-color: var(--surface-glass);
  border: 1px solid var(--border-dim);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s, box-shadow 0.2s;
}

.recommendation-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
  border-color: rgba(59, 130, 246, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}

.card-header h3 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
}

.namespace-text {
  font-size: 0.85rem;
  font-weight: 400;
  color: #6b7280;
}

.kind-badge {
  background-color: rgba(139, 92, 246, 0.1);
  color: #c084fc;
  border: 1px solid rgba(139, 92, 246, 0.2);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 700;
}

.card-body {
  margin-bottom: var(--space-4);
}

.reason-text {
  color: #d1d5db;
  line-height: 1.5;
  margin: 0;
}

.card-actions {
  display: flex;
  justify-content: flex-end;
}

.btn-action {
  background-color: rgba(16, 185, 129, 0.1);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.2);
  padding: 6px 14px;
  border-radius: var(--radius-md);
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  transition: all 0.2s;
}

.btn-action:hover {
  background-color: rgba(16, 185, 129, 0.2);
}

.empty-state, .loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem;
  color: #6b7280;
  text-align: center;
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.spinner {
  width: 30px;
  height: 30px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  border-top-color: #3b82f6;
  animation: spin 1s ease-in-out infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
