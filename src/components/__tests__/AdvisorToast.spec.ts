import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import AdviceBanner from '../AdviceBanner.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve('Namespace default suspended.')),
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
