<script setup lang="ts">
// WelcomeView.vue - Professional Welcome Screen for StreamK8s
</script>

<template>
  <div class="welcome-container">
    <div class="welcome-content">
      <div class="logo-section">
        <div class="logo-icon">⎈</div>
        <h1 class="title">StreamK8s</h1>
        <p class="subtitle">The Resilient Bridge between OS & Kubernetes</p>
      </div>

      <div class="shortcuts-section">
        <div class="shortcut-item">
          <span class="action">Search Workloads</span>
          <span class="keys"><kbd>Ctrl</kbd> + <kbd>P</kbd></span>
        </div>
        <div class="shortcut-item">
          <span class="action">Open Settings</span>
          <span class="keys"><kbd>Ctrl</kbd> + <kbd>,</kbd></span>
        </div>
        <div class="shortcut-item">
          <span class="action">Toggle Sidebar</span>
          <span class="keys"><kbd>Ctrl</kbd> + <kbd>B</kbd></span>
        </div>
      </div>

      <div class="footer-tips">
        <p>Tip: Press <kbd>Alt</kbd> to reveal the menu bar</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.welcome-container {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
  background: radial-gradient(circle at center, #111827 0%, #030712 100%);
  color: #f3f4f6;
}

.welcome-content {
  text-align: center;
  max-width: 600px;
  animation: fadeIn 0.8s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.logo-section {
  margin-bottom: 3rem;
}

.logo-icon {
  font-size: 5rem;
  color: #3b82f6;
  margin-bottom: 1rem;
  filter: drop-shadow(0 0 15px rgba(59, 130, 246, 0.3));
}

.title {
  font-size: 3.5rem;
  font-weight: 800;
  margin: 0;
  letter-spacing: -0.05em;
  background: linear-gradient(to right, #3b82f6, #60a5fa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.subtitle {
  font-size: 1.1rem;
  color: #9ca3af;
  margin-top: 0.5rem;
}

.shortcuts-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  background: rgba(255, 255, 255, 0.03);
  padding: 2rem;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(8px);
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 3rem;
}

.action {
  color: #9ca3af;
  font-size: 0.95rem;
}

.keys {
  display: flex;
  gap: 4px;
}

kbd {
  background-color: #374151;
  border: 1px solid #4b5563;
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 0.8rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  color: #e5e7eb;
  box-shadow: 0 2px 0 rgba(0,0,0,0.2);
}

.footer-tips {
  margin-top: 2rem;
  font-size: 0.85rem;
  color: #6b7280;
}
</style>
