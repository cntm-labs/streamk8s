import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import InspectorPanel from '../InspectorPanel.vue';

// Mock matchMedia for jsdom
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock getContext for canvas
HTMLCanvasElement.prototype.getContext = vi.fn().mockReturnValue({
  fillRect: vi.fn(),
  clearRect: vi.fn(),
  getImageData: vi.fn(),
  putImageData: vi.fn(),
  createImageData: vi.fn(),
  setTransform: vi.fn(),
  drawImage: vi.fn(),
  save: vi.fn(),
  fillText: vi.fn(),
  restore: vi.fn(),
  beginPath: vi.fn(),
  moveTo: vi.fn(),
  lineTo: vi.fn(),
  closePath: vi.fn(),
  stroke: vi.fn(),
  translate: vi.fn(),
  scale: vi.fn(),
  rotate: vi.fn(),
  arc: vi.fn(),
  fill: vi.fn(),
  measureText: vi.fn().mockReturnValue({ width: 0 }),
  transform: vi.fn(),
  rect: vi.fn(),
  clip: vi.fn(),
});

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

  it('renders terminal tab contents when Terminal is clicked', async () => {
    const mockResource = {
      contextName: 'ctx',
      namespace: 'default',
      name: 'test-pod',
      kind: 'Pods'
    };
    const wrapper = mount(InspectorPanel, {
      props: { selectedResource: mockResource }
    });
    // Set tab to Terminal
    await wrapper.setData({ activeTab: 'Terminal' });
    expect(wrapper.find('.terminal-panel-body').exists()).toBe(true);
  });
});
