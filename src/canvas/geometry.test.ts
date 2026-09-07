import { describe, expect, it } from 'vitest';
import type { StrokeAnnotation, TextAnnotation } from '../shared/types';
import { appendStrokePoint, distanceToSegment, hitTestAnnotation, hitTestAnnotationsAlongSegment } from './geometry';

const stroke: StrokeAnnotation = { kind: 'stroke', id: 'line', points: [{ x: 50, y: 0 }, { x: 50, y: 100 }], width: 4, color: '#fff' };

describe('eraser geometry', () => {
  it('accounts for stroke thickness, round ends and isolated dots', () => {
    expect(hitTestAnnotation(stroke, { x: 55, y: 50 }, 3)).toBe(true);
    expect(hitTestAnnotation(stroke, { x: 56, y: 50 }, 3)).toBe(false);
    expect(hitTestAnnotation(stroke, { x: 50, y: -5 }, 3)).toBe(true);
    const dot = { ...stroke, points: [{ x: 50, y: 50 }] };
    expect(hitTestAnnotation(dot, { x: 53, y: 54 }, 3)).toBe(true);
    expect(hitTestAnnotation(dot, { x: 54, y: 54 }, 3)).toBe(false);
  });

  it('sweeps between sparse pointer events instead of skipping a crossed stroke', () => {
    expect(hitTestAnnotation(stroke, { x: 0, y: 50 }, 1)).toBe(false);
    expect(hitTestAnnotation(stroke, { x: 100, y: 50 }, 1)).toBe(false);
    expect(hitTestAnnotationsAlongSegment([stroke], { x: 0, y: 50 }, { x: 100, y: 50 }, 1)).toEqual(['line']);
    expect(hitTestAnnotationsAlongSegment([stroke], { x: 0, y: -4 }, { x: 100, y: -4 }, 1)).toEqual([]);
  });

  it('tests the visible smoothed curve, not its control polygon', () => {
    const curved = { ...stroke, width: 1, points: [{ x: 0, y: 0 }, { x: 100, y: 100 }, { x: 200, y: 0 }] };
    // t=0.5 on the first quadratic: start (0,0), control (100,100), end (150,50).
    expect(hitTestAnnotation(curved, { x: 87.5, y: 62.5 }, 0.1)).toBe(true);
    expect(hitTestAnnotation(curved, { x: 100, y: 100 }, 1)).toBe(false);
  });

  it('uses each text line width and multiline height', () => {
    const text: TextAnnotation = { kind: 'text', id: 'text', x: 10, y: 20, fontSize: 10, text: 'long\nx', color: '#fff' };
    const measure = (line: string) => line.length * 10;
    expect(hitTestAnnotation(text, { x: 49, y: 25 }, 0, measure)).toBe(true);
    expect(hitTestAnnotation(text, { x: 49, y: 38 }, 0, measure)).toBe(false);
    expect(hitTestAnnotation(text, { x: 19, y: 45 }, 0, measure)).toBe(true);
    expect(hitTestAnnotation(text, { x: 20, y: 47 }, 1, measure)).toBe(true);
    expect(hitTestAnnotation(text, { x: 21, y: 47 }, 1, measure)).toBe(false);
  });

  it('handles zero-length segments, invalid coordinates and empty strokes', () => {
    expect(distanceToSegment({ x: 3, y: 4 }, { x: 0, y: 0 }, { x: 0, y: 0 })).toBe(5);
    expect(hitTestAnnotation({ ...stroke, points: [] }, { x: 0, y: 0 }, 5)).toBe(false);
    expect(hitTestAnnotation(stroke, { x: NaN, y: 1 }, 5)).toBe(false);
    expect(hitTestAnnotation(stroke, { x: 50, y: 50 }, -1)).toBe(false);
    expect(hitTestAnnotation({ ...stroke, width: Infinity }, { x: 50, y: 50 }, 5)).toBe(false);
  });
});

describe('pointer sampling', () => {
  it('rejects invalid/duplicate samples but preserves a short final endpoint', () => {
    const points = [{ x: 0, y: 0 }];
    expect(appendStrokePoint(points, { x: 0.2, y: 0 })).toBe(false);
    expect(appendStrokePoint(points, { x: 0.2, y: 0 }, 0.5, true)).toBe(true);
    expect(appendStrokePoint(points, { x: 0.2, y: 0 }, 0.5, true)).toBe(false);
    expect(appendStrokePoint(points, { x: Infinity, y: 0 })).toBe(false);
    expect(points).toHaveLength(2);
  });
});
