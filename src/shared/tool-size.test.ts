import { expect, it } from 'vitest';
import { DEFAULT_SETTINGS, defaultBrush, type Tool } from './types';
import { sizePatch, stepSize, toolSize } from './tool-size';

it('adjusts text and eraser sizes without touching pen settings', () => {
  const brush = { ...defaultBrush(DEFAULT_SETTINGS), tool: 'text' as const, width: 7 };
  expect(stepSize(brush, 1)).toEqual({ textSize: 42 });
  expect(stepSize({ ...brush, tool: 'eraser' }, -1)).toEqual({ eraserSize: 44 });
  expect(toolSize({ ...brush, tool: 'highlighter' })).toBe(24);
  expect(brush.width).toBe(7);
});
it('clamps each tool at its native bounds and rejects non-finite values', () => {
  for (const [tool, field, min, max] of [
    ['pen', 'width', 1, 32], ['arrow', 'width', 1, 32], ['text', 'textSize', 8, 144],
    ['eraser', 'eraserSize', 16, 128], ['highlighter', 'highlighterWidth', 8, 64],
  ] as const) {
    expect(sizePatch(tool, -100)).toEqual({ [field]: min });
    expect(sizePatch(tool, 1000)).toEqual({ [field]: max });
  }
  expect(sizePatch('text' as Tool, NaN)).toEqual({});
});
