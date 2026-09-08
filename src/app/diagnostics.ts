import { invoke, isTauri } from '@tauri-apps/api/core';

const KINDS = ['ready', 'heartbeat', 'state', 'scene', 'canvas_context_lost', 'canvas_context_restored', 'error', 'unhandled_rejection', 'pagehide'] as const;
export type DiagnosticKind = typeof KINDS[number];
export interface DiagnosticMetrics {
  mode?: 'draw' | 'interact';
  revision?: number;
  sceneRevision?: number;
  annotationCount?: number;
  canvasWidth?: number;
  canvasHeight?: number;
  viewportWidth?: number;
  viewportHeight?: number;
  dpr?: number;
  focused?: boolean;
  visible?: boolean;
  opaqueBackground?: boolean;
}
export interface DiagnosticEvent extends DiagnosticMetrics { kind: DiagnosticKind }
type Send = (event: DiagnosticEvent) => Promise<unknown>;

/** Rebuild a fixed schema; never forward an Error, DOM event, string, or arbitrary payload. */
export function sanitizeDiagnostic(value: unknown): DiagnosticEvent | null {
  if (!value || typeof value !== 'object') return null;
  const input = value as Record<string, unknown>;
  if (!KINDS.includes(input.kind as DiagnosticKind)) return null;
  const event: DiagnosticEvent = { kind: input.kind as DiagnosticKind };
  if (input.mode === 'draw' || input.mode === 'interact') event.mode = input.mode;
  for (const key of ['revision', 'sceneRevision', 'annotationCount', 'canvasWidth', 'canvasHeight', 'viewportWidth', 'viewportHeight'] as const) {
    const value = input[key];
    const limit = key === 'revision' || key === 'sceneRevision' ? Number.MAX_SAFE_INTEGER : key === 'annotationCount' ? 1_000_000 : 65_536;
    if (typeof value === 'number' && Number.isSafeInteger(value) && value >= 0 && value <= limit) event[key] = value;
  }
  if (typeof input.dpr === 'number' && Number.isFinite(input.dpr) && input.dpr > 0 && input.dpr <= 16) event.dpr = input.dpr;
  for (const key of ['focused', 'visible', 'opaqueBackground'] as const) {
    if (typeof input[key] === 'boolean') event[key] = input[key];
  }
  return event;
}

/** A maximum of nine pending records plus one IPC; noisy kinds send at most once per second. */
export class DiagnosticReporter {
  private pending = new Map<DiagnosticKind, DiagnosticEvent>();
  private lastSent = new Map<DiagnosticKind, number>();
  private inFlight = false;
  private disposed = false;
  private timer: ReturnType<typeof setTimeout> | undefined;
  constructor(private send: Send, private now: () => number = () => performance.now()) {}

  report(value: unknown): void {
    if (this.disposed) return;
    const event = sanitizeDiagnostic(value);
    if (!event) return;
    this.pending.set(event.kind, event);
    this.flush();
  }
  dispose(): void {
    this.disposed = true;
    this.pending.clear();
    this.lastSent.clear();
    clearTimeout(this.timer);
    this.timer = undefined;
  }
  private flush(): void {
    if (this.disposed || this.inFlight) return;
    clearTimeout(this.timer);
    this.timer = undefined;
    const now = this.now();
    let delay = Infinity;
    for (const [kind, event] of this.pending) {
      const wait = Math.max(0, (this.lastSent.get(kind) ?? -Infinity) + 1_000 - now);
      if (wait > 0) { delay = Math.min(delay, wait); continue; }
      this.pending.delete(kind);
      this.lastSent.set(kind, now);
      this.inFlight = true;
      // A diagnostic transport failure is intentionally not reported through this logger.
      void Promise.resolve().then(() => { if (!this.disposed) return this.send(event); }).catch(() => {}).finally(() => {
        this.inFlight = false;
        this.flush();
      });
      return;
    }
    if (Number.isFinite(delay)) this.timer = setTimeout(() => this.flush(), delay);
  }
}

