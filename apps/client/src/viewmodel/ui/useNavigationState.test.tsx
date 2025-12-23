import type {TaskID} from '@mydoo/tasklens';
import {act, renderHook} from '@testing-library/react';
import {describe, expect, it} from 'vitest';

import {useNavigationState} from './useNavigationState';

describe('useNavigationState', () => {
  it('manages expansion state', () => {
    const {result} = renderHook(() => useNavigationState());

    const id1 = '1' as TaskID;
    const id2 = '2' as TaskID;

    // Initial state
    expect(result.current.isExpanded(id1)).toBe(false);

    // Toggle On
    act(() => result.current.toggleExpanded(id1));
    expect(result.current.isExpanded(id1)).toBe(true);

    // Toggle Off
    act(() => result.current.toggleExpanded(id1));
    expect(result.current.isExpanded(id1)).toBe(false);

    // Expand All
    act(() => result.current.expandAll([id1, id2]));
    expect(result.current.isExpanded(id1)).toBe(true);
    expect(result.current.isExpanded(id2)).toBe(true);

    // Collapse All
    act(() => result.current.collapseAll());
    expect(result.current.expandedIds.size).toBe(0);
  });

  it('manages view path for drill-down', () => {
    const {result} = renderHook(() => useNavigationState());
    const id1 = 'root' as TaskID;
    const id2 = 'sub' as TaskID;

    // Initial
    expect(result.current.currentViewId).toBeUndefined();
    expect(result.current.viewPath).toEqual([]);

    // Push
    act(() => result.current.pushView(id1));
    expect(result.current.currentViewId).toBe(id1);
    expect(result.current.viewPath).toEqual([id1]);

    // Push Nested
    act(() => result.current.pushView(id2));
    expect(result.current.currentViewId).toBe(id2);
    expect(result.current.viewPath).toEqual([id1, id2]);

    // Pop
    act(() => result.current.popView());
    expect(result.current.currentViewId).toBe(id1);

    // Reset
    act(() => result.current.resetView());
    expect(result.current.viewPath).toEqual([]);
  });
});
