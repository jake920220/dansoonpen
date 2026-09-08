import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { backgroundHasAlpha, createDiagnosticSession, DiagnosticReporter, installDiagnostics, observeCanvasDiagnostics, sanitizeDiagnostic, type DiagnosticEvent, type DiagnosticKind } from './diagnostics';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => undefined), isTauri: vi.fn(() => false) }));
const cleanup: (() => void)[] = [];
beforeEach(() => { vi.useFakeTimers(); vi.setSystemTime(0); });
afterEach(() => { cleanup.splice(0).forEach((stop) => stop()); vi.useRealTimers(); vi.clearAllMocks(); });
const settle = () => vi.advanceTimersByTimeAsync(0);

describe('diagnostic privacy and bounded delivery', () => {
  it('rebuilds only fixed typed fields and rejects unsafe or arbitrary payloads', () => {
    const secret = 'private lecture content /Users/someone/file https://example.test';
    const event = sanitizeDiagnostic({
      kind: 'error', mode: 'draw', revision: 3, sceneRevision: 4, annotationCount: 5,
      canvasWidth: 2560, canvasHeight: 1440, viewportWidth: 1280, viewportHeight: 720,
      dpr: 2, focused: true, visible: false, opaqueBackground: false,
      message: secret, stack: secret, url: secret, key: secret, path: secret,
      color: secret, displayId: secret, x: 100, y: 200, reason: new Error(secret),
    });
    expect(event).toEqual({ kind: 'error', mode: 'draw', revision: 3, sceneRevision: 4, annotationCount: 5, canvasWidth: 2560, canvasHeight: 1440, viewportWidth: 1280, viewportHeight: 720, dpr: 2, focused: true, visible: false, opaqueBackground: false });
    expect(JSON.stringify(event)).not.toContain(secret);
    expect(sanitizeDiagnostic({ kind: secret })).toBeNull();
    expect(sanitizeDiagnostic(secret)).toBeNull();
    expect(sanitizeDiagnostic({ kind: 'state', mode: secret, revision: -1, sceneRevision: Number.MAX_SAFE_INTEGER + 1, annotationCount: 1_000_001, canvasWidth: Infinity, canvasHeight: 65_537, viewportWidth: 4.5, viewportHeight: secret, dpr: NaN, focused: secret })).toEqual({ kind: 'state' });
  });

  it('keeps one IPC in flight and coalesces a flood to the latest record per kind', async () => {
    let resolveFirst!: () => void;
    const send = vi.fn<(event: DiagnosticEvent) => Promise<void>>()
      .mockImplementationOnce(() => new Promise<void>((resolve) => { resolveFirst = resolve; }))
      .mockResolvedValue(undefined);
    const reporter = new DiagnosticReporter(send, () => Date.now()); cleanup.push(() => reporter.dispose());
    reporter.report({ kind: 'ready' }); await settle();
    const kinds: DiagnosticKind[] = ['ready', 'heartbeat', 'state', 'scene', 'canvas_context_lost', 'canvas_context_restored', 'error', 'unhandled_rejection', 'pagehide'];
    for (let revision = 0; revision < 10_000; revision++) {
      for (const kind of kinds) reporter.report({ kind, revision });
    }
    await vi.advanceTimersByTimeAsync(30_000);
    expect(send).toHaveBeenCalledTimes(1);
    resolveFirst(); await settle();
    expect(send).toHaveBeenCalledTimes(10);
    expect(send.mock.calls.slice(1).map(([event]) => event.revision)).toEqual(Array(9).fill(9_999));
  });

  it('rate-limits repeated errors and state changes while retaining the newest pending state', async () => {
    const send = vi.fn(async (_event: DiagnosticEvent) => {});
    const reporter = new DiagnosticReporter(send, () => Date.now()); cleanup.push(() => reporter.dispose());
    reporter.report({ kind: 'state', revision: 0 }); reporter.report({ kind: 'error' }); await settle();
    for (let revision = 1; revision <= 5_000; revision++) {
      reporter.report({ kind: 'state', revision }); reporter.report({ kind: 'error' });
    }
    await vi.advanceTimersByTimeAsync(999); expect(send).toHaveBeenCalledTimes(2);
    await vi.advanceTimersByTimeAsync(1); expect(send).toHaveBeenCalledTimes(4);
    expect(send.mock.calls[2][0]).toEqual({ kind: 'state', revision: 5_000 });
    await vi.advanceTimersByTimeAsync(60_000); expect(send).toHaveBeenCalledTimes(4);
  });

  it('swallows transport rejection without retries or a recursive diagnostic error', async () => {
    const send = vi.fn(async (_event: DiagnosticEvent) => { throw new Error('private IPC failure'); });
    const reporter = new DiagnosticReporter(send, () => Date.now()); cleanup.push(() => reporter.dispose());
    reporter.report({ kind: 'error' }); await settle();
    await vi.advanceTimersByTimeAsync(60_000); expect(send).toHaveBeenCalledTimes(1);
    reporter.report({ kind: 'heartbeat' }); await settle(); expect(send).toHaveBeenCalledTimes(2);
    expect(send.mock.calls.map(([event]) => event.kind)).toEqual(['error', 'heartbeat']);
  });

  it('dispose drops queued records and prevents new work after an in-flight call completes', async () => {
    let resolveFirst!: () => void;
    const send = vi.fn(() => new Promise<void>((resolve) => { resolveFirst = resolve; }));
    const reporter = new DiagnosticReporter(send, () => Date.now());
    reporter.report({ kind: 'ready' }); await settle();
    reporter.report({ kind: 'state' }); reporter.dispose(); resolveFirst();
    reporter.report({ kind: 'heartbeat' }); await vi.advanceTimersByTimeAsync(60_000);
    expect(send).toHaveBeenCalledTimes(1);
    const neverSent = vi.fn(async () => {});
    const immediate = new DiagnosticReporter(neverSent); immediate.report({ kind: 'ready' }); immediate.dispose();
    await settle(); expect(neverSent).not.toHaveBeenCalled();
  });
});

