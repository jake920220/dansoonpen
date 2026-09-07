<script lang="ts">
  import { untrack } from 'svelte';
  import Icon from './Icon.svelte';
  import { hexToHsv, hsvToHex, wheelPosition, type HSV } from '../shared/color';
  let { value, colors, onchange, onpalettechange }: { value: string; colors: string[]; onchange: (color: string) => void; onpalettechange: (colors: string[]) => void } = $props();
  let expanded = $state(false);
  let slot = $state(0);
  let hsv = $state<HSV>({ h: 0, s: 0, v: 1 });
  let hex = $state('');
  let hexError = $state('');
  let wheel = $state<HTMLDivElement>()!;
  let pointer: number | null = null;
  $effect(() => {
    const next = hexToHsv(value);
    const previous = untrack(() => hsv);
    // Retain hue/saturation while darkening to black so the brightness slider is reversible.
    hsv = { h: next.s > 0 ? next.h : previous.h, s: next.v > 0 ? next.s : previous.s, v: next.v };
    hex = value.toUpperCase();
  });
  function change(next: HSV) { hsv = next; hexError = ''; onchange(hsvToHex(next)); }
  function sample(event: PointerEvent) {
    const bounds = wheel.getBoundingClientRect();
    const radius = bounds.width / 2;
    change({ ...hsv, ...wheelPosition((event.clientX - bounds.left - radius) / radius, (event.clientY - bounds.top - radius) / radius) });
  }
  function down(event: PointerEvent) {
    if (event.button !== 0 || pointer !== null) return;
    pointer = event.pointerId; wheel.setPointerCapture(pointer); wheel.focus(); sample(event); event.preventDefault();
  }
  function up(event: PointerEvent) {
    if (pointer !== event.pointerId) return;
    sample(event); pointer = null; wheel.releasePointerCapture(event.pointerId);
  }
  function keydown(event: KeyboardEvent) {
    if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(event.key)) return;
    event.preventDefault(); event.stopPropagation();
    const h = (hsv.h + (event.key === 'ArrowRight' ? 3 : event.key === 'ArrowLeft' ? -3 : 0) + 360) % 360;
    const s = Math.max(0, Math.min(1, hsv.s + (event.key === 'ArrowUp' ? .03 : event.key === 'ArrowDown' ? -.03 : 0)));
    change({ ...hsv, h, s });
  }
  function applyHex() {
    const normalized = hex.startsWith('#') ? hex : `#${hex}`;
    if (!/^#[0-9a-f]{6}$/i.test(normalized)) { hexError = '6자리 색상 코드를 입력해 주세요.'; return; }
    hexError = ''; onchange(normalized.toLowerCase());
  }
</script>

<div class="color-palette">
  <div class="color-swatches" role="group" aria-label="빠른 색상">
    {#each colors as color, i}
      <button type="button" class="color-swatch" class:selected={value.toLowerCase() === color.toLowerCase()}
        style:background={color} aria-label={`색상 ${i + 1} ${color.toUpperCase()}`} aria-pressed={value.toLowerCase() === color.toLowerCase()}
        title={`색상 ${i + 1} · ${color.toUpperCase()}`} onclick={() => { slot = i; onchange(color); }}>
        {#if value.toLowerCase() === color.toLowerCase()}<Icon name="check" size={17} />{/if}
      </button>
    {/each}
    <button type="button" class="spectrum-swatch" aria-label="색상환 열기" aria-expanded={expanded} title="색상환에서 고르기" onclick={() => expanded = !expanded}><span>{expanded ? '−' : '+'}</span></button>
  </div>
  {#if expanded}
    <div class="color-studio">
      <div class="color-studio-heading"><span>나만의 색상</span><span class="color-preview" style:background={value}></span></div>
      <div bind:this={wheel} class="color-wheel" role="slider" tabindex="0" aria-label="색상환" aria-valuemin="0" aria-valuemax="360" aria-valuenow={Math.round(hsv.h)} aria-valuetext={`${value}, 채도 ${Math.round(hsv.s * 100)}%`}
        onpointerdown={down} onpointermove={(e) => { if (pointer === e.pointerId) sample(e); }} onpointerup={up} onpointercancel={() => pointer = null} onlostpointercapture={() => pointer = null} onkeydown={keydown}>
        <div class="wheel-shade" style:opacity={1 - hsv.v}></div>
        <span class="wheel-handle" style:left={`${50 + Math.sin(hsv.h * Math.PI / 180) * hsv.s * 50}%`} style:top={`${50 - Math.cos(hsv.h * Math.PI / 180) * hsv.s * 50}%`} style:background={value}></span>
      </div>
      <label class="brightness-control"><span>밝기</span><input type="range" min="0" max="100" aria-label="색상 밝기" value={Math.round(hsv.v * 100)} style:background={`linear-gradient(to right, #000, ${hsvToHex({ ...hsv, v: 1 })})`} oninput={(e) => change({ ...hsv, v: Number(e.currentTarget.value) / 100 })} /></label>
      <div class="color-code-row"><span>HEX</span><input aria-label="색상 코드" bind:value={hex} maxlength="7" spellcheck="false" onblur={applyHex} onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); applyHex(); } }} /></div>
      {#if hexError}<span class="color-input-error" role="alert">{hexError}</span>{/if}
      <button class="save-swatch" type="button" onclick={() => { const next = [...colors]; next[slot] = value; onpalettechange(next); }}>빠른 색상 {slot + 1}에 저장</button>
    </div>
  {/if}
</div>

<style>
  .color-palette { width: 100%; }
  .color-swatches { display: flex; align-items: center; flex-wrap: wrap; gap: 13px; padding: 6px 4px; }
  .color-swatch, .spectrum-swatch { flex: 0 0 29px; width: 29px; height: 29px; border: 0; border-radius: 50%; padding: 0; display: inline-flex; align-items: center; justify-content: center; box-shadow: inset 0 0 0 1px #ffffff25; transition: transform .12s; }
  .color-swatch { color: #20232b; }
  .color-swatch :global(svg) { filter: drop-shadow(0 0 1px #fff); }
  .color-swatch:hover, .spectrum-swatch:hover { transform: scale(1.12); }
  .color-swatch.selected { outline: 1.5px solid #f2f4f8; outline-offset: 4px; }
  .spectrum-swatch { background: conic-gradient(#ff6969, #ffe478, #62ddad, #65b8ff, #be91ff, #ff6969); }
  .spectrum-swatch span { display: flex; justify-content: center; align-items: center; width: 21px; height: 21px; border-radius: 50%; background: #242831; color: #eee; font-size: 17px; line-height: 1; }
  .color-studio { margin-top: 19px; padding: 17px; border: 1px solid #3c4350; background: #1c2028; border-radius: 16px; width: min(100%, 292px); }
  .color-studio-heading { display: flex; justify-content: space-between; align-items: center; font-size: 11px; color: #b1bbc9; margin-bottom: 19px; }
  .color-preview { width: 19px; height: 19px; border-radius: 50%; border: 1px solid #ffffff40; }
  .color-wheel { width: 174px; height: 174px; margin: 0 auto 21px; position: relative; border-radius: 50%; background: radial-gradient(circle closest-side, #fff, #fff0), conic-gradient(red, #ff0, #0f0, #0ff, blue, #f0f, red); touch-action: none; cursor: crosshair; }
  .wheel-shade { position: absolute; inset: 0; border-radius: 50%; background: #000; pointer-events: none; }
  .wheel-handle { position: absolute; width: 14px; height: 14px; border: 2px solid #fff; border-radius: 50%; transform: translate(-50%, -50%); box-shadow: 0 1px 4px #0008, 0 0 0 1px #0003; pointer-events: none; }
  .brightness-control { display: flex; gap: 12px; align-items: center; color: #a7b2c3; font-size: 10px; }
  .brightness-control input { appearance: none; -webkit-appearance: none; flex: 1; min-width: 0; height: 9px; border-radius: 10px; border: 1px solid #ffffff20; }
  .brightness-control input::-webkit-slider-thumb { appearance: none; -webkit-appearance: none; width: 13px; height: 13px; border-radius: 50%; border: 2px solid #fff; background: #505866; }
  .color-code-row { margin-top: 17px; display: flex; justify-content: space-between; align-items: center; gap: 10px; color: #8794a9; font-size: 9px; letter-spacing: 1px; }
  .color-code-row input { width: 106px; border: 1px solid #3b4554; border-radius: 7px; background: #252c38; color: #d6dfed; padding: 6px 9px; font: 11px ui-monospace, monospace; letter-spacing: .4px; }
  .save-swatch { width: 100%; border: 1px solid #424b59; background: transparent; color: #aebcd0; border-radius: 7px; padding: 8px; font-size: 10px; margin-top: 14px; }
  .save-swatch:hover { background: #2b3340; }
  .color-input-error { display: block; font-size: 10px; margin-top: 8px; color: #ffb9a7; }
</style>
