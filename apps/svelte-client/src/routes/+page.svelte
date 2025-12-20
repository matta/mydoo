<script lang="ts">
  console.log("[Page] Initializing script...");
  import { onMount } from "svelte";
  import { page } from "$app/state"
  import { goto } from "$app/navigation"
  import { getRepo } from "$lib/db";

  // Dynamic Import Variables
  let documentFunc: any;

  interface TodoList {
    todos: { [id: string]: TodoItem }
    todoOrder: string[]
  }

  interface TodoItem {
    title: string
    done: boolean
    children: TodoList
  }

  // The root document is effectively a TodoList
  interface TodoDoc extends TodoList {}

  // Initialize state
  let docStore = $state<any>(null) // Value of the store (the doc)
  let docHandle: any = null; // The store object itself (contains subscribe, change)

  let loading = $state(true)
  let error = $state<Error | null>(null)
  
  // Navigation State
  let viewPath = $state<string[]>([]) // Path of IDs to the current displayed root
  let expandedIds = $state<Set<string>>(new Set())

  // Edit State
  let editingId = $state<string | null>(null)
  let editingTitle = $state("") 

  onMount(async () => {
     try {
        /**
         * ARCHITECTURE NOTE: DYNAMIC IMPORTS
         * ----------------------------------
         * We use `import(...)` here instead of top-level static imports.
         * 
         * On certain mobile environments (specifically iPhone in non-secure HTTP contexts),
         * statically importing the `@automerge` libraries caused the entire JavaScript 
         * bundle to crash immediately upon load with a "ReferenceError". This was likely 
         * due to issues with WebAssembly initialization or Top-Level Await in that specific 
         * browser context preventing the app from even hydrating.
         * 
         * By moving these imports into `onMount`, we ensure:
         * 1. The minimal Svelte app shells loads and renders immediately.
         * 2. The heavy/risky libraries are loaded lazily. 
         * 3. If they fail, we can catch the error and show it in the UI, rather than white-screening.
         */
        console.log("[Page] Dynamically importing Automerge Store...");
        const [mod, repo] = await Promise.all([
          import("@automerge/automerge-repo-svelte-store"),
          getRepo()
        ]);
        
        documentFunc = mod.document;
        console.log("[Page] Modules and Repo loaded.");

        await loadDocument(repo);
     } catch (e) {
        console.error("[Page] Init failed:", e);
        error = e as Error;
        loading = false;
     }
  });

  async function loadDocument(repo: any) {
      try {
        loading = true
        let url = page.url.searchParams.get('docUrl');

        if (!url && repo) {
          console.log("[Page] Creating new document...");
          const handle = repo.create();
          handle.change((d: any) => {
            d.todos = {}
            d.todoOrder = []
          });
          url = handle.url;
          await goto(`?docUrl=${url}`, { replaceState: true });
        }

        if (url) {
          console.log("[Page] Loading document URL:", url);
          
          // CRITICAL FIX: PROMISE HANDLING
          // In most environments, `document(url, repo)` returns the Store object synchronously.
          // However, due to the way Vite/Rollup bundles the dynamic import chunks for this 
          // specific configuration, the function appears to return a Promise that resolves 
          // to the Store.
          //
          // We explicit check for `instanceof Promise` and `await` it to handle both behavior 
          // (sync or async) safely, ensuring we don't try to call `.subscribe` on a Promise.
          let result = documentFunc(url, repo);
          if (result instanceof Promise) {
              console.log("[Page] Awaiting promise from documentFunc...");
              result = await result;
          }
          
          docHandle = result;
          
          if (!docHandle || typeof docHandle.subscribe !== 'function') {
             throw new Error("docHandle is not a store! It is: " + typeof docHandle);
          }

          // Manual subscription to update our local 'docStore' state
          docHandle.subscribe((d: any) => {
             docStore = d;
          });
        }
        loading = false
      } catch (err) {
        console.error("[Page] Failed to load document:", err)
        error = err as Error
        loading = false
      }
  }

  // Helper to find list at path
  function getList(doc: TodoDoc, path: string[]): TodoList | null {
    if (!doc) return null;
    let current: TodoList = doc
    for (const id of path) {
      if (!current.todos || !current.todos[id]) {
        return null
      }
      current = current.todos[id].children
    }
    return current
  }

  function addTodo(path: string[], title: string) {
    if (!title.trim() || !docHandle) return
    
    // Polyfilled crypto check for mobile compatibility
    const id = self.crypto.randomUUID ? self.crypto.randomUUID() : Math.random().toString(36).substr(2);
    const safeTitle = title.trim()

    docHandle.change((doc: any) => {
      // Manual traversal to ensure 'children' objects exist or are created
      let list: TodoList = doc
      
      for (const stepId of path) {
        if (!list.todos[stepId]) return // Parent item missing, abort
        
        const item = list.todos[stepId]
        if (!item.children) {
           item.children = { todos: {}, todoOrder: [] }
        }
        list = item.children
      }

      // Initialize if needed
      if (!list.todos) list.todos = {}
      if (!list.todoOrder) list.todoOrder = []

      // Create new item with empty children list
      list.todos[id] = { 
        title: safeTitle, 
        done: false,
        children: { todos: {}, todoOrder: [] }
      }
      list.todoOrder.unshift(id)
    })
  }

  // Recursive check for "can mark done"
  function canMarkDone(item: TodoItem): boolean {
    if (!item.children || !item.children.todoOrder || item.children.todoOrder.length === 0) return true
    
    // Check if ALL children are done
    return item.children.todoOrder.every(childId => {
      const child = item.children.todos[childId]
      return child?.done
    })
  }

  function toggleTodo(path: string[], id: string) {
    if (!docHandle) return
    docHandle.change((doc: any) => {
      const list = getList(doc, path)
      if (!list || !list.todos[id]) return

      const item = list.todos[id]
      
      // Enforce Invariant: Cannot mark done if children are not done
      if (!item.done && !canMarkDone(item)) {
        // Maybe show alert or shake UI? For now just return.
        return 
      }

      item.done = !item.done
    })
  }

  function startEditing(id: string, currentTitle: string) {
    editingId = id
    editingTitle = currentTitle
  }

  function saveEdit(path: string[]) {
    if (!editingId || !docHandle) return
    const id = editingId
    const title = editingTitle.trim()
    
    if (title) {
       docHandle.change((doc: any) => {
        const list = getList(doc, path)
        if (list && list.todos[id]) {
          list.todos[id].title = title
        }
      })
    }
    
    cancelEdit()
  }

  function cancelEdit() {
    editingId = null
    editingTitle = ""
  }

  // View Navigation Logic
  let viewRootList = $derived.by(() => {
    if (!docStore) return null
    return getList(docStore, viewPath)
  })

  // Derive the path of titles for breadcrumbs
  let breadcrumbs = $derived.by(() => {
    if (!docStore) return []
    const crumbs: { id: string, title: string, path: string[] }[] = []
    
    // Always start with Root
    crumbs.push({ id: 'root', title: 'Root', path: [] })

    let currentPath: string[] = []
    let currentList = docStore
    
    for (const id of viewPath) {
      if (!currentList.todos || !currentList.todos[id]) break
      const item = currentList.todos[id]
      currentPath = [...currentPath, id]
      crumbs.push({ id, title: item.title, path: currentPath })
      currentList = item.children
    }
    return crumbs
  })

  function toggleExpand(fullPath: string[]) {
    const id = fullPath[fullPath.length - 1]
    
    // Toggle expansion
    if (expandedIds.has(id)) {
      expandedIds.delete(id)
      expandedIds = new Set(expandedIds) // Trigger reactivity
    } else {
      expandedIds.add(id)
      expandedIds = new Set(expandedIds)
      
      // Zoom Logic: Check depth relative to viewRoot
      // fullPath includes the item ID itself.
      // viewPath is the path to the current root list.
      // Depth = fullPath.length - viewPath.length - 1 (0-indexed)
      // Example: View=[], Item=[A]. Depth = 1 - 0 - 1 = 0.
      // Item=[A, B, C]. Depth = 3 - 0 - 1 = 2.
      
      const depth = fullPath.length - viewPath.length - 1
      
      // If we are expanding an item at Depth 2 (the 3rd visible level),
      // we need to shift the view so this item becomes depth 1.
      // Original: 0, 1, 2. Expand 2 -> show 3.
      // New View: 1, 2, 3.
      // So we want to push the first undefined component of path to viewPath?
      // Actually we want the new View Root to be the parent of the item at previous level 1.
      // Which is the item at level 0 relative to old view.
      
      if (depth >= 2) {
        // Shift view down by one level
        const nextRootId = fullPath[viewPath.length]
        viewPath = [...viewPath, nextRootId]
      }
    }
  }

  function navigate(path: string[]) {
    viewPath = path
    // Optional: Prune expandedIds that are no longer visible? Not strictly necessary.
  }


  function cleanup() {
    docHandle?.change((doc: any) => {
      cleanupList(doc)
    })
  }

  function cleanupList(list: TodoList) {
    if (!list.todos || !list.todoOrder) return
    
    // 1. Clean children recursively first (depth-first)
    // We iterate over a copy or just access by ID to avoid modification during iteration issues,
    // though we are modifying the map/order separately.
    for (const id of list.todoOrder) {
      if (list.todos[id]) {
        cleanupList(list.todos[id].children)
      }
    }

    // 2. Identify done items in this list
    const doneIds = new Set<string>()
    for (const [id, item] of Object.entries(list.todos)) {
      if (item.done) doneIds.add(id)
    }

    // 3. Delete done items
    // If a parent is done, its children are implicitly removed too.
    for (const id of doneIds) {
      delete list.todos[id]
    }
    
    // 4. Update Order
    const newOrder = list.todoOrder.filter(id => !doneIds.has(id))
    list.todoOrder.splice(0, list.todoOrder.length, ...newOrder)
  }


