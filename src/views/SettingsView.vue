<script setup lang="ts">
// SettingsView.vue - Placeholder for Settings
</script>

<template>
  <div class="settings-container">
    <div class="settings-header">
      <h1>Settings</h1>
      <p>Configure StreamK8s environment and preferences</p>
    </div>
    
    <div class="settings-content">
      <div class="settings-section">
        <h3>General</h3>
        <div class="setting-item">
          <label>Theme</label>
          <select disabled>
            <option>Dark (Default)</option>
            <option>Light</option>
            <option>OLED</option>
          </select>
        </div>
        <div class="setting-item">
          <label>Auto-suspend pods</label>
          <input type="checkbox" checked disabled />
        </div>
      </div>

      <div class="settings-section">
        <h3>Kubernetes</h3>
        <div class="setting-item">
          <label>Default Namespace</label>
          <input type="text" value="default" disabled />
        </div>
      </div>

      <div class="placeholder-notice">
        <p>Settings are currently managed via <code>tauri.conf.json</code> and <code>config.rs</code>.</p>
        <p>This UI is a placeholder for Milestone 11.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  padding: 2rem;
  max-width: 800px;
  color: #f3f4f6;
}

.settings-header {
  margin-bottom: 2rem;
  border-bottom: 1px solid #374151;
  padding-bottom: 1rem;
}

.settings-header h1 {
  margin: 0;
  font-size: 1.8rem;
}

.settings-header p {
  color: #9ca3af;
  margin: 0.5rem 0 0 0;
}

.settings-section {
  margin-bottom: 2.5rem;
}

.settings-section h3 {
  font-size: 1.1rem;
  color: #3b82f6;
  margin-bottom: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.setting-item label {
  font-weight: 500;
}

select, input[type="text"] {
  background-color: #1f2937;
  border: 1px solid #374151;
  color: #f3f4f6;
  padding: 4px 8px;
  border-radius: 4px;
}

.placeholder-notice {
  margin-top: 4rem;
  padding: 1rem;
  background-color: rgba(59, 130, 246, 0.1);
  border-radius: 8px;
  border: 1px dashed #3b82f6;
  text-align: center;
  color: #9ca3af;
  font-size: 0.9rem;
}
</style>
