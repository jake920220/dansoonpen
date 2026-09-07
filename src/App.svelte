<script lang="ts">
  import Control from './ui/Control.svelte';
  import Overlay from './ui/Overlay.svelte';
  const params = new URLSearchParams(location.search);
  const view = params.get('view') ?? 'control';
</script>

{#if view === 'overlay'}
  <Overlay displayId={params.get('display') ?? 'preview-display'} />
{:else}
  <Control />
{/if}
