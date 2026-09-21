<script lang="ts">
  import { t, setLanguage } from '../app/i18n';
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
  import SizeControl from './SizeControl.svelte';
  import FeedbackNotice from './FeedbackNotice.svelte';
  import { toolSize, sizePatch, stepSize } from '../shared/tool-size';
  import { constrainTextEditor, textDraftChanged, type TextDraft } from './text-edit';
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
  let textEntry = $state<TextDraft | null>(null);
  let textEditor = $state<HTMLDivElement>();
  let textDrag: { pointer: number; start: Point; origin: Point; entry: TextDraft } | null = null;
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
  let editingBrush = $derived(textEntry ? { ...brush, color: textEntry.color, textSize: textEntry.fontSize } : brush);
  let activeWidth = $derived(tool === 'highlighter' ? brush.highlighterWidth : tool === 'eraser' ? brush.eraserSize : localWidth);
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
      textEntry = null; textDrag = null; composing = false;
      renderer?.setPreview(null);
      duration ??= settings.reduceMotion ? 0 : 350;
    }
    scene = next;
    if (textEntry?.original && !visibleAnnotations().some((a) => a.id === textEntry?.original?.id)) { textEntry = null; textDrag = null; }
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
    const newEntry = appState && appState.brushGeneration !== next.brushGeneration;
    appState = next;
    setLanguage(next.settings.language);
    if (newEntry) brushWriter.invalidate();
    localWidth = optimisticBrush.width ?? next.brush.width;
    localTextSize = optimisticBrush.textSize ?? next.brush.textSize;
    if (changedMode) reportDiagnostic('state');
  }
  function resize() { renderer?.resize(window.innerWidth, window.innerHeight, window.devicePixelRatio || 1); if (textEntry) placeText(textEntry); }
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
        // Load the bundled font before text layout or hit testing can begin.
        await document.fonts.load('40px "Nanum Gothic"').catch(() => {
          error = $t("나눔고딕을 불러오지 못해 기본 글꼴로 표시합니다.");
          reportDiagnostic('error');
        });
        if (disposed) return;
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
        if (!disposed) { error = $t("그림을 반영하지 못했습니다: {0}", [message(e)]); renderScene(); }
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
    const hits = new Set(hitTestAnnotationsAlongSegment(visible, from, to, brush.eraserSize / 2));
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
      if (currentStroke.points.length >= 100_000) { finishPointer(); error = $t("긴 획을 저장했습니다. 마우스를 놓고 이어서 그려 주세요."); }
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
  function cancelText() { textEntry = null; textDrag = null; composing = false; renderScene(); }
  function placeText(point: Point) {
    if (!textEntry) return;
    const bounds = textEditor?.getBoundingClientRect();
    const next = constrainTextEditor(point, bounds?.width ?? editorWidth(), bounds?.height ?? 140, { width: innerWidth, height: innerHeight });
    textEntry.x = next.x; textEntry.y = next.y;
  }
  function startTextMove(event: PointerEvent) {
    if (event.button !== 0 || !textEntry || textDrag) return;
    textDrag = { pointer: event.pointerId, start: point(event), origin: { x: textEntry.x, y: textEntry.y }, entry: textEntry };
    (event.currentTarget as HTMLElement).focus({ preventScroll: true });
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    event.preventDefault();
  }
  function moveText(event: PointerEvent) {
    if (!textDrag || textDrag.pointer !== event.pointerId || textDrag.entry !== textEntry) return;
    placeText({ x: textDrag.origin.x + event.clientX - textDrag.start.x, y: textDrag.origin.y + event.clientY - textDrag.start.y });
  }
  function endTextMove(event: PointerEvent) { if (textDrag?.pointer === event.pointerId) textDrag = null; }
  function textMoveKey(event: KeyboardEvent) {
    const directions: Record<string, Point> = { ArrowLeft: { x: -1, y: 0 }, ArrowRight: { x: 1, y: 0 }, ArrowUp: { x: 0, y: -1 }, ArrowDown: { x: 0, y: 1 } };
    const delta = directions[event.key];
    if (!delta || !textEntry || event.isComposing) return;
    event.preventDefault(); event.stopPropagation(); const step = event.shiftKey ? 10 : 1;
    placeText({ x: textEntry.x + delta.x * step, y: textEntry.y + delta.y * step });
  }
  function changeColor(color: string) { if (textEntry) textEntry.color = color; else updateBrush({ color }); }

  function editorWidth() {
    if (!textEntry) return 440;
    const sample: TextAnnotation = { kind: 'text', id: '', text: textEntry.value, ...textEntry };
    const width = Math.max(textEntry.value ? 160 : 440, ...textLineBounds(sample).map((line) => line.width + 4));
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
      if (new TextEncoder().encode(value).length > 40_000) { error = $t("텍스트가 너무 깁니다. 내용을 나누어 입력해 주세요."); editor?.focus(); return; }
      textEntry = null; textDrag = null; composing = false;
      if (!textDraftChanged(entry, value)) { renderScene(); return; }
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
  const brushWriter = new SettingsWriter<BrushSettings>((patch) => bridge.updateBrush(patch, appState!.brushGeneration), acceptState, (pending) => {
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
    try { acceptState(await bridge.updatePreset(index, undefined, current)); presetStatus = $t("{0}번 프리셋에 저장했습니다.", [index + 1]); }
    catch (e) { error = message(e); }
  }
  function setSize(value: number) {
    const patch = sizePatch(tool, value);
    if (textEntry && patch.textSize !== undefined) textEntry.fontSize = patch.textSize;
    else updateBrush(patch);
  }
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
    else if (/^[1-6]$/.test(key)) { event.preventDefault(); changeColor(settings.quickColors[Number(key) - 1]); }
    else if (key === '[' || key === ']') { event.preventDefault(); const patch = stepSize(editingBrush, key === ']' ? 1 : -1); setSize(toolSize({ ...editingBrush, ...patch })); }
  }
  function textKey(event: KeyboardEvent) {
    if (event.isComposing || composing || event.keyCode === 229 || performance.now() - compositionEndedAt < 50) return;
    if (event.key === 'Enter' && !event.shiftKey) { event.preventDefault(); event.stopPropagation(); commitText(); canvas.focus(); }
  }
</script>

<svelte:window onresize={resize} onkeydown={keyboard} onblur={() => { finishPointer(); hideEraser(); }} />
{#if !native}<div class="preview-desktop" aria-hidden="true"><span>MY BRUSH / CANVAS PREVIEW</span><h1>{$t("이곳에 설명을 그려 보세요.")}</h1><p>{$t("브라우저에서는 그리기 도구만 미리 볼 수 있습니다.")}</p><div class="preview-note">{$t("화면 위의 표시를 유지한 채")}<br /><strong>{$t("다음 이야기로 넘어가세요.")}</strong></div></div>{/if}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<canvas bind:this={canvas} class="drawing-surface" style:visibility={appState?.annotationsVisible ? 'visible' : 'hidden'} class:enabled={drawing} class:text-tool={tool === 'text'} class:erase-tool={tool === 'eraser'} aria-label={$t("화면 필기 캔버스")} tabindex="-1" onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={cancelPointer} onlostpointercapture={cancelPointer} onpointerleave={() => { if (eraserCursor) eraserCursor.style.display = 'none'; }}></canvas>
<div bind:this={eraserCursor} class="eraser-cursor" style:width={`${brush.eraserSize}px`} style:height={`${brush.eraserSize}px`}></div>

{#if appState?.cursorEnabled && appState.activeDisplayId === displayId}<CursorHighlight {displayId} settings={settings.cursor} reduceMotion={settings.reduceMotion} />{/if}

{#if appState?.activeDisplayId === displayId}
  <FeedbackNotice feedback={appState.feedback} undoToken={appState.clearUndoToken} interactive={drawing || !native} reduceMotion={settings.reduceMotion} onundo={(token) => action(() => bridge.undoClear(token))} />
{/if}
{#if drawing}
  <div class="draw-frame" aria-hidden="true"></div>
  <ToolbarFrame {displayId} showPanel={showPalette} oncollapse={() => showPalette = false} oninteract={() => action(() => bridge.setMode('interact'))}>
    {#snippet children(detailed: boolean)}
      <span class="toolbar-brand" title="My Brush"><Icon name="pen" size={17} /></span>
      <div class="tool-group">
        <button class:chosen={tool === 'pen'} aria-pressed={tool === 'pen'} title={$t("펜 (P)")} aria-label={$t("펜")} onclick={() => chooseTool('pen')}><Icon name="pen" /></button>
        {#if detailed || tool === 'highlighter'}<button class:chosen={tool === 'highlighter'} aria-pressed={tool === 'highlighter'} title={$t("형광펜 (H)")} aria-label={$t("형광펜")} onclick={() => chooseTool('highlighter')}><Icon name="highlighter" /></button>{/if}
        {#if detailed || tool === 'arrow'}<button class:chosen={tool === 'arrow'} aria-pressed={tool === 'arrow'} title={$t("화살표 (A) · Shift로 방향 맞추기")} aria-label={$t("화살표")} onclick={() => chooseTool('arrow')}><Icon name="arrow" /></button>{/if}
        <button class:chosen={tool === 'eraser'} aria-pressed={tool === 'eraser'} title={$t("지우개 (E) · 획 단위로 지우기")} aria-label={$t("지우개")} onclick={() => chooseTool('eraser')}><Icon name="eraser" /></button>
        <button class:chosen={tool === 'text'} aria-pressed={tool === 'text'} title={$t("텍스트 (T)")} aria-label={$t("텍스트")} onclick={() => chooseTool('text')}><Icon name="text" /></button>
      </div>
      <span class="toolbar-divider"></span>
      <button class="color-trigger" aria-label={$t("색상과 굵기")} aria-expanded={showPalette} title={$t("색상과 굵기")} onclick={() => showPalette = !showPalette}><span style:background={editingBrush.color}></span><span class="current-size">{toolSize(editingBrush)}px</span></button>
      {#if detailed}
      <div class="tool-group preset-shortcuts" aria-label={$t("빠른 프리셋")}>
        {#each settings.presets as preset, i}<button aria-label={$t("프리셋 {0} {1}", [i + 1, preset.name])} title={`${preset.name} (Shift+${i + 1})`} onclick={() => applyPreset(i)}><span style:background={preset.brush.color}></span>{i + 1}</button>{/each}
      </div>
      {/if}
      <span class="toolbar-divider"></span>
      <div class="tool-group">
        <button aria-label={$t("실행 취소")} title={$t("실행 취소 (⌘/Ctrl+Z)")} onclick={() => action(bridge.undo)}><Icon name="undo" size={19} /></button>
        {#if detailed}<button aria-label={$t("다시 실행")} title={$t("다시 실행 (⌘/Ctrl+Shift+Z)")} onclick={() => action(bridge.redo)}><Icon name="redo" size={19} /></button>{/if}
        {#if detailed}<button aria-label={$t("이 화면만 지우기")} title={$t("이 화면의 필기만 지우기 · 현재 도구 유지")} onclick={() => action(bridge.clearCurrent)}><Icon name="clear-screen" size={19} /></button>{/if}
        <button aria-label={$t("전체 지우기")} title={$t("전체 지우기 ({0})", [shortcutLabel(settings.clearShortcut)])} onclick={() => action(bridge.clearAll)}><Icon name="clear" size={19} /></button>
      </div>
      {#if appState?.clearUndoToken != null}<button class="clear-undo" aria-label={$t("방금 지운 필기 되돌리기")} onclick={() => { const token = appState?.clearUndoToken; if (token != null) void action(() => bridge.undoClear(token)); }}><Icon name="undo" size={17} />{$t("삭제 복구")}</button>{/if}
      <span class="toolbar-divider"></span>
      {#if (appState?.displays.filter((d) => d.connected).length ?? 0) > 1}<select class="overlay-display-select" aria-label={$t("그릴 화면")} value={displayId} onchange={(e) => { const selectedId = e.currentTarget.value; void action(() => bridge.selectDisplay(selectedId)); }}>{#each appState?.displays.filter((d) => d.connected) ?? [] as d}<option value={d.id}>{$t(d.name)}</option>{/each}</select>{/if}
      {#if detailed || appState?.cursorEnabled}<button aria-label={$t("커서 강조")} aria-pressed={appState?.cursorEnabled} class:chosen={appState?.cursorEnabled} title={$t("커서 강조 켜기/끄기 (C)")} onclick={() => action(bridge.toggleCursor)}><Icon name="cursor-halo" size={19} /></button>{/if}
      {#if detailed}<button aria-label={$t("필기 잠시 숨기기")} title={$t("필기 잠시 숨기기 ({0})", [shortcutLabel(settings.visibilityShortcut)])} onclick={() => action(bridge.toggleAnnotations)}><Icon name="eye" size={19} /></button>{/if}
      {#if detailed}<button aria-label={$t("설정 열기")} title={$t("설정 열기 ({0})", [shortcutLabel(settingsShortcut)])} onclick={() => action(bridge.showControl)}><Icon name="settings" size={19} /></button>{/if}
      <button class="interact-button" aria-label={$t("앱 조작으로 돌아가기")} title={$t("그림을 유지하고 앱 조작 (Esc)")} onclick={() => action(() => bridge.setMode('interact'))}><Icon name="pointer" size={17} /><span>{$t("앱 조작")}</span><kbd>Esc</kbd></button>
    {/snippet}
    {#snippet panel()}
      <div class="palette-panel">
        {#if tool !== 'eraser'}<div class="palette-heading"><span>{$t("{0} 색상", [$t(TOOL_LABELS[tool])])}</span><code>{editingBrush.color.toUpperCase()}</code></div>
        <ColorPalette value={editingBrush.color} colors={settings.quickColors} onchange={changeColor} onpalettechange={(quickColors) => settingsWriter.update({ quickColors })} />{/if}
        <SizeControl {tool} value={toolSize(editingBrush)} color={editingBrush.color} opacity={brush.highlighterOpacity} onchange={setSize} />
        {#if tool === 'highlighter'}<label class="palette-range"><span>{$t("불투명도")}</span><input aria-label={$t("형광펜 불투명도")} type="range" min="10" max="80" step="1" value={Math.round(brush.highlighterOpacity * 100)} oninput={(e) => updateBrush({ highlighterOpacity: Number(e.currentTarget.value) / 100 })} /><output>{Math.round(brush.highlighterOpacity * 100)}%</output></label>{/if}
        <div class="brush-reset"><span>{textEntry ? $t("편집 중인 글에 적용됩니다.") : $t("현재 도구에만 적용됩니다.")}</span><button onclick={resetBrush}>{$t("기본 펜으로")}</button></div>
        <div class="preset-save"><select aria-label={$t("저장할 프리셋")} bind:value={presetSlot}>{#each settings.presets as preset, i}<option value={i}>{i + 1}. {preset.name}</option>{/each}</select><button onclick={savePreset}>{$t("현재 도구 저장")}</button></div>
        {#if presetStatus}<p role="status">{presetStatus}</p>{/if}
        <p>{$t("1–6 색상 · [ ] 굵기 · Shift+1–3 프리셋")}</p>
      </div>
    {/snippet}
  </ToolbarFrame>
  <div class="mode-hint"><span class="hint-dot"></span><strong>{$t(TOOL_LABELS[tool])}</strong><span class="hint-separator"></span>{#if tool === 'text'}{$t("빈 곳에 새 글 · 기존 글을 클릭해 수정")}{:else if tool === 'eraser'}{$t("그림을 문지르면 획 단위로 지워집니다")}{:else if tool === 'arrow'}{$t("드래그로 연결 · Shift로 45° 방향 맞추기")}{:else if tool === 'highlighter'}{$t("글자를 가리지 않고 강조하세요")}{:else}{$t("직접 지우기 전까지 남아 있습니다")}{/if}<span class="hint-count">{$t("{0}개", [annotationCount])}</span></div>
{:else if !native}
  <div class="preview-resume"><button class="primary" onclick={() => bridge.setMode('draw').then(acceptState)}>{$t("다시 그리기")}</button><span>{$t("그림은 유지됩니다 ·")} {currentDisplay?.name ?? $t("미리보기")}</span></div>
{/if}

{#if textEntry}
  <div bind:this={textEditor} class="text-editor" style:width={`${editorWidth()}px`} style:left={`${textEntry.x}px`} style:top={`${textEntry.y}px`}>
    <div class="text-editor-heading"><button aria-label={$t("텍스트 이동")} title={$t("드래그 또는 방향키로 이동 · Shift+방향키 10px")} onpointerdown={startTextMove} onpointermove={moveText} onpointerup={endTextMove} onpointercancel={endTextMove} onlostpointercapture={endTextMove} onkeydown={textMoveKey}><Icon name="grip" size={15} />{$t("드래그로 이동")}</button><span>{textEntry.original ? $t("텍스트 수정") : $t("새 텍스트")}</span></div>
    <textarea bind:this={textarea} bind:value={textEntry.value} aria-label={$t("화면에 입력할 텍스트")} placeholder={$t("텍스트 입력")} wrap="off" maxlength="10000" rows={Math.min(8, textEntry.value.split('\n').length + 1)} spellcheck="false" style:width={`${editorWidth()}px`} style:color={textEntry.color} style:font-family={FONT_FAMILY} style:font-size={`${textEntry.fontSize}px`} style:line-height={TEXT_LINE_HEIGHT} onkeydown={textKey} oncompositionstart={() => composing = true} oncompositionend={() => { composing = false; compositionEndedAt = performance.now(); }}></textarea>
    <div class="text-format" aria-label={$t("편집 중인 텍스트 서식")}>
      <div class="text-format-colors">{#each settings.quickColors as color}<button aria-label={$t("텍스트 색상 {0}", [color])} aria-pressed={textEntry.color === color} class:chosen={textEntry.color === color} style:background={color} onclick={() => changeColor(color)}></button>{/each}</div>
      <label>{$t("크기")} <input aria-label={$t("편집 중인 글자 크기")} type="number" min="8" max="144" step="2" value={textEntry.fontSize} oninput={(e) => { const n = Number(e.currentTarget.value); if (n >= 8 && n <= 144) setSize(n); }} onchange={(e) => { if (textEntry) { setSize(Number(e.currentTarget.value) || textEntry.fontSize); e.currentTarget.value = String(textEntry.fontSize); } }} /><span>px</span></label>
    </div>
    <div class="text-editor-actions"><span>{textEntry.original ? $t("수정 중 · ") : ''}{$t("Enter 완료 · Shift+Enter 줄바꿈 · Esc 취소")}</span><button aria-label={$t("텍스트 취소")} onclick={cancelText}><Icon name="close" size={16} /></button><button class="text-confirm" aria-label={$t("텍스트 완료")} onclick={commitText}><Icon name="check" size={16} />{$t("완료")}</button></div>
  </div>
{/if}
{#if error}<div class="overlay-error error-box" role="alert"><span>{$t(error)}</span><button onclick={() => action(() => bridge.setMode('interact'))}>{$t("앱 조작으로 돌아가기")}</button><button aria-label={$t("알림 닫기")} onclick={() => error = ''}><Icon name="close" size={16} /></button></div>{/if}