describe('document diagnostic lifecycle', () => {
  it('observes canvas context loss/restoration without changing recovery behavior and removes listeners', () => {
    const canvas = new EventTarget(), report = vi.fn();
    const stop = observeCanvasDiagnostics(canvas, report);
    const lost = new Event('contextlost', { cancelable: true });
    canvas.dispatchEvent(lost); canvas.dispatchEvent(new Event('contextrestored'));
    expect(lost.defaultPrevented).toBe(false);
    expect(report.mock.calls).toEqual([['canvas_context_lost'], ['canvas_context_restored']]);
    stop(); canvas.dispatchEvent(new Event('contextlost')); expect(report).toHaveBeenCalledTimes(2);
  });

  it('captures a ten-second heartbeat with current provider metrics and releases the provider/listeners', async () => {
    const events = new EventTarget();
    const send = vi.fn(async (_event: DiagnosticEvent) => {});
    const session = createDiagnosticSession({ enabled: true, events, readMetrics: () => ({ viewportWidth: 1280, viewportHeight: 720, dpr: 2, focused: false, visible: true }), send, now: () => Date.now() });
    cleanup.push(session.dispose);
    let count = 2;
    const stopProvider = session.registerProvider(() => ({ mode: 'draw', annotationCount: count, canvasWidth: 2560, canvasHeight: 1440 }));
    await settle(); expect(send.mock.calls[0][0].kind).toBe('ready');
    await vi.advanceTimersByTimeAsync(9_999); expect(send).toHaveBeenCalledTimes(1);
    count = 7; await vi.advanceTimersByTimeAsync(1);
    expect(send.mock.calls[1][0]).toEqual({ kind: 'heartbeat', mode: 'draw', annotationCount: 7, canvasWidth: 2560, canvasHeight: 1440, viewportWidth: 1280, viewportHeight: 720, dpr: 2, focused: false, visible: true });
    stopProvider(); await vi.advanceTimersByTimeAsync(10_000);
    expect(send.mock.calls[2][0].annotationCount).toBeUndefined();
    session.dispose(); events.dispatchEvent(new Event('error')); await vi.advanceTimersByTimeAsync(60_000);
    expect(send).toHaveBeenCalledTimes(3);
  });

  it('ignores all error payload fields and does not suppress browser default handling', async () => {
    const events = new EventTarget();
    const send = vi.fn(async (_event: DiagnosticEvent) => {});
    const session = createDiagnosticSession({ enabled: true, events, readMetrics: () => ({}), send, now: () => Date.now() }); cleanup.push(session.dispose);
    await settle();
    for (const name of ['error', 'unhandledrejection']) {
      const event = new Event(name, { cancelable: true });
      for (const key of ['error', 'reason', 'message', 'filename', 'stack']) {
        Object.defineProperty(event, key, { get() { throw new Error('Event contents must never be accessed'); } });
      }
      events.dispatchEvent(event); expect(event.defaultPrevented).toBe(false);
    }
    await settle();
    expect(send.mock.calls.slice(1)).toEqual([[{ kind: 'error' }], [{ kind: 'unhandled_rejection' }]]);
    events.dispatchEvent(new Event('pagehide')); await settle();
    expect(send.mock.calls[3][0]).toEqual({ kind: 'pagehide' });
    await vi.advanceTimersByTimeAsync(30_000); expect(send).toHaveBeenCalledTimes(4);
  });

  it('coalesces expensive metric reads as well as IPC during an error flood', async () => {
    const events = new EventTarget(), readMetrics = vi.fn(() => ({ canvasWidth: 2560 }));
    const send = vi.fn(async (_event: DiagnosticEvent) => {});
    const session = createDiagnosticSession({ enabled: true, events, readMetrics, send, now: () => Date.now() }); cleanup.push(session.dispose);
    await settle();
    for (let index = 0; index < 10_000; index++) events.dispatchEvent(new Event('error'));
    await settle();
    expect(readMetrics).toHaveBeenCalledTimes(2); expect(send).toHaveBeenCalledTimes(2);
    await vi.advanceTimersByTimeAsync(1_000);
    expect(readMetrics).toHaveBeenCalledTimes(3); expect(send).toHaveBeenCalledTimes(3);
  });

  it('does nothing in browser preview and tolerates unavailable metrics', async () => {
    const events = new EventTarget(), send = vi.fn(async () => {}), readMetrics = vi.fn(() => ({}));
    const disabled = createDiagnosticSession({ enabled: false, events, readMetrics, send }); cleanup.push(disabled.dispose);
    disabled.report('state'); disabled.registerProvider(() => { throw new Error('disabled'); });
    events.dispatchEvent(new Event('error')); await vi.advanceTimersByTimeAsync(30_000);
    expect(send).not.toHaveBeenCalled(); expect(readMetrics).not.toHaveBeenCalled();
    installDiagnostics()(); expect(invoke).not.toHaveBeenCalled();
    const partial = createDiagnosticSession({ enabled: true, events, readMetrics: () => { throw new Error('unavailable'); }, send }); cleanup.push(partial.dispose);
    partial.registerProvider(() => { throw new Error('detached canvas'); }); partial.report('state'); await settle();
    expect(send.mock.calls).toEqual([[{ kind: 'ready' }], [{ kind: 'state' }]]);
  });

  it('reports background alpha as a boolean without retaining CSS color strings', () => {
    for (const color of ['transparent', 'rgba(0, 0, 0, 0)', 'rgb(0 0 0 / 0%)']) expect(backgroundHasAlpha(color)).toBe(false);
    for (const color of ['rgb(0, 0, 0)', 'rgb(128 128 128)', 'rgba(0, 0, 0, 0.2)', 'rgb(0 0 0 / 10%)', 'color(srgb 0 0 0 / 0.1)']) expect(backgroundHasAlpha(color)).toBe(true);
    expect(backgroundHasAlpha('unavailable')).toBeUndefined();
  });
});
