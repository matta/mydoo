import {beforeEach, describe, expect, it} from 'vitest';

import {TunnelStore} from '../../src/persistence/store';
import {type RepeatConfig, type Task, TaskStatus} from '../../src/types';

describe('TunnelStore', () => {
  let store: TunnelStore;

  beforeEach(() => {
    store = new TunnelStore();
  });

  describe('createTask', () => {
    it('should create a task with default values', () => {
      const task = store.createTask({title: 'Test Task'});

      expect(task.id).toBeDefined();
      expect(task.title).toBe('Test Task');
      expect(task.status).toBe(TaskStatus.Pending);
      expect(task.importance).toBe(1.0);
      expect(task.creditIncrement).toBe(0.5);
      expect(task.credits).toBe(0.0);
      expect(task.desiredCredits).toBe(0.0);
      expect(task.isSequential).toBe(false);
      expect(task.childTaskIds).toEqual([]);
      expect(task.notes).toBe('');
    });

    it('should create a task with provided properties', () => {
      const task = store.createTask({
        title: 'Custom Task',
        importance: 0.8,
        creditIncrement: 2.0,
        status: TaskStatus.Done,
      });

      expect(task.title).toBe('Custom Task');
      expect(task.importance).toBe(0.8);
      expect(task.creditIncrement).toBe(2.0);
      expect(task.status).toBe(TaskStatus.Done);
    });

    it('should initialize task with notes', () => {
      const title = 'Task with notes';
      const notes = 'These are some important notes';
      const newTask = store.createTask({title, notes});

      expect(newTask.title).toBe(title);
      expect(newTask.notes).toBe(notes);
      expect(store.getTask(newTask.id)?.notes).toBe(notes);
    });

    it('should default notes to empty string if not provided', () => {
      const newTask = store.createTask({title: 'No notes'});
      expect(newTask.notes).toBe('');
    });
  });

  describe('updateTask', () => {
    let initialTask: Task;

    beforeEach(() => {
      initialTask = store.createTask({
        title: 'Original Task',
        importance: 0.5,
      });
    });

    it('should update an existing task field', () => {
      const updatedTitle = 'Updated Title';
      const updatedTask = store.updateTask(initialTask.id, {
        title: updatedTitle,
      });

      expect(updatedTask.id).toBe(initialTask.id);
      expect(updatedTask.title).toBe(updatedTitle);
      expect(store.getTask(initialTask.id)?.title).toBe(updatedTitle);
    });

    it('should allow updating notes', () => {
      const newNotes = 'Updated notes';
      const updatedTask = store.updateTask(initialTask.id, {notes: newNotes});

      expect(updatedTask.notes).toBe(newNotes);
      expect(store.getTask(initialTask.id)?.notes).toBe(newNotes);
    });

    it('should allow updating repeatConfig', () => {
      const repeatConfig: RepeatConfig = {frequency: 'weekly', interval: 2};
      const updatedTask = store.updateTask(initialTask.id, {repeatConfig});

      expect(updatedTask.repeatConfig).toEqual(repeatConfig);
      expect(store.getTask(initialTask.id)?.repeatConfig).toEqual(repeatConfig);
    });

    it('should explicitly delete repeatConfig when set to undefined', () => {
      store.updateTask(initialTask.id, {
        repeatConfig: {frequency: 'daily', interval: 1},
      });
      expect(store.getTask(initialTask.id)?.repeatConfig).toBeDefined();

      const updatedTask = store.updateTask(initialTask.id, {
        repeatConfig: undefined,
      });
      expect(updatedTask.repeatConfig).toBeUndefined();
      expect(store.getTask(initialTask.id)?.repeatConfig).toBeUndefined();
      const rawTask = store.state.tasks[initialTask.id];
      if (!rawTask) throw new Error('Task not found');
      expect('repeatConfig' in rawTask).toBe(false);
    });
  });

  describe('deleteTask', () => {
    it('should delete a task and return 1', () => {
      const task = store.createTask({title: 'To Delete'});
      const count = store.deleteTask(task.id);

      expect(count).toBe(1);
      expect(store.getTask(task.id)).toBeUndefined();
    });

    it('should delete descendants (cascade delete)', () => {
      const parent = store.createTask({title: 'Parent'});
      const child = store.createTask({title: 'Child', parentId: parent.id});
      const grandchild = store.createTask({
        title: 'Grandchild',
        parentId: child.id,
      });

      const count = store.deleteTask(parent.id);
      expect(count).toBe(3);
      expect(store.getTask(parent.id)).toBeUndefined();
      expect(store.getTask(child.id)).toBeUndefined();
      expect(store.getTask(grandchild.id)).toBeUndefined();
    });
  });
});
