import type { ArrowAnnotation, Point } from '../shared/types';

/** The same segments drive painting and erasing, including the arrowhead. */
export function arrowSegments(arrow: ArrowAnnotation): [Point, Point][] {
  const { start, end, width } = arrow;
  if (![start.x, start.y, end.x, end.y, width].every(Number.isFinite) || width <= 0) return [];
  const length = Math.hypot(end.x - start.x, end.y - start.y);
  if (length < 2) return [];
  const angle = Math.atan2(end.y - start.y, end.x - start.x);
  const head = Math.min(length * 0.5, Math.max(12, width * 3));
  return [[start, end], ...[-Math.PI / 6, Math.PI / 6].map((offset): [Point, Point] => [end, {
    x: end.x - Math.cos(angle + offset) * head,
    y: end.y - Math.sin(angle + offset) * head,
  }])];
}
export function arrowEndpoint(start: Point, end: Point, snap: boolean): Point {
  if (!snap) return { ...end };
  const length = Math.hypot(end.x - start.x, end.y - start.y);
  const angle = Math.round(Math.atan2(end.y - start.y, end.x - start.x) / (Math.PI / 4)) * (Math.PI / 4);
  return { x: start.x + Math.cos(angle) * length, y: start.y + Math.sin(angle) * length };
}
