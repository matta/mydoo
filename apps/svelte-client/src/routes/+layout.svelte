<script lang="ts">
	console.log("[Layout] Initializing script...");
	import favicon from '$lib/assets/favicon.svg';
	import { browser } from "$app/environment";
	import { getRepo } from "$lib/db";
	import { onMount } from "svelte";
	
	onMount(async () => {
		if (browser) {
			try {
				// Just trigger init so it starts connecting earlier
				await getRepo();
				console.log("[Layout] DB Initialized via Singleton.");
			} catch (err) {
				console.error("[Layout] DB Init Failed:", err);
			}
		}
	});

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
