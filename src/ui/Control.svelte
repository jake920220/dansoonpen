<script lang="ts">
  import { onMount } from 'svelte';
  import { bridge, message, native, shortcutLabel } from '../app/bridge';
  import { DEFAULT_SETTINGS, type AppState, type AppSettings } from '../shared/types';
  import Icon from './Icon.svelte';
  import ColorPalette from './ColorPalette.svelte';
  import ShortcutRecorder from './ShortcutRecorder.svelte';
  import { SettingsWriter } from '../app/settings-writer';
  import { isSettingsShortcut } from '../app/shortcuts';
  let appState = $state.raw<AppState | null>(null);
  let draft = $state<AppSettings>(structuredClone(DEFAULT_SETTINGS));
  let dirty = $state(false);
  let tab = $state<'start' | 'settings' | 'about'>(new URLSearchParams(location.search).get('tab') === 'settings' ? 'settings' : 'start');
  let busy = $state(false);
  let error = $state('');
  let saved = $state(false);
  let savingAppearance = $state(false);
  let appearanceError = $state('');
  let optimistic: Partial<AppSettings> = {};
  let captureEdits: Promise<void> = Promise.resolve();
  let recording = $state(false);
  let ready = $derived(appState !== null && !busy);
  const selected = $derived(appState?.displays.find((d) => d.id === appState?.activeDisplayId));

  function accept(next: AppState) {
    if (appState && next.revision < appState.revision) return;
    appState = next;
    const shortcuts = dirty ? { toggleShortcut: draft.toggleShortcut, clearShortcut: draft.clearShortcut } : {};
    draft = { ...structuredClone(next.settings), ...optimistic, ...shortcuts };
  }
  const writer = new SettingsWriter(bridge.updateSettings, accept, (pending) => {
    optimistic = pending; savingAppearance = Object.keys(pending).length > 0;
    if (appState) accept(appState);
  }, (e) => { appearanceError = message(e); });
  function appearance(patch: Partial<AppSettings>) { appearanceError = ''; writer.update(patch); }
  function capture(active: boolean): Promise<void> {
    recording = active;
    const request = captureEdits.then(() => bridge.captureShortcut(active));
    captureEdits = request.catch((e) => { error = message(e); });
    return request;
  }
  function keyboard(event: KeyboardEvent) {
    if (!recording && isSettingsShortcut(event)) { event.preventDefault(); tab = 'settings'; }
  }
  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    let stopSettings: (() => void) | undefined;
    void (async () => {
      try {
        const off = await bridge.onState((next) => { if (!disposed) accept(next); });
        if (disposed) { off(); return; }
        unlisten = off;
        const stop = await bridge.onOpenSettings(() => { if (!disposed) tab = 'settings'; });
        if (disposed) { stop(); return; }
        stopSettings = stop;
        const next = await bridge.getState();
        if (!disposed) accept(next);
      } catch (e) { if (!disposed) error = message(e); }
    })();
    return () => { disposed = true; unlisten?.(); stopSettings?.(); void writer.flush(); void capture(false).catch(() => {}); };
  });
  async function action(fn: () => Promise<unknown>) {
    if (busy) return;
    busy = true; error = ''; saved = false;
    try { await fn(); } catch (e) { error = message(e); } finally { busy = false; }
  }
  async function start() {
    if (!native) { location.href = '/?view=overlay&display=preview-display'; return; }
    await action(async () => accept(await bridge.setMode('draw')));
  }
  function change() { dirty = true; saved = false; }
  async function save() {
    await action(async () => {
      await captureEdits;
      const next = await bridge.updateSettings({ toggleShortcut: draft.toggleShortcut, clearShortcut: draft.clearShortcut });
      dirty = false; accept(next); saved = true;
    });
  }
</script>

<svelte:window onkeydown={keyboard} onblur={() => { if (recording) void capture(false).catch(() => {}); }} />

