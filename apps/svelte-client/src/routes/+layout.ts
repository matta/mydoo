// Disable Server-Side Rendering (SSR) because Automerge Repo relies on browser-specific APIs
// (like Wasm and IndexedDB) and client-side context injection which doesn't work well on the server.
export const ssr = false;
export const prerender = false;
