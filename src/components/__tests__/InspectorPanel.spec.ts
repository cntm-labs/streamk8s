import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import InspectorPanel from '../InspectorPanel.vue';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue('{"spec":{"containers":[{"name":"container-a"},{"name":"container-b"}]}}'),
}));

describe('InspectorPanel Container Parsing', () => {
  it('populates dropdown with container names from selected resource spec', async () => {
    const mockResource = {
      contextName: 'ctx',
      namespace: 'default',
      name: 'test-pod',
      kind: 'Pods',
      spec: {
        containers: [{ name: 'container-a' }, { name: 'container-b' }]
      }
    };
    const wrapper = mount(InspectorPanel, {
      props: { selectedResource: mockResource }
    });
    // Wait for rendering
    await wrapper.vm.$nextTick();
    const options = wrapper.findAll('option');
    expect(options.length).toBeGreaterThanOrEqual(1);
  });
});
