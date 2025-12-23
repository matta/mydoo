import {ANYWHERE_PLACE_ID, type PlaceID} from '@mydoo/tasklens';
import {describe, expect, it} from 'vitest';

import {createTask} from '../../src/persistence/ops';
import {TunnelStore} from '../../src/persistence/store';

describe('createTask - Default Values and Inheritance', () => {
  it('should set placeId to ANYWHERE_PLACE_ID for root tasks', () => {
    const store = new TunnelStore();
    const task = createTask(store.state, {title: 'Root Task'});

    expect(task.placeId).toBe(ANYWHERE_PLACE_ID);
  });

  it('should inherit placeId from parent task', () => {
    const store = new TunnelStore();
    const customPlaceId = 'Home' as PlaceID;

    // Create parent with custom placeId
    const parent = createTask(store.state, {
      title: 'Parent Task',
      placeId: customPlaceId,
    });

    // Create child without specifying placeId
    const child = createTask(store.state, {
      title: 'Child Task',
      parentId: parent.id,
    });

    expect(child.placeId).toBe(customPlaceId);
  });

  it('should allow explicit placeId override even with parent', () => {
    const store = new TunnelStore();
    const parentPlaceId = 'Home' as PlaceID;
    const childPlaceId = 'Office' as PlaceID;

    const parent = createTask(store.state, {
      title: 'Parent Task',
      placeId: parentPlaceId,
    });

    const child = createTask(store.state, {
      title: 'Child Task',
      parentId: parent.id,
      placeId: childPlaceId,
    });

    expect(child.placeId).toBe(childPlaceId);
  });

  it('should default to ANYWHERE_PLACE_ID if parent has no placeId', () => {
    const store = new TunnelStore();

    // Create parent without placeId (will get ANYWHERE_PLACE_ID)
    const parent = createTask(store.state, {title: 'Parent Task'});

    // Create child
    const child = createTask(store.state, {
      title: 'Child Task',
      parentId: parent.id,
    });

    expect(child.placeId).toBe(ANYWHERE_PLACE_ID);
  });

  it('should set importance to 1.0 by default', () => {
    const store = new TunnelStore();
    const task = createTask(store.state, {title: 'Test Task'});

    expect(task.importance).toBe(1.0);
  });

  it('should set status to Pending by default', () => {
    const store = new TunnelStore();
    const task = createTask(store.state, {title: 'Test Task'});

    expect(task.status).toBe('Pending');
  });

  it('should initialize childTaskIds as empty array', () => {
    const store = new TunnelStore();
    const task = createTask(store.state, {title: 'Test Task'});

    expect(task.childTaskIds).toEqual([]);
  });

  it('should add child to parent childTaskIds array', () => {
    const store = new TunnelStore();

    const parent = createTask(store.state, {title: 'Parent'});
    const child = createTask(store.state, {
      title: 'Child',
      parentId: parent.id,
    });

    const updatedParent = store.state.tasks[parent.id];
    expect(updatedParent?.childTaskIds).toContain(child.id);
  });
});
