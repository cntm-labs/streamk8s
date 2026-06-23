import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import AdviceBanner from '../AdviceBanner.vue';
import AdvisorToast from '../AdvisorToast.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve('Namespace default suspended.')),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('AdviceBanner & Optimization flow', () => {
  it('emits optimize event when clicking optimize button', async () => {
    const wrapper = mount(AdviceBanner, {
      props: {
        advice: { action: 'Suspend', reason: 'High load' }
      }
    });
    
    await wrapper.find('.optimize-btn').trigger('click');
    expect(wrapper.emitted('optimize')).toBeTruthy();
  });
});

describe('AdvisorToast Namespace Prop Check', () => {
  it('uses the provided namespace prop instead of default', async () => {
    const wrapper = mount(AdvisorToast, {
      props: {
        contextName: 'docker-desktop',
        namespace: 'custom-namespace'
      }
    });
    
    // Force set state to visible to render button
    (wrapper.vm as any).visible = true;
    (wrapper.vm as any).detectedApp = 'cyberpunk';
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    expect(invoke).toHaveBeenCalledWith('suspend_namespace', {
      contextName: 'docker-desktop',
      namespace: 'custom-namespace'
    });
  });
});
