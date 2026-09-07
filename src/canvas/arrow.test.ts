import { describe, expect, it } from 'vitest';
import type { ArrowAnnotation } from '../shared/types';
import { arrowEndpoint, arrowSegments } from './arrow';
import { hitTestAnnotation, hitTestAnnotationsAlongSegment } from './geometry';
const arrow: ArrowAnnotation = { kind: 'arrow', id: 'a', start: { x: 0, y: 0 }, end: { x: 100, y: 0 }, color: '#ffcf56', width: 4 };
describe('arrow geometry', () => {
  it('erases the shaft and head without erasing empty space inside their bounding rectangle', () => {
    expect(hitTestAnnotation(arrow, { x: 50, y: 0 }, 0)).toBe(true);
    expect(hitTestAnnotation(arrow, { x: 90, y: 5 }, 0)).toBe(true);
    expect(hitTestAnnotation(arrow, { x: 50, y: 6 }, 0)).toBe(false);
    expect(hitTestAnnotationsAlongSegment([arrow], { x: 90, y: -20 }, { x: 90, y: 20 }, 0)).toEqual(['a']);
  });
  it('snaps all directions to 45 degree increments and preserves distance', () => {
    const start = { x: 40, y: -10 }, end = { x: -20, y: 10 };
    const snapped = arrowEndpoint(start, end, true);
    expect(snapped.y).toBeCloseTo(-10);
    expect(Math.hypot(snapped.x - start.x, snapped.y - start.y)).toBeCloseTo(Math.hypot(-60, 20));
    expect(arrowEndpoint(start, end, false)).toEqual(end);
    const diagonal = arrowEndpoint({ x: 0, y: 0 }, { x: 45, y: 52 }, true);
    expect(diagonal.x).toBeCloseTo(diagonal.y);
  });
  it('bounds heads on short arrows and rejects degenerate or invalid coordinates', () => {
    expect(arrowSegments({ ...arrow, end: arrow.start })).toEqual([]);
    expect(arrowSegments({ ...arrow, end: { x: NaN, y: 0 } })).toEqual([]);
    const short = { ...arrow, end: { x: 4, y: 0 }, width: 32 };
    for (const [, end] of arrowSegments(short)) expect(end.x).toBeGreaterThanOrEqual(0);
  });
});
