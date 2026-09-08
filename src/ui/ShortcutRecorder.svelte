<script lang="ts">
  import { onDestroy } from 'svelte';
  import { shortcutLabel } from '../app/bridge';
  import { recordedShortcut } from '../app/shortcuts';
  let { id, value, onchange, capture }: { id: string; value: string; onchange: (value: string) => void; capture: (active: boolean) => Promise<void> } = $props();
  let focused = $state(false);
  let ready = $state(false);
  let candidate = $state('');
  let hint = $state('');
  let input = $state<HTMLInputElement>();
  onDestroy(() => { if (focused) void capture(false).catch(() => {}); });
  async function focus() {
    focused = true; ready = false; candidate = ''; hint = '';
    try { await capture(true); if (focused) ready = true; }
    catch { if (focused) hint = '입력을 시작하지 못했습니다. 다시 클릭해 주세요.'; }
  }
  function blur() {
    focused = false; ready = false; candidate = ''; hint = '';
    void capture(false).catch(() => {});
  }
  function keydown(event: KeyboardEvent) {
    if (event.key === 'Tab') return;
    event.preventDefault(); event.stopPropagation();
    if (event.key === 'Escape') { (event.currentTarget as HTMLInputElement).blur(); return; }
    if (event.key === 'Enter') {
      if (candidate) onchange(candidate);
      (event.currentTarget as HTMLInputElement).blur(); return;
    }
    if (!ready) return;
    const next = recordedShortcut(event);
    if (next) { candidate = next; hint = '키를 놓으면 입력됩니다.'; }
    else if (!/^(Shift|Control|Alt|Meta)$/.test(event.key)) hint = '⌘ / Ctrl / Alt와 함께 눌러 주세요.';
  }
  function keyup(event: KeyboardEvent) {
    if (!candidate || event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
    onchange(candidate);
    (event.currentTarget as HTMLInputElement).blur();
  }
</script>
<svelte:window onblur={() => { if (focused) blur(); }} onfocus={() => { if (input === document.activeElement) void focus(); }} />
<div class="shortcut-recorder">
  <input bind:this={input} {id} class="shortcut-input" class:recording={focused} readonly autocomplete="off" spellcheck="false"
    aria-describedby={`${id}-hint`} value={focused ? candidate ? shortcutLabel(candidate) : ready ? '단축키를 눌러 주세요' : '입력 준비 중…' : value ? shortcutLabel(value) : '지정하지 않음'}
    onfocus={() => void focus()} onblur={blur} onkeydown={keydown} onkeyup={keyup} />
  <span id={`${id}-hint`} class="recorder-hint">{focused ? hint || 'Esc 취소 · Tab 이동' : '클릭하여 단축키 입력'}</span>
</div>