<div class="control-shell">
  <aside class="sidebar">
    <div class="brand"><span class="brand-mark"><Icon name="pen" size={24} /></span><span>my brush<span class="brand-caption">SCREEN ANNOTATION</span></span></div>
    <nav aria-label="주 메뉴">
      <button class:active={tab === 'start'} onclick={() => tab = 'start'}><Icon name="pen" />시작하기</button>
      <button class:active={tab === 'settings'} onclick={() => tab = 'settings'}><Icon name="settings" />설정</button>
      <button class:active={tab === 'about'} onclick={() => tab = 'about'}><Icon name="info" />앱 정보</button>
    </nav>
    <div class="sidebar-bottom"><span class="status-dot" class:drawing={appState?.mode === 'draw'}></span>{appState?.mode === 'draw' ? '그리는 중' : appState ? '사용 준비 완료' : '연결 중'}<span class="version">v0.1.0</span></div>
  </aside>
  <main class="control-main">
    <header class="page-heading"><span>{tab === 'start' ? '설명에 필요한 만큼만.' : tab === 'settings' ? '나에게 맞는 도구로.' : '함께 만드는 작은 도구.'}</span><span class="local-label">{native ? '오프라인으로 작동' : '브라우저 미리보기'}</span></header>
    {#if error || appState?.error}<div class="error-box" role="alert">{error || appState?.error}</div>{/if}
    {#if tab === 'start'}
      <section class="welcome">
        <div class="eyebrow">YOUR SCREEN, YOUR CANVAS</div>
        <h1>화면 위에,<br /><span>설명을 더하세요.</span></h1>
        <p>강조하고 싶은 곳에 가볍게 그려 보세요.<br />직접 지우기 전까지, 설명은 그대로 남습니다.</p>
        <button class="primary" disabled={!ready} onclick={start}><Icon name="pen" size={18} />그리기 시작<kbd>{shortcutLabel(appState?.settings.toggleShortcut ?? DEFAULT_SETTINGS.toggleShortcut)}</kbd></button>
        <div class="ink-sample" aria-hidden="true"><svg viewBox="0 0 240 140"><path class="sample-line" d="M20 103Q64 65 91 85T173 57T215 77" /><path class="sample-underline" d="M28 117Q116 122 212 99" /><circle cx="205" cy="28" r="15" /><path class="sample-spark" d="m168 17 6 5m53 30 7 1m-50-48-1 7" /></svg><span>이 순간을 짚어 주세요</span></div>
      </section>
      <section class="display-card">
        <div class="card-icon"><Icon name="monitor" size={24} /></div>
        <div class="display-details"><h2>그릴 화면</h2><p>{selected ? `${Math.round(selected.width)} × ${Math.round(selected.height)} · ${Math.round(selected.scaleFactor * 100)}% 배율` : '화면을 찾고 있습니다'}</p></div>
        <select aria-label="그릴 화면 선택" value={appState?.activeDisplayId ?? ''} disabled={!ready} onchange={(e) => action(async () => accept(await bridge.selectDisplay(e.currentTarget.value)))}>{#each appState?.displays.filter((d) => d.connected) ?? [] as display}<option value={display.id}>{display.name}{display.isPrimary ? ' · 기본' : ''}</option>{/each}</select>
      </section>
      <section class="quick-guide" aria-label="사용 순서">
        <div><span class="step-number">01</span><h3>그리고</h3><p>펜·색상·굵기를 골라<br />필요한 곳에 표시하세요.</p></div>
        <div><span class="step-number">02</span><h3>그대로 두고</h3><p><kbd>Esc</kbd>로 앱 조작에 복귀해도<br />그림은 남아 있습니다.</p></div>
        <div><span class="step-number">03</span><h3>가볍게 지우세요</h3><p><kbd>{shortcutLabel(appState?.settings.clearShortcut ?? DEFAULT_SETTINGS.clearShortcut)}</kbd><br />부드럽게 지워집니다.</p></div>
      </section>
      <div class="sharing-note"><Icon name="monitor" size={17} /><p>화면공유는 <strong>모니터 전체</strong>로 선택해 주세요. 특정 앱 창만 공유하면 그림이 전달되지 않을 수 있습니다.</p></div>
    {:else if tab === 'settings'}
      <div class="settings-heading"><h1>설정</h1><p>색상과 굵기는 바꾸는 즉시 저장되고 다음 필기에 적용됩니다.</p></div>
      {#if appearanceError}<div class="error-box" role="alert">{appearanceError}</div>{/if}
      <form onsubmit={(e) => { e.preventDefault(); void save(); }}>
        <fieldset disabled={!ready}>
        <section class="settings-section"><h2><Icon name="pen" size={18} />기본 펜</h2>
          <div class="pen-preview" style:--ink={draft.color}><svg viewBox="0 0 360 70" aria-hidden="true"><path d="M18 47C60 10 70 64 112 34S159 57 210 27S277 40 337 23" fill="none" stroke="currentColor" stroke-width={draft.width} stroke-linecap="round" /></svg><span>{draft.width}px <code>{draft.color.toUpperCase()}</code></span></div>
          <div class="settings-palette"><ColorPalette value={draft.color} colors={draft.quickColors} onchange={(color) => appearance({ color })} onpalettechange={(quickColors) => appearance({ quickColors })} /></div>
          <div class="setting-row"><label for="pen-width">펜 굵기</label><div class="range-value"><input id="pen-width" type="range" min="1" max="32" step="1" value={draft.width} oninput={(e) => appearance({ width: Number(e.currentTarget.value) })} /><output>{draft.width}px</output></div></div>
          <div class="setting-row"><label for="text-size">글자 크기</label><div class="range-value"><input id="text-size" type="range" min="12" max="96" step="2" value={draft.textSize} oninput={(e) => appearance({ textSize: Number(e.currentTarget.value) })} /><output>{draft.textSize}px</output></div></div>
          <p class="appearance-status" role="status">{savingAppearance ? '저장 중…' : appearanceError ? '변경을 저장하지 못했습니다.' : '자동 저장 · 기존 필기의 색상은 유지됩니다.'}</p>
        </section>
        <section class="settings-section"><h2><Icon name="keyboard" size={18} />단축키</h2>
          <div class="setting-row"><label for="toggle-shortcut">그리기 / 앱 조작</label><ShortcutRecorder id="toggle-shortcut" value={draft.toggleShortcut} onchange={(value) => { draft.toggleShortcut = value; change(); }} {capture} /></div>
          <div class="setting-row"><label for="clear-shortcut">전체 지우기</label><ShortcutRecorder id="clear-shortcut" value={draft.clearShortcut} onchange={(value) => { draft.clearShortcut = value; change(); }} {capture} /></div>
          <p class="field-hint">입력칸을 클릭하고 원하는 키를 함께 누른 뒤 놓으세요. ‘단축키 적용’을 누르면 사용됩니다. 다른 앱의 단축키·특수문자 입력과 겹치면 조합을 바꿔 주세요.</p>
          <div class="shortcut-actions"><button type="button" class="secondary" onclick={() => { draft.toggleShortcut = DEFAULT_SETTINGS.toggleShortcut; draft.clearShortcut = DEFAULT_SETTINGS.clearShortcut; change(); }}>왼손 추천 조합</button><span class="success" role="status">{saved ? '단축키를 적용했습니다.' : ''}</span><button class="primary" type="submit" disabled={!ready || !dirty || recording}>단축키 적용</button></div>
        </section>
        <section class="settings-section"><label class="checkbox-row"><span><strong>애니메이션 줄이기</strong><small>지우기 명령을 누르면 그림을 즉시 지웁니다.</small></span><input type="checkbox" checked={draft.reduceMotion} onchange={(e) => appearance({ reduceMotion: e.currentTarget.checked })} /></label></section>
        <div class="settings-actions"><button type="button" class="secondary" onclick={() => { const { toggleShortcut, clearShortcut, ...defaults } = structuredClone(DEFAULT_SETTINGS); appearance(defaults); draft.toggleShortcut = toggleShortcut; draft.clearShortcut = clearShortcut; change(); }}>기본값으로</button></div>
        </fieldset>
      </form>
    {:else}
      <section class="about-panel"><span class="brand-mark large"><Icon name="pen" size={36} /></span><h1>My Brush</h1><p>설명을 오래 남기는 화면 드로잉 도구</p><span class="about-version">VERSION 0.1.0</span>
        <div class="about-details"><div><span>원저작자</span><strong>김준현</strong></div><div><span>라이선스</span><strong>Apache License 2.0</strong></div><div><span>원본 저장소</span><strong>공개 준비 중</strong></div></div>
        <p class="about-note">Copyright 2026 김준현<br />재배포 시 관련 저작권·출처 고지를 유지해 주세요.<br />라이선스 전문과 고지는 설치 배포물에 함께 제공됩니다.</p>
        <p class="about-note">화면을 녹화하거나 서버로 전송하지 않습니다.<br />설정만 이 기기에 저장하며, 그림은 앱을 종료하면 사라집니다.</p>
        <button class="secondary" onclick={() => action(bridge.quit)} disabled={busy}><Icon name="power" size={16} />앱 종료</button>
      </section>
    {/if}
  </main>
</div>
