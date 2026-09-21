<script lang="ts">
  import { t } from '../app/i18n';
  import type { Tool } from '../shared/types';
  import { SIZE_SPECS } from '../shared/tool-size';
  import { FONT_FAMILY } from '../canvas/geometry';
  let { tool, value, color, opacity = 1, onchange }: {
    tool: Tool; value: number; color: string; opacity?: number; onchange: (value: number) => void;
  } = $props();
  let spec = $derived(SIZE_SPECS[tool]);
</script>
<div class="size-control">
  <div class="size-heading"><span>{$t(spec.label)}</span><output>{value}px</output></div>
  <div class="quick-sizes" aria-label={$t("{0} 빠른 선택", [$t(spec.label)])}>
    {#each spec.quick as size, i}
      <button class:chosen={size === value} aria-pressed={size === value} onclick={() => onchange(size)}>{[$t("작게"), $t("보통"), $t("크게")][i]} <span>{size}</span></button>
    {/each}
  </div>
  <input aria-label={$t(spec.label)} type="range" min={spec.min} max={spec.max} step={spec.step} {value} oninput={(e) => onchange(Number(e.currentTarget.value))} />
  <div class="size-preview" aria-label={$t("{0}px 실제 크기 미리보기", [value])}>
    {#if tool === 'text'}<span style:font-family={FONT_FAMILY} style:font-size={`${value}px`} style:color={color}>{$t("가 Aa")}</span>
    {:else if tool === 'eraser'}<div class="eraser-sample" style:width={`${value}px`} style:height={`${value}px`}></div>
    {:else}<svg width="210" height="76" aria-hidden="true"><path d="M24 49 Q63 10 107 38 T186 28" fill="none" stroke={color} stroke-width={value} stroke-linecap="round" opacity={tool === 'highlighter' ? opacity : 1} /></svg>{/if}
  </div>
  <p>{$t("실제 크기 미리보기 · [ 작게 · ] 크게")}</p>
</div>
<style>
  .size-control { margin-top: 18px; color: #d4dce9; }
  .size-heading { display: flex; justify-content: space-between; font-size: 12px; margin-bottom: 10px; }
  output { color: #ffcf56; font-variant-numeric: tabular-nums; }
  .quick-sizes { display: flex; gap: 6px; }
  button { flex: 1; border: 1px solid #505969; border-radius: 8px; background: #303745; color: #cad3e2; padding: 8px 4px; font-size: 11px; }
  button span { margin-left: 4px; opacity: .7; }
  button.chosen { border-color: #ffcf56; color: #ffcf56; background: #ffcf5612; }
  input { width: 100%; margin: 14px 0; accent-color: #ffcf56; }
  .size-preview { height: 148px; display: flex; align-items: center; justify-content: center; overflow: auto; border: 1px solid #414b5b; border-radius: 10px; background: #141922; }
  .size-preview > * { flex-shrink: 0; }
  .size-preview span { white-space: nowrap; line-height: 1; }
  .eraser-sample { border: 1.5px solid white; border-radius: 50%; background: #ffffff10; box-sizing: border-box; }
  p { margin: 8px 0 0; color: #929fb2; font-size: 10px; }
</style>
