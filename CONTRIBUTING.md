# Contributing

## Development Workflow

### Viewing Components in Vitest Browser Mode

For interactive component development and debugging, use Vitest's UI mode:

```bash
vitest --project=browser --ui
```

This opens the Vitest Dashboard where you can view components rendering live in a real browser tab.

### Future: Vitest Workspace Configuration

We plan to implement a split testing workspace to better support WASM and pure logic:

- `--project=logic` — Runs in Node.js (pure logic tests)
- `--project=browser` — Runs in Vitest Browser Mode (component/integration tests)

> **Note:** This is aspirational. Currently, we use a single `turbo test` pipeline with JSDOM.
