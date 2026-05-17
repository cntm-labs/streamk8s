<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Save, CheckCircle } from 'lucide-vue-next';

interface AppConfig {
  ai_provider: string;
  api_key: string;
  endpoint: string;
  model: string;
}

const config = ref<AppConfig>({
  ai_provider: 'OpenAI',
  api_key: '',
  endpoint: 'https://api.openai.com/v1',
  model: 'gpt-4-turbo'
});

const isSaving = ref(false);
const showSuccess = ref(false);

const loadConfig = async () => {
  try {
    const savedConfig = await invoke<AppConfig>('get_config');
    config.value = savedConfig;
  } catch (e) {
    console.error('Failed to load settings:', e);
  }
};

const saveConfig = async () => {
  isSaving.value = true;
  try {
    await invoke('save_config', { config: config.value });
    showSuccess.value = true;
    setTimeout(() => { showSuccess.value = false; }, 3000);
  } catch (e) {
    alert(`Failed to save settings: ${e}`);
  } finally {
    isSaving.value = false;
  }
};

const updateEndpoint = () => {
  if (config.value.ai_provider === 'OpenAI') config.value.endpoint = 'https://api.openai.com/v1';
  else if (config.value.ai_provider === 'Gemini') config.value.endpoint = 'https://generativelanguage.googleapis.com';
  else if (config.value.ai_provider === 'Claude') config.value.endpoint = 'https://api.anthropic.com/v1';
  else if (config.value.ai_provider === 'Ollama') config.value.endpoint = 'http://localhost:11434';
};

onMounted(loadConfig);
</script>

<template>
  <div class="settings-container">
    <div class="settings-header">
      <div class="header-left">
        <h1>Settings</h1>
        <p>Manage AI providers and application preferences</p>
      </div>
      <div class="header-right">
        <button @click="saveConfig" class="save-btn" :disabled="isSaving">
          <Save v-if="!isSaving" :size="18" class="mr-2" />
          <span v-if="!isSaving">Save Changes</span>
          <span v-else>Saving...</span>
        </button>
      </div>
    </div>

    <div v-if="showSuccess" class="success-banner">
      <CheckCircle :size="18" />
      <span>Settings saved successfully!</span>
    </div>

    <div class="settings-content">
      <div class="settings-section">
        <h3>🤖 AI Configuration</h3>
        
        <div class="setting-item">
          <div class="info">
            <label>AI Provider</label>
            <p>Select the engine to use for cluster diagnostics</p>
          </div>
          <select v-model="config.ai_provider" @change="updateEndpoint">
            <option>OpenAI</option>
            <option>Gemini</option>
            <option>Claude</option>
            <option>Ollama</option>
          </select>
        </div>

        <div class="setting-item" v-if="config.ai_provider !== 'Ollama'">
          <div class="info">
            <label>API Key</label>
            <p>Your authentication token for the selected provider</p>
          </div>
          <input 
            v-model="config.api_key" 
            type="password" 
            placeholder="sk-..." 
            class="config-input" 
          />
        </div>

        <div class="setting-item">
          <div class="info">
            <label>API Endpoint</label>
            <p>The URL used to communicate with the AI service</p>
          </div>
          <input 
            v-model="config.endpoint" 
            type="text" 
            placeholder="http://..." 
            class="config-input" 
          />
        </div>

        <div class="setting-item">
          <div class="info">
            <label>Model Name</label>
            <p>Specify the model ID (e.g., gpt-4, llama3, gemini-pro)</p>
          </div>
          <input 
            v-model="config.model" 
            type="text" 
            placeholder="e.g. gpt-4-turbo" 
            class="config-input" 
          />
        </div>
      </div>

      <div class="settings-section">
        <h3>🏢 General Preferences</h3>
        <div class="setting-item">
          <div class="info">
            <label>Auto-Analyze Health</label>
            <p>Automatically run diagnostics when a resource error is detected</p>
          </div>
          <input type="checkbox" checked />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  padding: 2.5rem;
  max-width: 900px;
  margin: 0 auto;
  color: #f3f4f6;
  height: 100%;
  overflow-y: auto;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 2rem;
  border-bottom: 1px solid #374151;
  padding-bottom: 1.5rem;
}

.settings-header h1 {
  margin: 0;
  font-size: 2rem;
  font-weight: 800;
  letter-spacing: -0.025em;
}

.settings-header p {
  color: #9ca3af;
  margin: 0.5rem 0 0 0;
}

.save-btn {
  background: linear-gradient(135deg, #2563eb 0%, #1e40af 100%);
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 700;
  font-size: 0.9rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  transition: all 0.2s;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.2);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.success-banner {
  background-color: rgba(16, 185, 129, 0.1);
  border: 1px solid #10b981;
  color: #34d399;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 2rem;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-weight: 600;
  animation: slide-down 0.3s ease-out;
}

.settings-section {
  margin-bottom: 3.5rem;
}

.settings-section h3 {
  font-size: 1rem;
  color: #3b82f6;
  margin-bottom: 1.5rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-weight: 800;
}

.setting-item {
  display: grid;
  grid-template-columns: 1fr 300px;
  align-items: center;
  gap: 2rem;
  padding: 1.5rem 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.setting-item .info label {
  display: block;
  font-weight: 700;
  font-size: 1rem;
  margin-bottom: 0.25rem;
}

.setting-item .info p {
  margin: 0;
  font-size: 0.85rem;
  color: #9ca3af;
}

select, .config-input {
  width: 100%;
  background-color: #1f2937;
  border: 1px solid #374151;
  color: #f3f4f6;
  padding: 8px 12px;
  border-radius: 6px;
  font-family: inherit;
  font-size: 0.9rem;
  outline: none;
  transition: border-color 0.2s;
}

select:focus, .config-input:focus {
  border-color: #3b82f6;
}

.mr-2 { margin-right: 0.5rem; }

@keyframes slide-down {
  from { transform: translateY(-10px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

@keyframes slide-up {
  from { transform: translateY(10px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}
</style>
