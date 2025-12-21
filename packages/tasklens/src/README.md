# TaskLens (TypeScript Core)

The core algorithm and data store implementation for the TaskLens prioritization engine, written in
TypeScript.

## Features

- **Dynamic Prioritization**: Implements the 7-Pass Tunnel Algorithm for
  calculating task priority.
- **Automerge Backing**: Uses `automerge` CRDT for local-first data storage and
  synchronization.
- **Strict Typing**: Fully typed with TypeScript matching the `ALGORITHM.md`
  specification.

## Installation

```bash
pnpm add @mydoo/tasklens
```

## Usage

### Basic Example

```typescript
import {TunnelStore} from '@mydoo/tasklens';

// 1. Initialize Store
const store = new TunnelStore();

// 2. Create Data
const rootGoal = store.createTask({
  title: 'Work',
  desiredCredits: 100,
});

const task = store.createTask({
  title: 'Email',
  parentId: rootGoal.id,
  creditIncrement: 1.0,
});

// 3. Update Priorities
store.recalculateScores({placeId: 'All'});

// 4. Get Todo List
const todos = store.getTodoList({currentTime: Date.now()});
console.log(todos.map(t => `${t.title}: ${t.priority}`));
```

### Persistence

```typescript
// Save to Uint8Array
const data = store.save();

// Load from Uint8Array
const loadedStore = TunnelStore.load(data);
```

## Development

- **Build**: `pnpm tsc`
- **Test**: `make test` (Runs Vitest)
- **Lint**: `make check` (Runs ESLint and TSC)
