<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { bridge, message, native, shortcutLabel } from '../app/bridge';
  import { arrowEndpoint } from '../canvas/arrow';
  import { CanvasRenderer } from '../canvas/renderer';
  import { appendStrokePoint, hitTestAnnotationsAlongSegment, textAtPoint, textLineBounds, FONT_FAMILY, TEXT_LINE_HEIGHT } from '../canvas/geometry';
  import { DEFAULT_SETTINGS, TOOL_LABELS, defaultBrush, type ArrowAnnotation, type BrushSettings, type Annotation, type AppSettings, type AppState, type Point, type SceneSnapshot, type SceneUpdate, type StrokeAnnotation, type TextAnnotation, type Tool } from '../shared/types';
  import Icon from './Icon.svelte';
  import ToolbarFrame from './ToolbarFrame.svelte';
  import CursorHighlight from './CursorHighlight.svelte';
  import ColorPalette from './ColorPalette.svelte';
  import { SettingsWriter } from '../app/settings-writer';
  import { canvasKey, isMac, isSettingsShortcut, settingsShortcut, matchesShortcut } from '../app/shortcuts';
  import { observeCanvasDiagnostics, opaqueBackground, registerDiagnosticMetrics, reportDiagnostic } from '../app/diagnostics';

  let { displayId }: { displayId: string } = $props();
  let canvas: HTMLCanvasElement;
  let eraserCursor: HTMLDivElement;
  let textarea = $state<HTMLTextAreaElement>();
  let renderer: CanvasRenderer | undefined;
  let appState = $state.raw<AppState | null>(null);
  let error = $state('');
  let showPalette = $state(false);
  let localWidth = $state(DEFAULT_SETTINGS.width);
  let localTextSize = $state(DEFAULT_SETTINGS.textSize);
  let annotationCount = $state(0);
  let textEntry = $state<{ x: number; y: number; value: string; color: string; fontSize: number; original?: TextAnnotation } | null>(null);
  let composing = false;
  let compositionEndedAt = 0;
  let scene: SceneSnapshot = { displayId: '', revision: -1, clearGeneration: 0, annotations: [] };
  let currentStroke: StrokeAnnotation | null = null;
  let currentArrow: ArrowAnnotation | null = null;
  let activePointer: number | null = null;
  let lastPoint: Point | null = null;
  let erased = new Set<string>();
  const pendingAdds = new Map<string, Annotation>();
  const pendingRemovals = new Set<string>();
  let edits: Promise<unknown> = Promise.resolve();
  let disposed = false;
  let optimisticSettings = $state<Partial<AppSettings>>({});
  let optimisticBrush = $state<Partial<BrushSettings>>({});
  let presetSlot = $state(0);
  let presetStatus = $state('');
  let textCommit: Promise<void> | null = null;
  let drawing = $derived(appState?.mode === 'draw' && appState?.activeDisplayId === displayId);
  let settings = $derived({ ...(appState?.settings ?? DEFAULT_SETTINGS), ...optimisticSettings });
  let brush = $derived({ ...(appState?.brush ?? defaultBrush(DEFAULT_SETTINGS)), ...optimisticBrush });
  let tool = $derived(brush.tool);
  let activeWidth = $derived(tool === 'highlighter' ? brush.highlighterWidth : localWidth);
  let currentDisplay = $derived(appState?.displays.find((d) => d.id === displayId));

  function visibleAnnotations(): Annotation[] {
    const ids = new Set(scene.annotations.map((a) => a.id));
    return [...scene.annotations.filter((a) => !erased.has(a.id) && (!pendingRemovals.has(a.id) || pendingAdds.has(a.id))).map((a) => pendingAdds.get(a.id) ?? a), ...[...pendingAdds.values()].filter((a) => !ids.has(a.id) && !erased.has(a.id))];
  }
  function renderScene() { const annotations = visibleAnnotations(); renderer?.setScene(annotations.filter((a) => a.id !== textEntry?.original?.id)); annotationCount = annotations.length; }
  function acceptScene(next: SceneSnapshot, removed: Annotation[] = [], duration?: number) {
    if (next.displayId !== displayId || next.revision < scene.revision) return;
    // Eraser hits already started their fade locally. The native acknowledgement
    // can arrive after that animation ends; never resurrect those objects.
    removed = removed.filter((a) => !erased.has(a.id) && !pendingRemovals.has(a.id));
    const cleared = next.clearGeneration > scene.clearGeneration && scene.revision >= 0;
    if (next.clearGeneration > scene.clearGeneration && scene.revision >= 0) {
      const visible = visibleAnnotations();
      if (currentStroke) visible.push(currentStroke);
      if (currentArrow) visible.push(currentArrow);
      if (textEntry?.value.trim()) {
        const id = textEntry.original?.id ?? crypto.randomUUID();
        const draft: TextAnnotation = { kind: 'text', id, x: textEntry.x, y: textEntry.y, text: textarea?.value ?? textEntry.value, color: textEntry.color, fontSize: textEntry.fontSize };
        const existing = visible.findIndex((a) => a.id === id);
        if (existing >= 0) visible[existing] = draft; else visible.push(draft);
        removed = removed.map((a) => a.id === id ? draft : a);
      }
      const ids = new Set(removed.map((a) => a.id));
      removed = [...removed, ...visible.filter((a) => !ids.has(a.id))];
      const pointer = activePointer; activePointer = null;
      if (pointer !== null && canvas?.hasPointerCapture(pointer)) canvas.releasePointerCapture(pointer);
      currentStroke = null; currentArrow = null; lastPoint = null; erased.clear();
      pendingAdds.clear(); pendingRemovals.clear();
      textEntry = null; composing = false;
      renderer?.setPreview(null);
      duration ??= settings.reduceMotion ? 0 : 350;
    }
    scene = next;
    if (textEntry?.original && !visibleAnnotations().some((a) => a.id === textEntry?.original?.id)) textEntry = null;
    renderScene();
    if (removed.length) renderer?.fadeOut(removed, duration ?? 0);
    if (cleared) reportDiagnostic('scene');
  }
  function acceptUpdate(update: SceneUpdate) {
    if (update.scene.displayId !== displayId || update.scene.revision <= scene.revision) return;
    acceptScene(update.scene, update.fadeOut, update.fadeDurationMs);
  }
  function acceptState(next: AppState) {
    if (appState && next.revision < appState.revision) return;
    const changedMode = !appState || appState.mode !== next.mode || appState.activeDisplayId !== next.activeDisplayId;
    const leaving = drawing && (next.mode !== 'draw' || next.activeDisplayId !== displayId);
    if (leaving) { finishPointer(); void commitText(); showPalette = false; hideEraser(); }
    appState = next;
    localWidth = optimisticBrush.width ?? next.brush.width;
    localTextSize = optimisticBrush.textSize ?? next.brush.textSize;
    if (changedMode) reportDiagnostic('state');
  }
  function resize() { renderer?.resize(window.innerWidth, window.innerHeight, window.devicePixelRatio || 1); }
  onMount(() => {
    const unlisteners: (() => void)[] = [];
    const stopMetrics = registerDiagnosticMetrics(() => ({
      mode: appState?.mode,
      revision: appState?.revision,
      sceneRevision: scene.revision >= 0 ? scene.revision : undefined,
      annotationCount,
      canvasWidth: canvas.width,
      canvasHeight: canvas.height,
      opaqueBackground: opaqueBackground(document.documentElement, document.body, document.getElementById('app'), canvas),
    }));
    const stopContextDiagnostics = observeCanvasDiagnostics(canvas, reportDiagnostic);
    try { renderer = new CanvasRenderer(canvas); resize(); } catch (e) { error = message(e); reportDiagnostic('error'); }
    void (async () => {
      try {
        for (const promise of [bridge.onState(acceptState), bridge.onScene(acceptUpdate)]) {
          const off = await promise;
          if (disposed) off(); else unlisteners.push(off);
        }
        const [initialState, initialScene] = await Promise.all([bridge.getState(), bridge.getScene(displayId)]);
        if (!disposed) { acceptState(initialState); acceptScene(initialScene); reportDiagnostic('ready'); }
      } catch (e) { if (!disposed) { error = message(e); reportDiagnostic('error'); } }
    })();
    return () => { disposed = true; stopMetrics(); stopContextDiagnostics(); unlisteners.forEach((off) => off()); void settingsWriter.flush(); void brushWriter.flush(); renderer?.dispose(); };
  });
  function submit(added: Annotation[], removedIds: string[]) {
    if (!added.length && !removedIds.length) return;
    const clearGeneration = scene.clearGeneration;
    added.forEach((a) => pendingAdds.set(a.id, a));
    removedIds.forEach((id) => pendingRemovals.add(id));
    renderScene();
    edits = edits.then(async () => {
      try {
        const next = await bridge.applyEdit({ displayId, clearGeneration, added, removedIds });
        added.forEach((a) => pendingAdds.delete(a.id));
        removedIds.forEach((id) => pendingRemovals.delete(id));
        if (!disposed) { acceptScene(next); renderScene(); }
      } catch (e) {
        added.forEach((a) => pendingAdds.delete(a.id));
        removedIds.forEach((id) => pendingRemovals.delete(id));
        if (!disposed) { error = `그림을 반영하지 못했습니다: ${message(e)}`; renderScene(); }
      }
    });
  }
  function point(event: PointerEvent): Point { return { x: event.clientX, y: event.clientY }; }
  function updateEraser(event: PointerEvent) {
    if (!eraserCursor) return;
    eraserCursor.style.left = `${event.clientX}px`; eraserCursor.style.top = `${event.clientY}px`;
    eraserCursor.style.display = drawing && tool === 'eraser' ? 'block' : 'none';
  }
  function hideEraser() { if (eraserCursor) eraserCursor.style.display = 'none'; }
  function erase(from: Point, to: Point) {
    const visible = visibleAnnotations();
    const hits = new Set(hitTestAnnotationsAlongSegment(visible, from, to, Math.max(10, localWidth * 2)));
    if (!hits.size) return;
    for (const id of hits) erased.add(id);
    renderScene();
    renderer?.fadeOut(visible.filter((a) => hits.has(a.id)), settings.reduceMotion ? 0 : 220);
  }
  function down(event: PointerEvent) {
    if (!drawing || scene.revision < 0 || event.button !== 0 || activePointer !== null) return;
    error = ''; showPalette = false;
    const start = point(event);
    if (tool === 'text') {
      event.preventDefault();
      void openText(start);
      return;
    }
    if (textEntry) { void commitText(); return; }
    activePointer = event.pointerId; lastPoint = start;
    canvas.setPointerCapture(event.pointerId);
    if (tool === 'pen' || tool === 'highlighter') {
      currentStroke = { kind: 'stroke', id: crypto.randomUUID(), points: [start], color: brush.color, width: activeWidth, opacity: tool === 'highlighter' ? brush.highlighterOpacity : 1 };
      renderer?.setPreview(currentStroke);
    } else if (tool === 'arrow') {
      currentArrow = { kind: 'arrow', id: crypto.randomUUID(), start, end: { ...start }, color: brush.color, width: localWidth };
      renderer?.setPreview(currentArrow);
    } else { erased = new Set(); erase(start, start); }
    event.preventDefault();
  }
  function move(event: PointerEvent) {
    updateEraser(event);
    if (!drawing || event.pointerId !== activePointer) return;
    const samples = event.getCoalescedEvents?.() ?? [event];
    const actual = samples.length ? samples : [event];
    if (currentStroke) {
      for (const sample of actual) {
        if (currentStroke.points.length >= 100_000) break;
        appendStrokePoint(currentStroke.points, point(sample));
      }
      renderer?.setPreview(currentStroke);
      if (currentStroke.points.length >= 100_000) { finishPointer(); error = '긴 획을 저장했습니다. 마우스를 놓고 이어서 그려 주세요.'; }
    } else if (currentArrow) {
      currentArrow.end = arrowEndpoint(currentArrow.start, point(event), event.shiftKey);
      renderer?.setPreview(currentArrow);
    } else if (lastPoint) {
      for (const sample of actual) { const next = point(sample); erase(lastPoint, next); lastPoint = next; }
    }
    event.preventDefault();
  }
  function up(event: PointerEvent) {
    if (event.pointerId !== activePointer) return;
    if (currentStroke && currentStroke.points.length < 100_000) appendStrokePoint(currentStroke.points, point(event), 0, true);
    else if (currentArrow) currentArrow.end = arrowEndpoint(currentArrow.start, point(event), event.shiftKey);
    else if (lastPoint) erase(lastPoint, point(event));
    finishPointer();
  }
  function finishPointer() {
    if (activePointer === null) return;
    const pointer = activePointer; activePointer = null;
    if (canvas?.hasPointerCapture(pointer)) canvas.releasePointerCapture(pointer);
    if (currentStroke) {
      const stroke = currentStroke; currentStroke = null;
      submit([stroke], []); renderer?.setPreview(null);
    } else if (currentArrow) {
      const arrow = currentArrow; currentArrow = null;
      if (Math.hypot(arrow.end.x - arrow.start.x, arrow.end.y - arrow.start.y) >= 4) submit([arrow], []);
      renderer?.setPreview(null);
    } else if (erased.size) { const ids = [...erased]; erased.clear(); submit([], ids); }
    lastPoint = null;
  }
  function cancelPointer(event: PointerEvent) { if (activePointer === event.pointerId) finishPointer(); }
  function chooseTool(next: Tool) { finishPointer(); commitText(); updateBrush({ tool: next }); showPalette = false; if (eraserCursor) eraserCursor.style.display = 'none'; }
  async function openText(start: Point) {
    const generation = scene.clearGeneration;
    await commitText();
    await edits;
    if (!drawing || textEntry || scene.clearGeneration !== generation) return;
    const original = textAtPoint(visibleAnnotations(), start);
    textEntry = original
      ? { x: original.x, y: original.y, value: original.text, color: original.color, fontSize: original.fontSize, original: { ...original } }
      : { x: Math.max(10, Math.min(start.x, innerWidth - 160)), y: Math.max(80, Math.min(start.y, innerHeight - 160)), value: '', color: brush.color, fontSize: localTextSize };
    renderScene();
    await tick();
    textarea?.focus();
    if (original) textarea?.setSelectionRange(original.text.length, original.text.length);
  }
  function cancelText() { textEntry = null; composing = false; renderScene(); }
  function editorWidth() {
    if (!textEntry) return 440;
    const sample: TextAnnotation = { kind: 'text', id: '', text: textEntry.value, ...textEntry };
    const width = Math.max(440, ...textLineBounds(sample).map((line) => line.width + 4));
    return Math.max(140, Math.min(width, innerWidth - textEntry.x - 10));
  }
  function commitText(): Promise<void> {
    if (textCommit) return textCommit;
    if (!textEntry) return Promise.resolve();
    const entry = textEntry;
    const editor = textarea;
    const settling = composing;
    editor?.blur();
    textCommit = (async () => {
      // Native mode changes can precede the final IME input event. Keep the editor
      // alive for the blur/composition events and use its actual DOM value.
      if (settling) await new Promise<void>((resolve) => setTimeout(resolve, 0));
      const value = editor?.value ?? entry.value;
      if (textEntry !== entry || disposed) return;
      if (new TextEncoder().encode(value).length > 40_000) { error = '텍스트가 너무 깁니다. 내용을 나누어 입력해 주세요.'; editor?.focus(); return; }
      textEntry = null; composing = false;
      if (entry.original && value === entry.original.text) { renderScene(); return; }
      const removed = entry.original ? [entry.original.id] : [];
      const annotation: TextAnnotation = { kind: 'text', id: entry.original?.id ?? crypto.randomUUID(), x: entry.x, y: entry.y, text: value, color: entry.color, fontSize: entry.fontSize };
      submit(value.trim() ? [annotation] : [], removed);
      renderScene();
    })().finally(() => { textCommit = null; });
    return textCommit;
  }
  async function action(fn: () => Promise<unknown>) {
    finishPointer(); await commitText(); showPalette = false; error = '';
    await edits;
    await settingsWriter.flush();
    await brushWriter.flush();
    try { await fn(); } catch (e) { error = message(e); }
  }
  const settingsWriter = new SettingsWriter(bridge.updateSettings, acceptState, (pending) => {
    optimisticSettings = pending;
  }, (e) => { if (!disposed) error = message(e); });
  const brushWriter = new SettingsWriter<BrushSettings>(bridge.updateBrush, acceptState, (pending) => {
    optimisticBrush = pending;
    localWidth = pending.width ?? appState?.brush.width ?? DEFAULT_SETTINGS.width;
    localTextSize = pending.textSize ?? appState?.brush.textSize ?? DEFAULT_SETTINGS.textSize;
  }, (e) => { if (!disposed) error = message(e); });
  function updateBrush(patch: Partial<BrushSettings>) { presetStatus = ''; brushWriter.update(patch); }
  function applyPreset(index: number) {
    finishPointer(); void commitText();
    updateBrush(settings.presets[index].brush); showPalette = false; hideEraser();
  }
  function resetBrush() { finishPointer(); void commitText(); updateBrush(defaultBrush(settings)); }
  async function savePreset() {
    const current = { ...brush }; const index = Number(presetSlot);
    presetStatus = ''; error = '';
    try { acceptState(await bridge.updatePreset(index, undefined, current)); presetStatus = `${index + 1}번 프리셋에 저장했습니다.`; }
    catch (e) { error = message(e); }
  }
  function setWidth(width: number) { updateBrush(tool === 'highlighter' ? { highlighterWidth: width } : { width }); }
  function keyboard(event: KeyboardEvent) {
    if (!drawing) return;
    if ((!native || !isMac) && isSettingsShortcut(event)) { event.preventDefault(); void action(bridge.showControl); return; }
    const target = event.target as HTMLElement;
    const editingInput = ['INPUT', 'TEXTAREA', 'SELECT'].includes(target?.tagName) || target?.isContentEditable;
    if (event.isComposing || composing || event.keyCode === 229) return;
    if (event.key === 'Escape') {
      if (performance.now() - compositionEndedAt < 50) return;
      event.preventDefault();
      if (textEntry) { cancelText(); return; }
      if (showPalette) { showPalette = false; return; }
      void action(() => bridge.setMode('interact')); return;
    }
    if (editingInput) return;
    if ((event.metaKey || event.ctrlKey) && !event.altKey && event.code === 'KeyZ') {
      event.preventDefault(); void action(event.shiftKey ? bridge.redo : bridge.undo); return;
    }
    if (!native && matchesShortcut(event, settings.clearShortcut)) { event.preventDefault(); void action(bridge.clearAll); return; }
    if (!native && matchesShortcut(event, settings.toggleShortcut)) { event.preventDefault(); void action(() => bridge.setMode('interact')); return; }
    if (event.shiftKey && !event.ctrlKey && !event.metaKey && !event.altKey && /^Digit[1-3]$/.test(event.code) && !event.repeat) { event.preventDefault(); applyPreset(Number(event.code.slice(-1)) - 1); return; }
    if (event.ctrlKey || event.metaKey || event.altKey || event.shiftKey || event.repeat) return;
    const key = canvasKey(event);
    const tools: Record<string, Tool> = { p: 'pen', e: 'eraser', t: 'text', a: 'arrow', h: 'highlighter' };
    if (key === 'c') { event.preventDefault(); void action(bridge.toggleCursor); }
    else if (tools[key]) { event.preventDefault(); chooseTool(tools[key]); }
    else if (/^[1-6]$/.test(key)) { event.preventDefault(); updateBrush({ color: settings.quickColors[Number(key) - 1] }); }
    else if (key === '[' || key === ']') { event.preventDefault(); setWidth(Math.max(tool === 'highlighter' ? 8 : 1, Math.min(tool === 'highlighter' ? 64 : 32, activeWidth + (key === ']' ? 1 : -1)))); }
  }
  function textKey(event: KeyboardEvent) {
    if (event.isComposing || composing || event.keyCode === 229 || performance.now() - compositionEndedAt < 50) return;
    if (event.key === 'Enter' && !event.shiftKey) { event.preventDefault(); event.stopPropagation(); commitText(); canvas.focus(); }
  }
