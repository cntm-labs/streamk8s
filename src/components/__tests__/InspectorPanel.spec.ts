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
  invoke: vi.fn().mockResolvedValue(''),
}));

describe('InspectorPanel Simplified Diagnostics Redesign', () => {
  it('displays only Logs and Terminal tabs for Pods', () => {
    const wrapper = mount(InspectorPanel, {
      props: {
        selectedResource: {
          contextName: 'kind-cluster',
          namespace: 'default',
          name: 'nginx-pod',
          kind: 'Pods'
        },
        containerName: 'nginx'
      }
    });
    
    const buttons = wrapper.findAll('.tab-btn');
    const tabNames = buttons.map(b => b.text());
    expect(tabNames.some(t => t.includes('Logs'))).toBe(true);
    expect(tabNames.some(t => t.includes('Terminal'))).toBe(true);
    expect(tabNames.some(t => t.includes('YAML'))).toBe(false);
    expect(tabNames.some(t => t.includes('Files'))).toBe(false);
  });

  it('renders terminal tab contents when Terminal is active', async () => {
    const wrapper = mount(InspectorPanel, {
      props: {
        selectedResource: {
          contextName: 'kind-cluster',
          namespace: 'default',
          name: 'nginx-pod',
          kind: 'Pods'
        },
        containerName: 'nginx'
      }
    });
    
    await wrapper.setData({ activeTab: 'Terminal' });
    expect(wrapper.find('.terminal-panel-body').exists()).toBe(true);
  });
});