</script>

<style>
  .todo-app {
    max-width: 600px;
    margin: 2rem auto;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    padding: 1rem;
  }

  .breadcrumbs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 2rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .breadcrumb {
    cursor: pointer;
    color: #666;
    font-weight: 500;
  }

  .breadcrumb:hover {
    color: #000;
    text-decoration: underline;
  }

  .breadcrumb-separator {
    color: #ccc;
  }

  .breadcrumb:last-child {
    color: #000;
    font-weight: bold;
    cursor: default;
    text-decoration: none;
  }

  .todo-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .nested-list {
    margin-left: 1.5rem;
    padding-left: 0.5rem;
    border-left: 1px solid #eee;
  }

  .todo-item-container {
    margin-bottom: 0.5rem;
  }

  .todo-row {
    display: flex;
    align-items: center;
    padding: 0.5rem;
    background: white;
    border-radius: 4px;
    gap: 0.5rem;
  }
  
  .todo-row:hover {
    background: #f9f9f9;
  }

  .checkbox {
    cursor: pointer;
    width: 1.2rem;
    height: 1.2rem;
  }
  
  .todo-title {
    flex: 1;
    cursor: pointer;
    padding: 2px 4px; 
    font-size: 1rem;
  }

  .todo-title.done {
    text-decoration: line-through;
    opacity: 0.6;
  }

  .edit-input {
    flex: 1;
    padding: 4px;
    font-size: 1rem;
    border: 1px solid #ccc;
    border-radius: 4px;
  }

  .expand-btn {
    background: none;
    border: none;
    cursor: pointer;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
    font-size: 0.8rem;
    border-radius: 4px;
  }

  .expand-btn:hover {
    background: #eee;
    color: #000;
  }

  .add-row {
     margin-top: 0.5rem;
     padding: 0.5rem;
     display: flex;
     align-items: center;
  }

  .add-btn {
    background: none;
    border: 1px dashed #ccc;
    color: #666;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    width: 100%;
    text-align: left;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .add-btn:hover {
    border-color: #999;
    color: #000;
  }
</style>

{#snippet todoList(list: TodoList, path: string[], depth: number, isParentDone: boolean = false)}
  <ul class="todo-list" class:nested-list={depth > 0}>
    {#if list && list.todoOrder}
      {#each list.todoOrder as id (id)}
        {@const item = list.todos[id]}
        
        {#if item}
          {@const fullPath = [...path, id]}
          {@const isExpanded = expandedIds.has(id)}
          {@const hasChildren = item.children && item.children.todoOrder && item.children.todoOrder.length > 0}
          {@const canCheck = canMarkDone(item)}

          <li class="todo-item-container">
            <div class="todo-row">
              <!-- Expand Button -->
                 <button 
                class="expand-btn" 
                onclick={() => toggleExpand(fullPath)}
                aria-label={isExpanded ? "Collapse" : "Expand"}
                 >
                {#if isExpanded}
                  ▼
                {:else if hasChildren}
                  ▶
              {:else}
                  <span style="opacity: 0.3">○</span>
              {/if}
              </button>

              <!-- Checkbox -->
              <input 
                type="checkbox" 
                class="checkbox"
                checked={item.done}
                disabled={!item.done && !canCheck}
                title={!item.done && !canCheck ? "Complete all children first" : ""}
                onchange={() => toggleTodo(path, id)}
              />
              
              <!-- Title / Edit -->
              {#if editingId === id}
                <input 
                  type="text" 
                  class="edit-input"
                  bind:value={editingTitle}
                  autofocus
                  onblur={() => saveEdit(path)}
                  onkeydown={(e) => {
                     if (e.key === 'Enter') saveEdit(path);
                     if (e.key === 'Escape') cancelEdit();
                     e.stopPropagation();
                  }}
                />
              {:else}
                <span 
                  class="todo-title"
                  class:done={item.done}
                  role="button"
                  tabindex="0"
                  onclick={() => startEditing(id, item.title)}
                  onkeydown={(e) => e.key === 'Enter' && startEditing(id, item.title)}
                >
                  {item.title}
                </span>
              {/if}
            </div>

            <!-- Recursive Children -->
            {#if isExpanded && depth < 2} 
              {@render todoList(item.children, fullPath, depth + 1, item.done)}
            {/if}
          </li>
        {/if}
      {/each}
    {/if}

    <!-- Add Item Button for this level -->
    <!-- Only show if parent is NOT done and list exists (meaning parent exists or it is root) -->
    <!-- Root (depth 0) is never 'Done' in this context, so default false works. -->
    {#if !isParentDone && list}
      <li class="add-row">
        <button class="add-btn" onclick={() => {
          const title = prompt("New Task Title")
          if (title) addTodo(path, title)
        }}>
          + Add Item
        </button>
      </li>
    {/if}
  </ul>
{/snippet}

<div class="todo-app">
  {#if loading}
    <p>Loading document...</p>
  {:else if error}
    <p style="color: red">Error: {error.message}</p>
  {:else if viewRootList}
    <!-- Breadcrumbs -->
    <div class="breadcrumbs">
      {#each breadcrumbs as crumb, i}
        {#if i > 0}
          <span class="breadcrumb-separator">/</span>
        {/if}
        <button 
          class="breadcrumb" 
          onclick={() => navigate(crumb.path)}
        >
          {crumb.title}
        </button>
      {/each}
    </div>

    <!-- Recursive List Root -->
    {@render todoList(viewRootList, viewPath, 0, false)}

    <div style="margin-top: 2rem; border-top: 1px solid #eee; padding-top: 1rem;">
      <button class="cleanup" onclick={cleanup}>
        Cleanup Completed Tasks
      </button>
    </div>

  {:else}
     <p>Initializing...</p>
  {/if}
</div>