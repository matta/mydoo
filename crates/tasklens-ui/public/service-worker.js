// ==============================================================================
// Service Worker for Todo MVP
// ==============================================================================
//
// WHY THIS FILE EXISTS:
// This file runs in the background, separate from the main page. It is responsible
// for:
// 1. Caching assets (so the app works offline). [Coming Soon]
// 2. Managing updates (so the user gets the new version).
// 3. Intercepting network requests.
//
// HOW TO DEBUG:
// - Open Chrome DevTools -> Application -> Service Workers.
// - Check "Update on reload" to force a fresh install on every page load during dev.
// - Check "TodoMVPLogs" in IndexedDB to see persistent logs from this worker.

// ------------------------------------------------------------------------------
// 1. IMPORT VERSION
// ------------------------------------------------------------------------------
// We rely on `build.rs` to generate `version.js` in the public directory.
// This sets a global `BUILD_VERSION` variable.
try {
  importScripts("version.js");
} catch (e) {
  console.warn("version.js missing", e);
}

// ------------------------------------------------------------------------------
// 2. UTILITIES
// ------------------------------------------------------------------------------

/**
 * Returns the current build version, or "unknown" if not set.
 * DRY helper to avoid repeating the fallback logic.
 */
function getVersion() {
  const v = self.__TODO_MVP_BUILD_VERSION__;
  return typeof v === "string" ? v : "unknown";
}

// ------------------------------------------------------------------------------
// 3. LOGGING (IndexedDB)
// ------------------------------------------------------------------------------
// WHY: Console logs in Service Workers can be lost if the worker is killed/restarted.
// IndexedDB provides a durable record of what happened.

const LOG_DB_NAME = "TodoMVPLogs";
const LOG_STORE_NAME = "logs";

/**
 * Writes a message to IndexedDB with a timestamp.
 *
 * HOW:
 * 1. Opens the `TodoMVPLogs` database (version 1).
 * 2. Creates the `logs` store if it doesn't exist (onupgradeneeded).
 * 3. Adds the message object.
 * 4. Catches validation errors silently to avoid crashing the worker.
 */
function logToIDB(type, message) {
  const version = getVersion();
  console.log(`[SW v${version}][${type}] ${message}`);

  const request = indexedDB.open(LOG_DB_NAME, 1);

  request.onupgradeneeded = (event) => {
    const db = event.target.result;
    if (!db.objectStoreNames.contains(LOG_STORE_NAME)) {
      db.createObjectStore(LOG_STORE_NAME, {
        keyPath: "id",
        autoIncrement: true,
      });
    }
  };

  request.onsuccess = (event) => {
    const db = event.target.result;
    try {
      const transaction = db.transaction(LOG_STORE_NAME, "readwrite");
      const store = transaction.objectStore(LOG_STORE_NAME);
      const addRequest = store.add({
        timestamp: new Date().toISOString(),
        version: getVersion(),
        type: type,
        message: message,
      });
      addRequest.onerror = (e) => {
        console.error("Failed to add log entry:", e.target.error);
      };
    } catch (e) {
      console.error("Failed to write log to IDB:", e);
    }
  };

  request.onerror = (e) => {
    console.error("IDB Error:", e.target.error);
  };
}

// ------------------------------------------------------------------------------
// 4. LIFECYCLE: INSTALL
// ------------------------------------------------------------------------------
// WHEN: This fires when the browser sees a new `service-worker.js` (byte difference)
// or a new `BUILD_VERSION` inside `version.js`.

self.addEventListener("install", (_event) => {
  const version = getVersion();
  const msg = `Installing Service Worker v${version}`;
  logToIDB("install", msg);

  // WHY skipWaiting():
  // Normally, a new SW waits in the "waiting" state until all old tabs are closed.
  // For this MVP phase, we want to force the update immediately so we don't
  // have to manually close tabs to see changes.
  // WARNING: This can break apps if they rely on the old versions of API/assets.
  self.skipWaiting();
});

// ------------------------------------------------------------------------------
// 5. LIFECYCLE: ACTIVATE
// ------------------------------------------------------------------------------
// WHEN: This fires after the "waiting" phase is over and this SW becomes the
// active controller.

self.addEventListener("activate", (event) => {
  const msg = `Service Worker v${getVersion()} is now ACTIVE`;
  logToIDB("activate", msg);

  // Take control of all pages immediately.
  event.waitUntil(self.clients.claim());
});

// ------------------------------------------------------------------------------
// 6. MESSAGE HANDLER (Coming in Phase 5.3)
// ------------------------------------------------------------------------------
// FUTURE: Listen for 'SKIP_WAITING' messages from the page to allow
// user-controlled updates. This will replace the automatic skipWaiting() call
// in the install handler above.
//
// self.addEventListener("message", (event) => {
//     if (event.data && event.data.type === "SKIP_WAITING") {
//         self.skipWaiting();
//     }
// });