</script>

<svelte:window onresize={resize} onkeydown={keyboard} onblur={() => { finishPointer(); hideEraser(); }} />
{#if !native}<div class="preview-desktop" aria-hidden="true"><span>MY BRUSH / CANVAS PREVIEW</span><h1>이곳에 설명을 그려 보세요.</h1><p>브라우저에서는 그리기 도구만 미리 볼 수 있습니다.</p><div class="preview-note">화면 위의 표시를 유지한 채<br /><strong>다음 이야기로 넘어가세요.</strong></div></div>{/if}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<canvas bind:this={canvas} class="drawing-surface" style:visibility={appState?.annotationsVisible ? 'visible' : 'hidden'} class:enabled={drawing} class:text-tool={tool === 'text'} class:erase-tool={tool === 'eraser'} aria-label="화면 필기 캔버스" tabindex="-1" onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={cancelPointer} onlostpointercapture={cancelPointer} onpointerleave={() => { if (eraserCursor) eraserCursor.style.display = 'none'; }}></canvas>
<div bind:this={eraserCursor} class="eraser-cursor" style:width={`${Math.max(10, localWidth * 2) * 2}px`} style:height={`${Math.max(10, localWidth * 2) * 2}px`}></div>

{#if appState?.cursorEnabled && appState.activeDisplayId === displayId}<CursorHighlight {displayId} settings={settings.cursor} reduceMotion={settings.reduceMotion} />{/if}

{#if drawing}
  <div class="draw-frame" aria-hidden="true"></div>
  <ToolbarFrame {displayId} showPanel={showPalette} oncollapse={() => showPalette = false} oninteract={() => action(() => bridge.setMode('interact'))}>
      <span class="toolbar-brand" title="My Brush"><Icon name="pen" size={17} /></span>
      <div class="tool-group">
        <button class:chosen={tool === 'pen'} aria-pressed={tool === 'pen'} title="펜 (P)" aria-label="펜" onclick={() => chooseTool('pen')}><Icon name="pen" /></button>
        <button class:chosen={tool === 'highlighter'} aria-pressed={tool === 'highlighter'} title="형광펜 (H)" aria-label="형광펜" onclick={() => chooseTool('highlighter')}><Icon name="highlighter" /></button>
        <button class:chosen={tool === 'arrow'} aria-pressed={tool === 'arrow'} title="화살표 (A) · Shift로 방향 맞추기" aria-label="화살표" onclick={() => chooseTool('arrow')}><Icon name="arrow" /></button>
        <button class:chosen={tool === 'eraser'} aria-pressed={tool === 'eraser'} title="지우개 (E) · 획 단위로 지우기" aria-label="지우개" onclick={() => chooseTool('eraser')}><Icon name="eraser" /></button>
        <button class:chosen={tool === 'text'} aria-pressed={tool === 'text'} title="텍스트 (T)" aria-label="텍스트" onclick={() => chooseTool('text')}><Icon name="text" /></button>
      </div>
      <span class="toolbar-divider"></span>
      <button class="color-trigger" aria-label="색상과 굵기" aria-expanded={showPalette} title="색상과 굵기" onclick={() => showPalette = !showPalette}><span style:background={brush.color}></span><span class="width-dot" style:width={`${Math.min(14, activeWidth + 2)}px`} style:height={`${Math.min(14, activeWidth + 2)}px`}></span></button>
      <div class="tool-group preset-shortcuts" aria-label="빠른 프리셋">
        {#each settings.presets as preset, i}<button aria-label={`프리셋 ${i + 1} ${preset.name}`} title={`${preset.name} (Shift+${i + 1})`} onclick={() => applyPreset(i)}><span style:background={preset.brush.color}></span>{i + 1}</button>{/each}
      </div>
      <span class="toolbar-divider"></span>
      <div class="tool-group">
        <button aria-label="실행 취소" title="실행 취소 (⌘/Ctrl+Z)" onclick={() => action(bridge.undo)}><Icon name="undo" size={19} /></button>
        <button aria-label="다시 실행" title="다시 실행 (⌘/Ctrl+Shift+Z)" onclick={() => action(bridge.redo)}><Icon name="redo" size={19} /></button>
        <button aria-label="이 화면만 지우기" title="이 화면만 지우고 앱 조작으로 복귀" onclick={() => action(bridge.clearCurrent)}><Icon name="clear-screen" size={19} /></button>
        <button aria-label="전체 지우기" title={`전체 지우기 (${shortcutLabel(settings.clearShortcut)})`} onclick={() => action(bridge.clearAll)}><Icon name="clear" size={19} /></button>
      </div>
      <span class="toolbar-divider"></span>
      {#if (appState?.displays.filter((d) => d.connected).length ?? 0) > 1}<select class="overlay-display-select" aria-label="그릴 화면" value={displayId} onchange={(e) => { const selectedId = e.currentTarget.value; void action(() => bridge.selectDisplay(selectedId)); }}>{#each appState?.displays.filter((d) => d.connected) ?? [] as d}<option value={d.id}>{d.name}</option>{/each}</select>{/if}
      <button aria-label="커서 강조" aria-pressed={appState?.cursorEnabled} class:chosen={appState?.cursorEnabled} title="커서 강조 켜기/끄기 (C)" onclick={() => action(bridge.toggleCursor)}><Icon name="cursor-halo" size={19} /></button>
      <button aria-label="필기 잠시 숨기기" title={`필기 잠시 숨기기 (${shortcutLabel(settings.visibilityShortcut)})`} onclick={() => action(bridge.toggleAnnotations)}><Icon name="eye" size={19} /></button>
      <button aria-label="설정 열기" title={`설정 열기 (${shortcutLabel(settingsShortcut)})`} onclick={() => action(bridge.showControl)}><Icon name="settings" size={19} /></button>
      <button class="interact-button" aria-label="앱 조작으로 돌아가기" title="그림을 유지하고 앱 조작 (Esc)" onclick={() => action(() => bridge.setMode('interact'))}><Icon name="pointer" size={17} /><span>앱 조작</span><kbd>Esc</kbd></button>
    {#snippet panel()}
      <div class="palette-panel">
        <div class="palette-heading"><span>{TOOL_LABELS[tool]} 색상</span><code>{brush.color.toUpperCase()}</code></div>
        <ColorPalette value={brush.color} colors={settings.quickColors} onchange={(color) => updateBrush({ color })} onpalettechange={(quickColors) => settingsWriter.update({ quickColors })} />
        <label class="palette-range"><span>{tool === 'text' ? '글자 크기' : `${TOOL_LABELS[tool]} 굵기`}</span>{#if tool === 'text'}<input aria-label="글자 크기" type="range" min="12" max="96" step="2" value={localTextSize} oninput={(e) => updateBrush({ textSize: Number(e.currentTarget.value) })} /><output>{localTextSize}px</output>{:else}<input aria-label="도구 굵기" type="range" min={tool === 'highlighter' ? 8 : 1} max={tool === 'highlighter' ? 64 : 32} step="1" value={activeWidth} oninput={(e) => setWidth(Number(e.currentTarget.value))} /><output>{activeWidth}px</output>{/if}</label>
        {#if tool === 'highlighter'}<label class="palette-range"><span>불투명도</span><input aria-label="형광펜 불투명도" type="range" min="10" max="80" step="1" value={Math.round(brush.highlighterOpacity * 100)} oninput={(e) => updateBrush({ highlighterOpacity: Number(e.currentTarget.value) / 100 })} /><output>{Math.round(brush.highlighterOpacity * 100)}%</output></label>{/if}
        <div class="brush-reset"><span>현재 도구에만 적용됩니다.</span><button onclick={resetBrush}>기본 펜으로</button></div>
        <div class="preset-save"><select aria-label="저장할 프리셋" bind:value={presetSlot}>{#each settings.presets as preset, i}<option value={i}>{i + 1}. {preset.name}</option>{/each}</select><button onclick={savePreset}>현재 도구 저장</button></div>
        {#if presetStatus}<p role="status">{presetStatus}</p>{/if}
        <p>1–6 색상 · [ ] 굵기 · Shift+1–3 프리셋</p>
      </div>
    {/snippet}
  </ToolbarFrame>
  <div class="mode-hint"><span class="hint-dot"></span><strong>{TOOL_LABELS[tool]}</strong><span class="hint-separator"></span>{#if tool === 'text'}빈 곳에 새 글 · 기존 글을 클릭해 수정{:else if tool === 'eraser'}그림을 문지르면 획 단위로 지워집니다{:else if tool === 'arrow'}드래그로 연결 · Shift로 45° 방향 맞추기{:else if tool === 'highlighter'}글자를 가리지 않고 강조하세요{:else}직접 지우기 전까지 남아 있습니다{/if}<span class="hint-count">{annotationCount}개</span></div>
{:else if !native}
  <div class="preview-resume"><button class="primary" onclick={() => bridge.setMode('draw').then(acceptState)}>다시 그리기</button><span>그림은 유지됩니다 · {currentDisplay?.name ?? '미리보기'}</span></div>
{/if}

{#if textEntry}
  <div class="text-editor" style:left={`${textEntry.x}px`} style:top={`${textEntry.y}px`}>
    <textarea bind:this={textarea} bind:value={textEntry.value} aria-label="화면에 입력할 텍스트" placeholder="텍스트 입력" wrap="off" maxlength="10000" rows={Math.min(8, textEntry.value.split('\n').length + 1)} spellcheck="false" style:width={`${editorWidth()}px`} style:color={textEntry.color} style:font-family={FONT_FAMILY} style:font-size={`${textEntry.fontSize}px`} style:line-height={TEXT_LINE_HEIGHT} onkeydown={textKey} oncompositionstart={() => composing = true} oncompositionend={() => { composing = false; compositionEndedAt = performance.now(); }}></textarea>
    <div class="text-editor-actions"><span>{textEntry.original ? '수정 중 · ' : ''}Enter 완료 · Shift+Enter 줄바꿈 · Esc 취소</span><button aria-label="텍스트 취소" onclick={cancelText}><Icon name="close" size={16} /></button><button class="text-confirm" aria-label="텍스트 완료" onclick={commitText}><Icon name="check" size={16} />완료</button></div>
  </div>
{/if}
{#if error}<div class="overlay-error error-box" role="alert"><span>{error}</span><button onclick={() => action(() => bridge.setMode('interact'))}>앱 조작으로 돌아가기</button><button aria-label="알림 닫기" onclick={() => error = ''}><Icon name="close" size={16} /></button></div>{/if}
