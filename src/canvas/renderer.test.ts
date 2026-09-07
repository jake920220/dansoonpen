import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { StrokeAnnotation } from '../shared/types';
import { CanvasRenderer } from './renderer';

const stroke = (id: string, color = '#fff'): StrokeAnnotation => ({
  kind: 'stroke', id, color, width: 4, points: [{ x: 10, y: 20 }, { x: 30, y: 40 }],
});

describe('CanvasRenderer scheduling and fade ownership', () => {
  let callbacks: Map<number, FrameRequestCallback>;
  let now: number;
  let canvas: HTMLCanvasElement;
  let renderer: CanvasRenderer;
  let ctx: Record<string, any>;
  let draws: { color: string; alpha: number }[];

  function frame(time: number) {
    now = time;
    const pending = [...callbacks.values()];
    callbacks.clear();
    for (const callback of pending) callback(time);
  }

  beforeEach(() => {
    callbacks = new Map();
    now = 0;
    draws = [];
    let next = 0;
    vi.stubGlobal('requestAnimationFrame', vi.fn((callback: FrameRequestCallback) => {
      callbacks.set(++next, callback);
      return next;
    }));
    vi.stubGlobal('cancelAnimationFrame', vi.fn((id: number) => callbacks.delete(id)));
    vi.spyOn(performance, 'now').mockImplementation(() => now);
    ctx = {
      setTransform: vi.fn(), clearRect: vi.fn(), beginPath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(),
      quadraticCurveTo: vi.fn(), arc: vi.fn(), fill: vi.fn(), fillText: vi.fn(),
      stroke: vi.fn(() => draws.push({ color: ctx.strokeStyle, alpha: ctx.globalAlpha })),
    };
    canvas = { width: 0, height: 0, style: {}, getContext: () => ctx } as unknown as HTMLCanvasElement;
    renderer = new CanvasRenderer(canvas);
  });

  afterEach(() => {
    renderer.dispose();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it('coalesces mutations and never schedules a frame for idle persistent ink', () => {
    renderer.resize(100, 50, 2);
    renderer.setScene([stroke('one')]);
    renderer.setPreview(stroke('preview'));
    expect(callbacks.size).toBe(1);
    frame(10);
    expect(ctx.stroke).toHaveBeenCalledTimes(2);
    expect(callbacks.size).toBe(0);
    frame(30 * 60 * 1_000);
    expect(ctx.stroke).toHaveBeenCalledTimes(2);
    expect(callbacks.size).toBe(0);
  });

  it('uses exact DPR scaling and clears the complete backing bitmap', () => {
    renderer.resize(101, 51, 1.25);
    frame(1);
    expect(canvas.width).toBe(127);
    expect(canvas.height).toBe(64);
    expect(canvas.style.width).toBe('101px');
    expect(ctx.clearRect).toHaveBeenCalledWith(0, 0, 127, 64);
    expect(ctx.setTransform).toHaveBeenLastCalledWith(1.25, 0, 0, 1.25, 0, 0);
    expect(() => renderer.resize(100, 100, 0)).toThrow(RangeError);
  });

  it('fades only a detached snapshot while newly drawn ink remains fully opaque', () => {
    const old = stroke('old', '#f00');
    renderer.setScene([]);
    renderer.fadeOut([old], 350);
    old.points[0].x = 999;
    renderer.setScene([stroke('new', '#00f')]);
    frame(175);
    expect(draws).toEqual([{ color: '#f00', alpha: 0.5 }, { color: '#00f', alpha: 1 }]);
    expect(ctx.moveTo).toHaveBeenCalledWith(10, 20);
    expect(ctx.moveTo).not.toHaveBeenCalledWith(999, 20);
    expect(callbacks.size).toBe(1);
    draws.length = 0;
    frame(351);
    expect(draws).toEqual([{ color: '#00f', alpha: 1 }]);
    expect(callbacks.size).toBe(0);
  });

  it('finishes simultaneous fades independently, including after background throttling', () => {
    renderer.fadeOut([stroke('a', '#a00')], 350);
    frame(100);
    renderer.fadeOut([stroke('b', '#b00')], 350);
    draws.length = 0;
    frame(360);
    expect(draws).toHaveLength(1);
    expect(draws[0].color).toBe('#b00');
    expect(draws[0].alpha).toBeCloseTo(1 - 260 / 350);
    draws.length = 0;
    frame(5_000);
    expect(draws).toEqual([]);
    expect(callbacks.size).toBe(0);
  });

  it('undo restoration cancels a matching fade without duplicating restored ink', () => {
    const old = stroke('same');
    renderer.fadeOut([old], 350);
    frame(100);
    renderer.setScene([old]);
    draws.length = 0;
    frame(150);
    expect(draws).toEqual([{ color: '#fff', alpha: 1 }]);
    expect(callbacks.size).toBe(0);
    renderer.fadeOut([old], 350); // A stale clear event cannot ghost live ids.
    draws.length = 0;
    frame(200);
    expect(draws).toEqual([{ color: '#fff', alpha: 1 }]);
    expect(callbacks.size).toBe(0);
  });

  it('reduces motion immediately, and caps retained fade objects and duration', () => {
    renderer.fadeOut([stroke('instant')], 0);
    frame(0);
    expect(draws).toEqual([]);
    expect(callbacks.size).toBe(0);
    renderer.fadeOut(Array.from({ length: 2_100 }, (_, i) => stroke(`${i}`)), 100_000);
    frame(1);
    expect(draws).toHaveLength(2_000);
    draws.length = 0;
    frame(2_001);
    expect(draws).toEqual([]);
    expect(callbacks.size).toBe(0);
  });

  it('renders a tap as a round dot and text on separate lines', () => {
    renderer.setScene([
      { ...stroke('dot'), points: [{ x: 10, y: 20 }] },
      { kind: 'text', id: 'text', x: 5, y: 10, text: '한글\nEnglish', color: '#fff', fontSize: 20 },
    ]);
    frame(0);
    expect(ctx.arc).toHaveBeenCalledWith(10, 20, 2, 0, Math.PI * 2);
    expect(ctx.fill).toHaveBeenCalledOnce();
    expect(ctx.fillText.mock.calls).toEqual([['한글', 5, 10], ['English', 5, 36]]);
    expect(ctx.textBaseline).toBe('top');
  });

  it('dispose cancels outstanding animation and releases retained drawing state', () => {
    renderer.resize(1920, 1080, 2);
    renderer.fadeOut([stroke('old')], 350);
    renderer.dispose();
    expect(callbacks.size).toBe(0);
    expect(canvas.width).toBe(0);
    expect(canvas.height).toBe(0);
    renderer.setScene([stroke('late')]);
    renderer.setPreview(stroke('late'));
    renderer.fadeOut([stroke('late')], 350);
    renderer.resize(100, 100, 1);
    expect(callbacks.size).toBe(0);
  });
});
