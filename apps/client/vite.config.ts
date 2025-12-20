import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig({
  plugins: [
    react(),
    VitePWA({
      // STRATEGY: "generateSW" (default) will generate the service worker file
      // automatically based on the config. This is the simplest way to get
      // precaching and basic PWA functionality. If we needed fine-grained control
      // over network requests or wanted to write our own SW code, we would use
      // "injectManifest".
      strategies: "generateSW",

      // BEHAVIOR: With 'prompt' (and no UI to trigger the update), the new
      // service worker will be installed but remain in the 'waiting' phase. It
      // will specifically NOT take over until all tabs of the application are
      // closed and the browser detects that the old service worker is no longer
      // in use. This results in the "update on next launch" behavior.
      registerType: "prompt",

      includeAssets: ["favicon.ico", "apple-touch-icon.png", "mask-icon.svg"],

      manifest: {
        name: "My Local First App",
        short_name: "LocalApp",
        description: "A local-first Automerge PWA",
        theme_color: "#ffffff",
        icons: [
          {
            src: "pwa-192x192.png",
            sizes: "192x192",
            type: "image/png",
          },
          {
            src: "pwa-512x512.png",
            sizes: "512x512",
            type: "image/png",
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
        enabled: true,
      },
    }),
  ],
});
