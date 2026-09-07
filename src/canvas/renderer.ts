import type { Annotation, StrokeAnnotation, TextAnnotation } from '../shared/types';
import { FONT_FAMILY, TEXT_LINE_HEIGHT, finiteStrokePoints, isFinitePoint, visitStrokePath } from './geometry';

// Fade snapshots are temporary: retain at most this many objects/points and 2 seconds.
// Objects beyond the budget disappear immediately on clear; native undo still retains
// the complete scene. Prefer recent objects if multiple clears overlap the budget.
const MAX_FADE_OBJECTS = 2_000;
const MAX_FADE_POINTS = 200_000;
const MAX_FADE_DURATION_MS = 2_000;
interface FadingAnnotation { annotation: Annotation; startedAt: number; duration: number }

/** An event-driven view of the native scene. This class never edits scene/history. */
export class CanvasRenderer {
  private readonly context: CanvasRenderingContext2D;
  private scene: readonly Annotation[] = [];
  private preview: StrokeAnnotation | null = null;
  private textPreview: TextAnnotation | null = null;
  private fades: FadingAnnotation[] = [];
  private frame: number | null = null;
  private disposed = false;
  private dpr = 1;

  constructor(private readonly canvas: HTMLCanvasElement) {
    const context = canvas.getContext('2d');
    if (!context) throw new Error('Canvas 2D 렌더링을 사용할 수 없습니다.');
    this.context = context;
  }

  resize(logicalWidth: number, logicalHeight: number, dpr: number): void {
    if (this.disposed) return;
    if (![logicalWidth, logicalHeight, dpr].every(Number.isFinite) || logicalWidth < 0 || logicalHeight < 0 || dpr <= 0) {
      throw new RangeError('Canvas dimensions and DPR must be finite, with nonnegative dimensions and positive DPR.');
    }
    const width = Math.ceil(logicalWidth * dpr);
    const height = Math.ceil(logicalHeight * dpr);
    if (this.canvas.width !== width) this.canvas.width = width;
    if (this.canvas.height !== height) this.canvas.height = height;
    this.canvas.style.width = `${logicalWidth}px`;
    this.canvas.style.height = `${logicalHeight}px`;
    this.dpr = dpr;
    this.schedule();
  }

  setScene(annotations: readonly Annotation[]): void {
    if (this.disposed) return;
    this.scene = annotations.slice();
    const restored = new Set(annotations.map(({ id }) => id));
    this.fades = this.fades.filter(({ annotation }) => !restored.has(annotation.id));
    this.schedule();
  }

  setPreview(stroke: StrokeAnnotation | null): void {
    if (this.disposed) return;
    this.preview = stroke;
    this.schedule();
  }

  setTextPreview(text: TextAnnotation | null): void {
    if (this.disposed) return;
    this.textPreview = text;
    this.schedule();
  }

  /** Call after setScene with the removed objects supplied by the native event. */
  fadeOut(annotations: readonly Annotation[], durationMs: number): void {
    if (this.disposed || annotations.length === 0) return;
    const now = performance.now();
    const incoming = new Set(annotations.map(({ id }) => id));
    this.fades = this.fades.filter(({ annotation, startedAt, duration }) => !incoming.has(annotation.id) && now < startedAt + duration);
    const duration = Number.isFinite(durationMs) ? Math.min(MAX_FADE_DURATION_MS, Math.max(0, durationMs)) : 0;
    if (duration > 0) {
      const live = new Set(this.scene.map(({ id }) => id));
      // Keep newest fades first in the budget. Snapshot coordinates against future mutation.
      let pointCount = 0;
      const next: FadingAnnotation[] = [];
      for (let i = annotations.length - 1; i >= 0 && next.length < MAX_FADE_OBJECTS; i--) {
        const annotation = annotations[i];
        if (live.has(annotation.id)) continue;
        const cost = annotation.kind === 'stroke' ? annotation.points.length : annotation.text.length;
        if (pointCount + cost > MAX_FADE_POINTS) continue;
        pointCount += cost;
        next.push({ annotation: annotation.kind === 'stroke' ? { ...annotation, points: annotation.points.map((point) => ({ ...point })) } : { ...annotation }, startedAt: now, duration });
      }
      next.reverse();
      for (let i = this.fades.length - 1; i >= 0 && next.length < MAX_FADE_OBJECTS; i--) {
        const fade = this.fades[i];
        const cost = fade.annotation.kind === 'stroke' ? fade.annotation.points.length : fade.annotation.text.length;
        if (pointCount + cost > MAX_FADE_POINTS) continue;
        pointCount += cost;
        next.unshift(fade);
      }
      this.fades = next;
    }
    this.schedule();
  }

  dispose(): void {
    if (this.frame !== null) cancelAnimationFrame(this.frame);
    this.frame = null;
    this.disposed = true;
    this.scene = [];
    this.fades = [];
    this.preview = null;
    this.textPreview = null;
    // Release the potentially large backing bitmap when an overlay is unmounted.
    this.canvas.width = 0;
    this.canvas.height = 0;
  }

  private schedule(): void {
    if (this.disposed || this.frame !== null) return;
    this.frame = requestAnimationFrame(() => {
      this.frame = null;
      if (this.disposed) return;
      this.render(performance.now());
    });
  }

  private render(now: number): void {
    const ctx = this.context;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.fades = this.fades.filter(({ startedAt, duration }) => now < startedAt + duration);
    for (const fade of this.fades) this.draw(fade.annotation, Math.max(0, 1 - (now - fade.startedAt) / fade.duration));
    for (const annotation of this.scene) this.draw(annotation, 1);
    if (this.preview) this.draw(this.preview, 1);
    if (this.textPreview) this.draw(this.textPreview, 1);
    ctx.globalAlpha = 1;
    if (this.fades.length > 0) this.schedule();
  }

  private draw(annotation: Annotation, alpha: number): void {
    const ctx = this.context;
    ctx.globalAlpha = alpha;
    ctx.fillStyle = annotation.color;
    if (annotation.kind === 'text') {
      if (!isFinitePoint(annotation) || !Number.isFinite(annotation.fontSize) || annotation.fontSize <= 0) return;
      ctx.font = `${annotation.fontSize}px ${FONT_FAMILY}`;
      ctx.textBaseline = 'top';
      ctx.textAlign = 'left';
      annotation.text.split('\n').forEach((line, index) => {
        ctx.fillText(line, annotation.x, annotation.y + index * annotation.fontSize * TEXT_LINE_HEIGHT);
      });
      return;
    }
    if (!Number.isFinite(annotation.width) || annotation.width <= 0) return;
    const points = finiteStrokePoints(annotation.points);
    if (points.length === 0) return;
    ctx.beginPath();
    if (points.length === 1) {
      ctx.arc(points[0].x, points[0].y, annotation.width / 2, 0, Math.PI * 2);
      ctx.fill();
      return;
    }
    ctx.strokeStyle = annotation.color;
    ctx.lineWidth = annotation.width;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    visitStrokePath(points, {
      move: (point) => ctx.moveTo(point.x, point.y),
      line: (point) => ctx.lineTo(point.x, point.y),
      curve: (control, end) => ctx.quadraticCurveTo(control.x, control.y, end.x, end.y),
    });
    ctx.stroke();
  }
}
