import {execSync} from 'node:child_process';
import path from 'node:path';
import react from '@vitejs/plugin-react';
import type {PluginOption} from 'vite';
import {defineConfig} from 'vite';
import {VitePWA} from 'vite-plugin-pwa';
import topLevelAwait from 'vite-plugin-top-level-await';
import wasm from 'vite-plugin-wasm';

const getBuildInfo = () => {
  // 1. Cloudflare Pages (Environment Variable)
  const cfSha = process.env.CF_PAGES_COMMIT_SHA;
  if (cfSha) {
    return {
      hash: cfSha.substring(0, 7),
      date: new Date().toISOString(), // Use build time as proxy
      clean: true, // Assume CI builds are clean
    };
  }

  // 2. Local Git (Plumbing Commands)
  try {
    const hash = execSync('git rev-parse --short HEAD', {
      stdio: ['ignore', 'pipe', 'ignore'],
      encoding: 'utf-8',
    }).trim();

    // Returns "<timestamp> <hash>"
    const timestampOutput = execSync('git rev-list -1 --timestamp HEAD', {
      stdio: ['ignore', 'pipe', 'ignore'],
      encoding: 'utf-8',
    }).trim();

    const parts = timestampOutput.split(' ');
    const timestampStr = parts[0];
    if (!timestampStr) {
      throw new Error('Unexpected git output: no timestamp found');
    }
    const timestamp = parseInt(timestampStr, 10);
    const date = new Date(timestamp * 1000).toISOString();

    // Returns exit code 1 if dirty, 0 if clean.
    // We use try/catch because execSync throws on non-zero exit code.
    let clean = true;
    try {
      execSync('git diff-index --quiet HEAD --', {
        stdio: ['ignore', 'ignore', 'ignore'],
      });
    } catch {
      clean = false;
    }

    return {hash, date, clean};
  } catch (error) {
    console.error('Failed to get build info from git:', error);
    return {
      hash: 'unknown',
      date: new Date().toISOString(),
      clean: false,
    };
  }
};

const buildInfo = getBuildInfo();

export default defineConfig({
  define: {
    __BUILD_INFO__: JSON.stringify(buildInfo),
  },
  resolve: {
    dedupe: [
      '@automerge/automerge-repo-react-hooks',
      'react',
      'react-dom',
      'react-redux',
    ],
    alias: {
      '@mydoo/tasklens': path.resolve(__dirname, '../../packages/tasklens/src'),
      '@automerge/automerge-repo-react-hooks': path.resolve(
        __dirname,
        'node_modules/@automerge/automerge-repo-react-hooks',
      ),
    },
  },
  plugins: [
    react(),
    wasm(),
    topLevelAwait(),
    VitePWA({
      // STRATEGY: "generateSW" (default) will generate the service worker file
      // automatically based on the config. This is the simplest way to get
      // precaching and basic PWA functionality. If we needed fine-grained control
      // over network requests or wanted to write our own SW code, we would use
      // "injectManifest".
      strategies: 'generateSW',

      // BEHAVIOR: With 'prompt' (and no UI to trigger the update), the new
      // service worker will be installed but remain in the 'waiting' phase. It
      // will specifically NOT take over until all tabs of the application are
      // closed and the browser detects that the old service worker is no longer
      // in use. This results in the "update on next launch" behavior.
      registerType: 'prompt',

      includeAssets: ['favicon.ico', 'apple-touch-icon.png', 'mask-icon.svg'],

      manifest: {
        name: 'My Local First App',
        short_name: 'LocalApp',
        description: 'A local-first Automerge PWA',
        theme_color: '#ffffff',
        icons: [
          {
            src: 'pwa-192x192.png',
            sizes: '192x192',
            type: 'image/png',
          },
          {
            src: 'pwa-512x512.png',
            sizes: '512x512',
            type: 'image/png',
          },
        ],
      },

      devOptions: {
        // DEVELOPMENT NOTE:
        // Setting this to true enables the Service Worker in development mode.
        // extremely useful for testing PWA features (like installation) locally.
        //
        // HOWEVER, NOTE ON HOT RELOAD (HMR):
        // Service Workers do NOT support HMR. If you change the service worker
        // configuration or code, you usually need to manually reload the page
        // or hard-refresh to see changes. The "normal" Vite HMR for your React
        // components still works fine, but the SW itself is a separate entity
        // that lives outside the HMR cycle.
        enabled: process.env.VITE_PWA_DEV === 'true',
      },
      workbox: {
        maximumFileSizeToCacheInBytes: 4 * 1024 * 1024, // 4MB
      },
    }),
    // TODO: Remove cast when vite-plugin-pwa ships better types
  ] as PluginOption[],
  server: {
    // Allows any host to access the dev server, which is required for using
    // tunnels like Tailscale/ngrok where the hostname is not known in advance.
    allowedHosts: true,
  },
});
