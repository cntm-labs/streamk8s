import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import ResourceDetailModal from '../ResourceDetailModal.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue('{"metadata":{"name":"nginx","namespace":"default","creationTimestamp":"2026-06-25T00:00:00Z"},"spec":{"containers":[{"name":"web"}]}}'),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe('ResourceDetailModal Component', () => {
  it('renders metadata and containers lists', async () => {
    const wrapper = mount(ResourceDetailModal, {
      props: {
        visible: true,
        resource: {
          contextName: 'kind-cluster',
          namespace: 'default',
          name: 'nginx',
          kind: 'Pods'
        }
      }
    });

    await wrapper.vm.$nextTick();
    expect(wrapper.find('.modal-title').text()).toContain('nginx');
  });
});
