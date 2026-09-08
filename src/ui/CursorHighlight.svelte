<script lang="ts">
  import { onMount } from 'svelte';
  import { bridge, native } from '../app/bridge';
  import type { CursorFrame, CursorSettings } from '../shared/types';
  let { displayId, settings, reduceMotion }: { displayId: string; settings: CursorSettings; reduceMotion: boolean } = $props();
  let halo: HTMLDivElement;
  let ripple: HTMLDivElement;
  let animation: Animation | undefined;
  let previous: CursorFrame | null = null;
  function accept(frame: CursorFrame | null) {
    if (!frame || frame.displayId !== displayId || (previous && frame.sequence <= previous.sequence)) return;
    halo.style.visibility = frame.visible ? 'visible' : 'hidden';
    halo.style.transform = `translate(${frame.x}px, ${frame.y}px)`;
    if (!frame.visible) { animation?.cancel(); }
    else if (previous && frame.clicks !== previous.clicks && frame.clicks !== 0 && settings.showClicks) {
      animation?.cancel();
      ripple.style.left = `${frame.x}px`; ripple.style.top = `${frame.y}px`;
      animation = ripple.animate([
        { opacity: 0.9, transform: 'translate(-50%, -50%) scale(0.8)' },
        { opacity: 0, transform: `translate(-50%, -50%) scale(${reduceMotion ? 0.8 : 1.7})` },
      ], { duration: reduceMotion ? 150 : 420, easing: 'ease-out' });
    }
    previous = frame;
  }
  onMount(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    void (async () => {
      const off = await bridge.onCursor((frame) => { if (!disposed) accept(frame); });
      if (disposed) { off(); return; }
      stop = off;
      const initial = await bridge.getCursor();
      if (!disposed) accept(initial);
    })().catch(() => { halo.style.visibility = 'hidden'; });
    let clicks = 0; let sequence = 0;
    function preview(event: PointerEvent) {
      if (event.type === 'pointerdown' && event.button === 0) clicks++;
      accept({ displayId, x: event.clientX, y: event.clientY, visible: true, clicks, sequence: ++sequence });
    }
    if (!native) { window.addEventListener('pointermove', preview); window.addEventListener('pointerdown', preview); }
    return () => { disposed = true; stop?.(); animation?.cancel(); window.removeEventListener('pointermove', preview); window.removeEventListener('pointerdown', preview); };
  });
</script>
<div class="cursor-highlight" style:--cursor-color={settings.color} style:--cursor-size={`${settings.size}px`} aria-hidden="true">
  <div class="cursor-position" bind:this={halo}><div class="cursor-halo"></div></div>
  <div class="cursor-ripple" bind:this={ripple}></div>
</div>
<style>
  .cursor-highlight { position: fixed; inset: 0; pointer-events: none; z-index: 6; overflow: hidden; }
  .cursor-position { position: absolute; top: 0; left: 0; visibility: hidden; }
  .cursor-halo, .cursor-ripple { width: var(--cursor-size); height: var(--cursor-size); border: 2px solid var(--cursor-color); border-radius: 50%; transform: translate(-50%, -50%); }
  .cursor-halo { background: color-mix(in srgb, var(--cursor-color) 22%, transparent); box-shadow: 0 0 0 1px #0003; }
  .cursor-ripple { position: absolute; opacity: 0; border-width: 3px; }
</style>
