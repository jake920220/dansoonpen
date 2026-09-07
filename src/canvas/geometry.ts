import type { Annotation, Point, TextAnnotation } from '../shared/types';

export const FONT_FAMILY = '-apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans KR", sans-serif';
export const TEXT_LINE_HEIGHT = 1.3;
export type MeasureText = (text: string, fontSize: number) => number;

export function isFinitePoint(point: Point): boolean {
  return Number.isFinite(point.x) && Number.isFinite(point.y);
}

/** Native scenes are already validated; avoid a new 100k-point array on each frame. */
export function finiteStrokePoints(points: readonly Point[]): readonly Point[] {
  return points.every(isFinitePoint) ? points : points.filter(isFinitePoint);
}

/** Keep local pointer samples compact; force=true preserves the pointer-up endpoint. */
export function appendStrokePoint(points: Point[], point: Point, minSpacing = 0.5, force = false): boolean {
  if (!isFinitePoint(point)) return false;
  const last = points[points.length - 1];
  const distance = last ? Math.hypot(point.x - last.x, point.y - last.y) : Infinity;
  if (distance === 0 || (!force && distance < Math.max(0, minSpacing))) return false;
  points.push({ x: point.x, y: point.y });
  return true;
}

export function distanceToSegment(point: Point, from: Point, to: Point): number {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const lengthSquared = dx * dx + dy * dy;
  if (lengthSquared === 0) return Math.hypot(point.x - from.x, point.y - from.y);
  const t = Math.max(0, Math.min(1, ((point.x - from.x) * dx + (point.y - from.y) * dy) / lengthSquared));
  return Math.hypot(point.x - from.x - t * dx, point.y - from.y - t * dy);
}

function cross(a: Point, b: Point, c: Point): number {
  return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
}

function segmentDistance(a: Point, b: Point, c: Point, d: Point): number {
  // Proper crossing, then endpoint distances also handle collinear and degenerate segments.
  if (cross(a, b, c) * cross(a, b, d) < 0 && cross(c, d, a) * cross(c, d, b) < 0) return 0;
  return Math.min(distanceToSegment(a, c, d), distanceToSegment(b, c, d),
    distanceToSegment(c, a, b), distanceToSegment(d, a, b));
}

/** The renderer and hit testing share these midpoint quadratic paths. */
export function visitStrokePath(points: readonly Point[], visitor: {
  move(point: Point): void;
  line(point: Point): void;
  curve(control: Point, end: Point): void;
}): void {
  if (points.length === 0) return;
  visitor.move(points[0]);
  if (points.length === 1) return;
  for (let i = 1; i < points.length - 1; i++) {
    const current = points[i];
    const next = points[i + 1];
    visitor.curve(current, { x: (current.x + next.x) / 2, y: (current.y + next.y) / 2 });
  }
  visitor.line(points[points.length - 1]);
}

function flattenQuadratic(start: Point, control: Point, end: Point, visit: (a: Point, b: Point) => void, depth = 0): void {
  if (depth >= 12 || distanceToSegment(control, start, end) <= 0.25) {
    visit(start, end);
    return;
  }
  const a = { x: (start.x + control.x) / 2, y: (start.y + control.y) / 2 };
  const b = { x: (control.x + end.x) / 2, y: (control.y + end.y) / 2 };
  const mid = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
  flattenQuadratic(start, a, mid, visit, depth + 1);
  flattenQuadratic(mid, b, end, visit, depth + 1);
}

let measurementContext: CanvasRenderingContext2D | null | undefined;
function defaultMeasureText(text: string, fontSize: number): number {
  if (measurementContext === undefined && typeof document !== 'undefined') {
    measurementContext = document.createElement('canvas').getContext('2d');
  }
  if (measurementContext) {
    measurementContext.font = `${fontSize}px ${FONT_FAMILY}`;
    return measurementContext.measureText(text).width;
  }
  // Used outside browser environments. Tests may inject exact font metrics.
  return [...text].reduce((width, char) => width + (/[^\u0000-\u00ff]/u.test(char) ? fontSize : fontSize * 0.6), 0);
}