interface SessionOptions {
  enabled: boolean;
  events: EventTarget;
  readMetrics: () => DiagnosticMetrics;
  send: Send;
  now?: () => number;
}
export function createDiagnosticSession({ enabled, events, readMetrics, send, now }: SessionOptions) {
  let provider: (() => DiagnosticMetrics) | undefined;
  let disposed = false;
  const listeners: [string, EventListener][] = [];
  let heartbeat: ReturnType<typeof setInterval> | undefined;
  // Read layout metrics only when a rate-limited record is actually sent, not on every error.
  const reporter = enabled ? new DiagnosticReporter((event) => {
    let metrics: DiagnosticMetrics = {};
    try { metrics = readMetrics(); } catch { /* Partial records still identify lifecycle events. */ }
    try { metrics = { ...metrics, ...provider?.() }; } catch { /* Diagnostics must not affect drawing. */ }
    return send(sanitizeDiagnostic({ ...metrics, ...event })!);
  }, now) : undefined;
  function report(kind: DiagnosticKind) {
    if (!reporter || disposed) return;
    reporter.report({ kind });
  }
  function dispose() {
    disposed = true;
    clearInterval(heartbeat);
    listeners.forEach(([name, listener]) => events.removeEventListener(name, listener));
    provider = undefined;
    reporter?.dispose();
  }
  if (enabled) {
    for (const kind of ['error', 'unhandled_rejection', 'pagehide'] as const) {
      const name = kind === 'unhandled_rejection' ? 'unhandledrejection' : kind;
      // Do not read event.error, reason, message, filename, stack, or target.
      const listener: EventListener = () => {
        report(kind);
        if (kind === 'pagehide') clearInterval(heartbeat);
      };
      listeners.push([name, listener]);
      events.addEventListener(name, listener);
    }
    heartbeat = setInterval(() => report('heartbeat'), 10_000);
    report('ready');
  }
  return {
    report,
    registerProvider(next: () => DiagnosticMetrics) {
      if (!enabled || disposed) return () => {};
      provider = next;
      return () => { if (provider === next) provider = undefined; };
    },
    dispose,
  };
}

/** Only the alpha result leaves this function; CSS color strings never enter a diagnostic record. */
export function backgroundHasAlpha(color: string): boolean | undefined {
  const value = color.trim().toLowerCase();
  if (value === 'transparent') return false;
  if (/^rgb\([^/]+\)$/.test(value) && !value.includes(',')) return true;
  if (/^rgb\(\s*[\d.]+\s*,\s*[\d.]+\s*,\s*[\d.]+\s*\)$/.test(value)) return true;
  const alpha = value.match(/(?:rgba\([^)]*,|(?:rgba?|color|lab|lch|oklab|oklch)\([^)]*\/)\s*([\d.]+)(%)?\s*\)$/);
  if (alpha) {
    const number = Number(alpha[1]);
    if (Number.isFinite(number)) return number > 0;
  }
  return undefined;
}
export function opaqueBackground(...elements: (Element | null | undefined)[]): boolean | undefined {
  try {
    const values = elements.filter((element): element is Element => !!element)
      .map((element) => backgroundHasAlpha(getComputedStyle(element).backgroundColor));
    if (values.some((value) => value === true)) return true;
    if (values.length > 0 && values.every((value) => value === false)) return false;
  } catch { /* Unavailable styles are omitted, never guessed. */ }
  return undefined;
}

export function observeCanvasDiagnostics(canvas: EventTarget, report: (kind: DiagnosticKind) => void): () => void {
  const lost = () => report('canvas_context_lost');
  const restored = () => report('canvas_context_restored');
  canvas.addEventListener('contextlost', lost);
  canvas.addEventListener('contextrestored', restored);
  return () => {
    canvas.removeEventListener('contextlost', lost);
    canvas.removeEventListener('contextrestored', restored);
  };
}

let session: ReturnType<typeof createDiagnosticSession> | undefined;
export function installDiagnostics(): () => void {
  if (!isTauri() || session) return () => {};
  const current = createDiagnosticSession({
    enabled: true,
    events: window,
    readMetrics: () => ({
      viewportWidth: window.innerWidth,
      viewportHeight: window.innerHeight,
      dpr: window.devicePixelRatio,
      focused: document.hasFocus(),
      visible: document.visibilityState === 'visible',
      opaqueBackground: opaqueBackground(document.documentElement, document.body, document.getElementById('app')),
    }),
    send: (event) => invoke('record_diagnostic', { event }),
  });
  session = current;
  return () => { current.dispose(); if (session === current) session = undefined; };
}
export function reportDiagnostic(kind: DiagnosticKind): void { session?.report(kind); }
export function registerDiagnosticMetrics(provider: () => DiagnosticMetrics): () => void {
  return session?.registerProvider(provider) ?? (() => {});
}
