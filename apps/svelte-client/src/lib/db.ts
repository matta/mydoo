import { browser } from '$app/environment';

/**
 * ARCHITECTURE NOTE: ASYNC SINGLETON PATTERN
 *
 * Why not use Svelte Context (setContext/getContext)?
 * --------------------------------------------------
 * Svelte's context API is synchronous and strictly bound to the component initialization phase.
 * However, we are forced to initialize the Automerge Repo *asynchronously* (via dynamic imports)
 * to prevent the application from crashing on certain mobile browsers (see +page.svelte).
 *
 * Because the Repo is created after the component has mounted (outside the init phase),
 * we cannot use `setContext`. Therefore, we use this Singleton pattern to:
 * 1. Lazily load the heavy Automerge dependencies only when needed.
 * 2. Ensure only one Repo instance is ever created (shared between Layout and Page).
 * 3. Provide safe async access to the Repo from anywhere in the app.
 */

let repoPromise: Promise<any> | null = null;
let repoInstance: any = null;

export async function getRepo() {
	if (!browser) return null;
	if (repoInstance) return repoInstance;
	if (repoPromise) return repoPromise;

	repoPromise = (async () => {
		try {
			console.log('[DB] Dynamically importing Automerge...');
			const [{ Repo }, { IndexedDBStorageAdapter }, { BrowserWebSocketClientAdapter }] =
				await Promise.all([
					import('@automerge/automerge-repo'),
					import('@automerge/automerge-repo-storage-indexeddb'),
					import('@automerge/automerge-repo-network-websocket')
				]);

			console.log('[DB] Initializing Repo...');
			repoInstance = new Repo({
				network: [new BrowserWebSocketClientAdapter('wss://sync.automerge.org')],
				storage: new IndexedDBStorageAdapter()
			});
			console.log('[DB] Repo Ready.');
			return repoInstance;
		} catch (e) {
			console.error('[DB] Failed to init:', e);
			throw e;
		}
	})();

	return repoPromise;
}
