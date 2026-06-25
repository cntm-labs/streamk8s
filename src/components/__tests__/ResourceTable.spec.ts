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

  it('emits edit-yaml event when Edit YAML button is clicked', async () => {
    const rows = [{ name: 'pod-1', namespace: 'default', status: 'Running' }];
    const wrapper = mount(ResourceTable, {
      props: { 
        rows, 
        contextName: 'ctx', 
        kind: 'Pod' 
      }
    });

    const editBtn = wrapper.find('.edit-yaml-btn');
    expect(editBtn.exists()).toBe(true);
    
    await editBtn.trigger('click');
    
    const emitted = wrapper.emitted('edit-yaml');
    expect(emitted).toBeTruthy();
    expect(emitted?.[0]).toEqual(['default', 'pod-1', 'Pod']);
  });
});
