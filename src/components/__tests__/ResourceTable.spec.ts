import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ResourceTable from '../ResourceTable.vue';

describe('ResourceTable', () => {
  it('renders rows based on props', () => {
    const rows = [{ name: 'pod-1', namespace: 'default', status: 'Running' }];
    const wrapper = mount(ResourceTable, {
      props: { 
        rows, 
        contextName: 'ctx', 
        kind: 'Pod' 
      }
    });
    // Check if the table renders the header + the row
    expect(wrapper.findAll('tr').length).toBe(2);
    expect(wrapper.text()).toContain('pod-1');
  });
});
