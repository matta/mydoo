// public/pwa-glue.js
// Safe "Glue" code to handle Service Worker registration without WASM panics.

window.registerServiceWorker = async () => {
  console.log("[PwaGlue] Checking Service Worker requirements...");

  // We check for feature support first to avoid "undefined" errors in older browsers.
  if (!("serviceWorker" in navigator)) {
    console.warn("[PwaGlue] Service Workers not supported in this browser.");
    return false;
  }

  // Service Workers require a Secure Context (HTTPS or localhost).
  // In some environments, accessing `navigator.serviceWorker` outside of a secure
  // context throws a SecurityError immediately. We guard against this to ensure
  // a graceful degradation.
  if (!window.isSecureContext) {
    console.warn(
      "[PwaGlue] Insecure Context (not HTTPS/localhost). Skipping SW registration to prevent crash.",
    );
    return false;
  }

  try {
    const reg = await navigator.serviceWorker.register("/service-worker.js", {
      scope: "/",
    });
    console.log("[PwaGlue] SW Registered Successfully:", reg);
    return true;
  } catch (e) {
    console.error("[PwaGlue] SW Registration Failed:", e);
    return false;
  }
};

window.subscribeToServiceWorkerStatus = (callback) => {
  if (!("serviceWorker" in navigator)) {
    callback(false);
    return;
  }

  // notify immediately
  callback(!!navigator.serviceWorker.controller);

  // Notify on controller changes. This listener is intentionally never removed;
  // the callback lives for the app's lifetime.
  navigator.serviceWorker.addEventListener("controllerchange", () => {
    callback(!!navigator.serviceWorker.controller);
  });
};
