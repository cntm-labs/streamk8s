import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import App from './App.vue';
import WelcomeView from './views/WelcomeView.vue';
import ActivityBar from './components/ActivityBar.vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('App.vue Threshold Events', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    
    global.ResizeObserver = vi.fn().mockImplementation(() => ({
      observe: vi.fn(),
      unobserve: vi.fn(),
      disconnect: vi.fn(),
    }));

    // Default mock implementation for invoke
    (invoke as any).mockImplementation((cmd: string) => {
      if (cmd === 'get_available_contexts') {
        return Promise.resolve([{ name: 'test-cluster', is_current: true }]);
      }
      if (cmd === 'get_namespaces') {
        return Promise.resolve(['default']);
      }
      if (cmd.startsWith('get_')) {
        return Promise.resolve([]);
      }
      return Promise.resolve();
    });
  });

  it('renders advice banner when hardware-threshold-exceeded is emitted', async () => {
    let thresholdExceededCallback: Function | null = null;
    let thresholdRecoveredCallback: Function | null = null;
    
    (listen as any).mockImplementation((event: string, callback: Function) => {
      if (event === 'hardware-threshold-exceeded') {
        thresholdExceededCallback = callback;
      }
      if (event === 'hardware-threshold-recovered') {
        thresholdRecoveredCallback = callback;
      }
      return Promise.resolve(() => {});
    });

    const wrapper = mount(App);

    // Wait for onMounted to complete
    await flushPromises();

    // Verify callbacks were registered
    expect(thresholdExceededCallback).toBeTruthy();
    expect(thresholdRecoveredCallback).toBeTruthy();

    const activityBar = wrapper.findComponent(ActivityBar);
    if (activityBar.exists()) {
      activityBar.vm.$emit('update:activeId', 'settings');
      await flushPromises();
      activityBar.vm.$emit('update:activeId', 'explorer');
      await flushPromises();
    }

    // Simulate threshold exceeded event
    if (thresholdExceededCallback) {
      await thresholdExceededCallback({ payload: {} });
    }
    
    await flushPromises();

    // Check that the advice reason is displayed
    expect(wrapper.text()).toContain('Hardware threshold exceeded');

    // Simulate recovery
    if (thresholdRecoveredCallback) {
      await thresholdRecoveredCallback({ payload: {} });
    }

    await flushPromises();

    // Banner should be gone
    expect(wrapper.text()).not.toContain('Hardware threshold exceeded');
  });
});
