import {configureProject} from '../../eslint.config.mjs';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import {defineConfig} from 'eslint/config';

export default defineConfig([
  ...configureProject(import.meta.dirname),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [reactHooks.configs.flat.recommended, reactRefresh.configs.vite],
  },
]);
