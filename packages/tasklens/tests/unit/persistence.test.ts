import {describe, it, expect} from 'vitest';
import {TunnelStore} from '../../src/store';

describe('TunnelStore Persistence', () => {
  it('should save and load state correctly', () => {
    const store = new TunnelStore();
    const task = store.createTask({title: 'Persistent Task'});

    // Save
    const data = store.save();

    // Load
    const loadedStore = TunnelStore.load(data);
    const loadedTask = loadedStore.getTask(task.id);

    expect(loadedTask).toBeDefined();
    expect(loadedTask?.title).toBe('Persistent Task');
    expect(loadedStore.state.nextTaskId).toBe(store.state.nextTaskId);
  });
});
