import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    globals: true,
    setupFiles: [], // Add setup files if needed
    coverage: {
      provider: "v8", // or 'istanbul'
      reporter: ["text", "json", "html"],
      include: ["src/**/*.ts"],
      exclude: ["src/index.ts", "src/**/*.d.ts"],
    },
    // For ESM support in NodeNext, Vitest usually handles it well
    // but if issues arise, consider 'alias' or 'resolve.conditions'
    exclude: ["dist/**", "node_modules/**"],
  },
});
