import { describe, expect, it } from 'vitest';
import { DEFAULT_POSITION, readPosition, toolbarBounds } from './toolbar-position';
describe('toolbar placement', () => {
  it('recovers invalid storage and clamps old off-screen positions', () => {
    for (const value of [null, 'broken', '{}', '{"x":null,"y":1,"collapsed":false}']) expect(readPosition(value)).toEqual(DEFAULT_POSITION);
    expect(readPosition('{"x":-3,"y":5,"collapsed":true}')).toEqual({ x: 0, y: 1, collapsed: true });
  });
  it('keeps each edge visible on resize including a toolbar larger than the viewport', () => {
    expect(toolbarBounds(1440, 600)).toEqual({ min: 10, travel: 820 });
    expect(toolbarBounds(360, 340)).toEqual({ min: 10, travel: 0 });
    expect(toolbarBounds(300, 340)).toEqual({ min: 0, travel: 0 });
  });
});
