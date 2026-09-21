<script lang="ts">
  import { t } from '../app/i18n';
  import { onMount, type Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  import { DEFAULT_POSITION, readPosition, toolbarBounds } from './toolbar-position';
  import { bridge, message, native } from '../app/bridge';
  import type { ToolbarState } from '../shared/types';
  let { displayId, children, panel, showPanel, oncollapse, oninteract }: {
    displayId: string; children: Snippet<[boolean]>; panel: Snippet; showPanel: boolean;
    oncollapse: () => void; oninteract: () => void;
  } = $props();
  let position = $state({ ...DEFAULT_POSITION });
  let bar = $state<HTMLDivElement>();
  let popup = $state<HTMLDivElement>();
  let vw = $state(0), vh = $state(0), width = $state(0), height = $state(0), panelHeight = $state(0);
  let drag = $state.raw<{ id: number; token: string; ready: Promise<void>; lastX: number; lastY: number; x: number; y: number; left: number; top: number } | null>(null);
  let toolbar = $state<ToolbarState | null>(null);
  let visible = $derived(!native || toolbar?.displayId === displayId);
  let error = $state('');
  let moving = false, pendingMove = false;
  let moveFrame: number | undefined;
  function scheduleMove() {
    if (moveFrame === undefined) moveFrame = requestAnimationFrame(() => { moveFrame = undefined; void pump(); });
  }
  let storageKey = $derived(`my-brush.toolbar.${displayId}`);
  let bx = $derived(toolbarBounds(vw, width));
  let by = $derived(toolbarBounds(vh, height));
  let left = $derived(bx.min + bx.travel * position.x);
  let top = $derived(by.min + by.travel * position.y);
  let above = $derived(vh - top - height < Math.min(panelHeight, 360) + 20 && top > vh - top - height);
  let panelWidth = $derived(Math.min(362, Math.max(0, vw - 20)));
  function save() {
    if (native) { void bridge.updateToolbar({ ...position, displayId }).catch((e) => error = message(e)); }
    else { try { localStorage.setItem(storageKey, JSON.stringify(position)); } catch { /* Optional UI preference. */ } }
  }
  function accept(next: ToolbarState) {
    if (toolbar && next.revision < toolbar.revision) return;
    toolbar = next;
    position = { x: next.position.x, y: next.position.y, collapsed: next.position.collapsed, detailed: next.position.detailed };
    if (!visible) oncollapse();
  }
  function place(x: number, y: number) {
    position.x = bx.travel ? Math.max(0, Math.min(1, (x - bx.min) / bx.travel)) : 0.5;
    position.y = by.travel ? Math.max(0, Math.min(1, (y - by.min) / by.travel)) : 0;
  }
  function start(e: PointerEvent) {
    if (e.button !== 0) return;
    const token = crypto.randomUUID();
    const ready = native ? bridge.beginToolbarDrag({ token, offsetX: e.clientX - left, offsetY: e.clientY - top, width, height }) : Promise.resolve();
    ready.catch((e) => error = message(e));
    drag = { id: e.pointerId, token, ready, lastX: e.clientX, lastY: e.clientY, x: e.clientX, y: e.clientY, left, top };
    e.currentTarget instanceof HTMLElement && e.currentTarget.setPointerCapture(e.pointerId);
    e.preventDefault();
  }
  async function pump() {
    if (moving || !pendingMove || !drag) return;
    moving = true; pendingMove = false;
    const current = drag;
    try { await current.ready; if (drag === current) await bridge.moveToolbar(current.token, { x: current.lastX, y: current.lastY }); }
    catch (e) { error = message(e); }
    finally { moving = false; if (pendingMove) scheduleMove(); }
  }
  function move(e: PointerEvent) {
    if (drag?.id !== e.pointerId) return;
    drag.lastX = e.clientX; drag.lastY = e.clientY;
    if (native) { pendingMove = true; scheduleMove(); }
    else place(drag.left + e.clientX - drag.x, drag.top + e.clientY - drag.y);
  }
  function finish() {
    const current = drag; drag = null; pendingMove = false;
    if (moveFrame !== undefined) { cancelAnimationFrame(moveFrame); moveFrame = undefined; }
    if (!current) return;
    if (native) void current.ready.then(() => bridge.moveToolbar(current.token, { x: current.lastX, y: current.lastY }, true)).catch((e) => error = message(e));
    else save();
  }
  function end(e: PointerEvent) { if (drag?.id === e.pointerId) { if (e.type === 'pointerup') { drag.lastX = e.clientX; drag.lastY = e.clientY; } finish(); } }
  function reset() { position = { ...DEFAULT_POSITION, collapsed: position.collapsed, detailed: position.detailed }; save(); }
  function keyboard(e: KeyboardEvent) {
    if (e.key === 'Home') { e.preventDefault(); e.stopPropagation(); reset(); return; }
    const directions: Record<string, [number, number]> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
    const delta = directions[e.key];
    if (!delta) return;
    e.preventDefault(); e.stopPropagation(); const step = e.shiftKey ? 40 : 10;
    place(left + delta[0] * step, top + delta[1] * step); save();
  }
  function collapse() { position.collapsed = !position.collapsed; oncollapse(); save(); }
  onMount(() => {
    let disposed = false;
    let off: (() => void) | undefined;
    if (native) {
      void (async () => {
        const stop = await bridge.onToolbar((state) => { if (!disposed) accept(state); });
        if (disposed) { stop(); return; }
        off = stop;
        const initial = await bridge.getToolbar();
        if (!disposed) accept(initial);
      })().catch((e) => error = message(e));
    } else { try { position = readPosition(localStorage.getItem(storageKey)); } catch { /* Keep defaults. */ } }
    return () => { disposed = true; off?.(); finish(); };
  });
</script>
<svelte:window bind:innerWidth={vw} bind:innerHeight={vh} />
<!-- Keep the source grip mounted during cross-screen pointer capture. -->
<div class="toolbar-area" class:inactive={!visible} inert={!visible && !drag} aria-hidden={!visible} style:left={`${left}px`} style:top={`${top}px`}>
  <div class="toolbar" bind:this={bar} bind:clientWidth={width} bind:clientHeight={height} role="toolbar" aria-label={$t("그리기 도구")}>
    <button class="toolbar-grip" aria-label={$t("도구막대 이동")} title={$t("다른 모니터로도 드래그해서 이동 · 방향키 이동 · 두 번 클릭 또는 Home으로 위치 초기화")} onpointerdown={start} onpointermove={move} onpointerup={end} onpointercancel={end} onlostpointercapture={end} ondblclick={reset} onkeydown={keyboard}><Icon name="grip" size={16} /></button>
    {#if !position.collapsed}{@render children(position.detailed)}
      <button aria-label={position.detailed ? $t("간단히 보기") : $t("도구 더보기")} aria-expanded={position.detailed} title={position.detailed ? $t("자주 쓰는 도구만 표시") : $t("형광펜·화살표·프리셋·추가 도구")} onclick={() => { position.detailed = !position.detailed; oncollapse(); save(); }}><Icon name="more" size={18} /></button>{:else}
      <span class="compact-label">My Brush</span>
      <button aria-label={$t("앱 조작으로 돌아가기")} title={$t("그림을 유지하고 앱 조작 (Esc)")} onclick={oninteract}><Icon name="pointer" size={18} /></button>
    {/if}
    <button aria-label={position.collapsed ? $t("도구막대 펼치기") : $t("도구막대 접기")} title={position.collapsed ? $t("도구막대 펼치기") : $t("도구막대 접기")} aria-expanded={!position.collapsed} onclick={collapse}><Icon name={position.collapsed ? 'expand' : 'collapse'} size={17} /></button>
  </div>
  {#if error}<span role="alert">{$t(error)}</span>{/if}
</div>
{#if visible && showPanel && !position.collapsed}
  <div class="toolbar-popup" bind:this={popup} bind:clientHeight={panelHeight} style:width={`${panelWidth}px`} style:left={`${Math.max(10, Math.min(vw - panelWidth - 10, left + width / 2 - panelWidth / 2))}px`} style:top={above ? undefined : `${top + height + 10}px`} style:bottom={above ? `${vh - top + 10}px` : undefined} style:max-height={`${Math.max(60, above ? top - 20 : vh - top - height - 20)}px`}>
    {@render panel()}
  </div>
{/if}
<style>
  .toolbar-area { position: fixed; z-index: 10; user-select: none; width: max-content; max-width: calc(100vw - 20px); }
  .toolbar-area.inactive { opacity: 0; pointer-events: none; }
  .toolbar-popup { position: fixed; z-index: 11; overflow-y: auto; border-radius: 16px; box-shadow: 0 8px 24px #0004; }
  .toolbar :global(.toolbar-grip) { cursor: grab; touch-action: none; min-width: 22px; width: 22px; padding: 3px; color: #8d99ad; }
  .toolbar :global(.toolbar-grip:active) { cursor: grabbing; }
  .compact-label { color: #ffcf56; font-size: 11px; padding: 0 4px; }
</style>
