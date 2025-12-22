import {describe, expect, it} from 'vitest';

import {TunnelStateSchema} from '../../src/persistence/schemas';
import {TunnelStore} from '../../src/persistence/store';

describe('Schema Validation', () => {
  it('should validate Automerge proxy objects correctly', () => {
    // 1. Initialize store (uses Automerge)
    const store = new TunnelStore();

    // 2. Validate initial state
    // Automerge proxy objects contain internal symbols that must be handled by the schema
    let validation = TunnelStateSchema.safeParse(store.state);
    expect(validation.success).toBe(true);

    // 3. Add a task to modify the state
    store.createTask({title: 'New Task'});

    // 4. Validate state after modification
    validation = TunnelStateSchema.safeParse(store.state);
    expect(validation.success).toBe(true);
  });
});