export function textLineBounds(annotation: TextAnnotation, measure: MeasureText = defaultMeasureText) {
  return annotation.text.split('\n').map((line, index) => ({
    x: annotation.x,
    y: annotation.y + index * annotation.fontSize * TEXT_LINE_HEIGHT,
    width: measure(line, annotation.fontSize),
    height: annotation.fontSize * TEXT_LINE_HEIGHT,
  }));
}

function hitAlongSegment(annotation: Annotation, from: Point, to: Point, radius: number, measure?: MeasureText): boolean {
  if (!isFinitePoint(from) || !isFinitePoint(to) || !Number.isFinite(radius) || radius < 0) return false;
  if (annotation.kind === 'text') {
    if (!isFinitePoint(annotation) || !Number.isFinite(annotation.fontSize) || annotation.fontSize <= 0) return false;
    return textLineBounds(annotation, measure).some(({ x, y, width, height }) => {
      if (width <= 0) return false;
      const inside = (point: Point) => point.x >= x && point.x <= x + width && point.y >= y && point.y <= y + height;
      if (inside(from) || inside(to)) return true;
      const corners = [{ x, y }, { x: x + width, y }, { x: x + width, y: y + height }, { x, y: y + height }];
      return corners.some((corner, index) => segmentDistance(from, to, corner, corners[(index + 1) % 4]) <= radius);
    });
  }
  if (!Number.isFinite(annotation.width) || annotation.width <= 0) return false;
  const points = finiteStrokePoints(annotation.points);
  if (points.length === 0) return false;
  const reach = radius + annotation.width / 2;
  if (points.length === 1) return distanceToSegment(points[0], from, to) <= reach;
  const left = Math.min(from.x, to.x) - reach - 0.25;
  const right = Math.max(from.x, to.x) + reach + 0.25;
  const top = Math.min(from.y, to.y) - reach - 0.25;
  const bottom = Math.max(from.y, to.y) + reach + 0.25;
  // A quadratic lies inside its control-point hull. Reject distant curves before
  // recursive flattening / segment distances; fast erasing must not scan their geometry.
  const overlaps = (a: Point, b: Point, c: Point = b) =>
    Math.max(a.x, b.x, c.x) >= left && Math.min(a.x, b.x, c.x) <= right &&
    Math.max(a.y, b.y, c.y) >= top && Math.min(a.y, b.y, c.y) <= bottom;
  let previous = points[0];
  let hit = false;
  visitStrokePath(points, {
    move: (point) => { previous = point; },
    line: (point) => {
      hit ||= overlaps(previous, point) && segmentDistance(from, to, previous, point) <= reach;
      previous = point;
    },
    curve: (control, end) => {
      if (!hit && overlaps(previous, control, end)) flattenQuadratic(previous, control, end, (a, b) => {
        // Flattening is within 0.25 logical px; conservatively avoid a boundary miss.
        hit ||= segmentDistance(from, to, a, b) <= reach + 0.25;
      });
      previous = end;
    },
  });
  return hit;
}

export function hitTestAnnotation(annotation: Annotation, point: Point, eraserRadius: number, measure?: MeasureText): boolean {
  return hitAlongSegment(annotation, point, point, eraserRadius, measure);
}

export function hitTestAnnotations(annotations: readonly Annotation[], point: Point, eraserRadius: number): string[] {
  return hitTestAnnotationsAlongSegment(annotations, point, point, eraserRadius);
}

/** Sweep the eraser between events so fast movement cannot skip thin strokes. */
export function hitTestAnnotationsAlongSegment(annotations: readonly Annotation[], from: Point, to: Point, eraserRadius: number): string[] {
  return annotations.filter((annotation) => hitAlongSegment(annotation, from, to, eraserRadius)).map(({ id }) => id);
}
