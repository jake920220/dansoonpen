<script lang="ts">
  import { t } from '../app/i18n';
  import { onMount, type Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  import { DEFAULT_POSITION, readPosition, toolbarBounds } from './toolbar-position';
  let { displayId, children, panel, showPanel, oncollapse, oninteract }: {
    displayId: string; children: Snippet<[boolean]>; panel: Snippet; showPanel: boolean;
    oncollapse: () => void; oninteract: () => void;
  } = $props();
  let position = $state({ ...DEFAULT_POSITION });
  let bar = $state<HTMLDivElement>();
  let popup = $state<HTMLDivElement>();
  let vw = $state(0), vh = $state(0), width = $state(0), height = $state(0), panelHeight = $state(0);
  let drag: { id: number; x: number; y: number; left: number; top: number } | null = null;
  let storageKey = $derived(`my-brush.toolbar.${displayId}`);
  let bx = $derived(toolbarBounds(vw, width));
  let by = $derived(toolbarBounds(vh, height));
  let left = $derived(bx.min + bx.travel * position.x);
  let top = $derived(by.min + by.travel * position.y);
  let above = $derived(vh - top - height < Math.min(panelHeight, 360) + 20 && top > vh - top - height);
  let panelWidth = $derived(Math.min(362, Math.max(0, vw - 20)));
  function save() { try { localStorage.setItem(storageKey, JSON.stringify(position)); } catch { /* Optional UI preference. */ } }
  function place(x: number, y: number) {
    position.x = bx.travel ? Math.max(0, Math.min(1, (x - bx.min) / bx.travel)) : 0.5;
    position.y = by.travel ? Math.max(0, Math.min(1, (y - by.min) / by.travel)) : 0;
  }
  function start(e: PointerEvent) {
    if (e.button !== 0) return;
    drag = { id: e.pointerId, x: e.clientX, y: e.clientY, left, top };
    e.currentTarget instanceof HTMLElement && e.currentTarget.setPointerCapture(e.pointerId);
    e.preventDefault();
  }
  function move(e: PointerEvent) { if (drag?.id === e.pointerId) place(drag.left + e.clientX - drag.x, drag.top + e.clientY - drag.y); }
  function end(e: PointerEvent) { if (drag?.id === e.pointerId) { drag = null; save(); } }
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
    try { position = readPosition(localStorage.getItem(storageKey)); } catch { /* Keep defaults. */ }
  });
</script>
<svelte:window bind:innerWidth={vw} bind:innerHeight={vh} />
<div class="toolbar-area" style:left={`${left}px`} style:top={`${top}px`}>
  <div class="toolbar" bind:this={bar} bind:clientWidth={width} bind:clientHeight={height} role="toolbar" aria-label={$t("그리기 도구")}>
    <button class="toolbar-grip" aria-label={$t("도구막대 이동")} title={$t("드래그로 이동 · 방향키 이동 · 두 번 클릭 또는 Home으로 위치 초기화")} onpointerdown={start} onpointermove={move} onpointerup={end} onpointercancel={end} onlostpointercapture={end} ondblclick={reset} onkeydown={keyboard}><Icon name="grip" size={16} /></button>
    {#if !position.collapsed}{@render children(position.detailed)}
      <button aria-label={position.detailed ? $t("간단히 보기") : $t("도구 더보기")} aria-expanded={position.detailed} title={position.detailed ? $t("자주 쓰는 도구만 표시") : $t("형광펜·화살표·프리셋·추가 도구")} onclick={() => { position.detailed = !position.detailed; oncollapse(); save(); }}><Icon name="more" size={18} /></button>{:else}
      <span class="compact-label">My Brush</span>
      <button aria-label={$t("앱 조작으로 돌아가기")} title={$t("그림을 유지하고 앱 조작 (Esc)")} onclick={oninteract}><Icon name="pointer" size={18} /></button>
    {/if}
    <button aria-label={position.collapsed ? $t("도구막대 펼치기") : $t("도구막대 접기")} title={position.collapsed ? $t("도구막대 펼치기") : $t("도구막대 접기")} aria-expanded={!position.collapsed} onclick={collapse}><Icon name={position.collapsed ? 'expand' : 'collapse'} size={17} /></button>
  </div>
</div>
{#if showPanel && !position.collapsed}
  <div class="toolbar-popup" bind:this={popup} bind:clientHeight={panelHeight} style:width={`${panelWidth}px`} style:left={`${Math.max(10, Math.min(vw - panelWidth - 10, left + width / 2 - panelWidth / 2))}px`} style:top={above ? undefined : `${top + height + 10}px`} style:bottom={above ? `${vh - top + 10}px` : undefined} style:max-height={`${Math.max(60, above ? top - 20 : vh - top - height - 20)}px`}>
    {@render panel()}
  </div>
{/if}
<style>
  .toolbar-area { position: fixed; z-index: 10; user-select: none; width: max-content; max-width: calc(100vw - 20px); }
  .toolbar-popup { position: fixed; z-index: 11; overflow-y: auto; border-radius: 16px; box-shadow: 0 8px 24px #0004; }
  .toolbar :global(.toolbar-grip) { cursor: grab; touch-action: none; min-width: 22px; width: 22px; padding: 3px; color: #8d99ad; }
  .toolbar :global(.toolbar-grip:active) { cursor: grabbing; }
  .compact-label { color: #ffcf56; font-size: 11px; padding: 0 4px; }
</style>
