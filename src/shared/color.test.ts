import { expect, it } from 'vitest';
import { hexToHsv, hsvToHex, wheelPosition } from './color';
it('preserves the default palette and arbitrary HEX colors through the color wheel', () => {
  for (const color of ['#ffcf56', '#ff6b6b', '#57d9c6', '#78a9ff', '#c4a0ff', '#ffffff', '#000000', '#19384c']) {
    expect(hsvToHex(hexToHsv(color))).toBe(color);
  }
});
it('maps the wheel orientation and clamps dragging beyond its edge', () => {
  expect(wheelPosition(0, -1)).toEqual({ h: 0, s: 1 });
  expect(wheelPosition(1, 0)).toEqual({ h: 90, s: 1 });
  expect(wheelPosition(0, 1)).toEqual({ h: 180, s: 1 });
  expect(wheelPosition(0, 0).s).toBe(0);
  expect(wheelPosition(10, 10).s).toBe(1);
});
