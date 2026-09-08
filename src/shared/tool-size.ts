import type { BrushSettings, Tool } from './types';

export const SIZE_SPECS = {
  pen: { label: '펜 굵기', min: 1, max: 32, step: 1, quick: [4, 12, 24] },
  arrow: { label: '화살표 굵기', min: 1, max: 32, step: 1, quick: [4, 12, 24] },
  highlighter: { label: '형광펜 굵기', min: 8, max: 64, step: 2, quick: [12, 24, 48] },
  eraser: { label: '지우개 크기', min: 16, max: 128, step: 4, quick: [24, 48, 96] },
  text: { label: '글자 크기', min: 8, max: 144, step: 2, quick: [32, 40, 64] },
} satisfies Record<Tool, { label: string; min: number; max: number; step: number; quick: number[] }>;

export function toolSize(brush: BrushSettings): number {
  return brush.tool === 'text' ? brush.textSize : brush.tool === 'eraser' ? brush.eraserSize
    : brush.tool === 'highlighter' ? brush.highlighterWidth : brush.width;
}
export function sizePatch(tool: Tool, value: number): Partial<BrushSettings> {
  if (!Number.isFinite(value)) return {};
  const spec = SIZE_SPECS[tool];
  const size = Math.max(spec.min, Math.min(spec.max, value));
  return tool === 'text' ? { textSize: size } : tool === 'eraser' ? { eraserSize: size }
    : tool === 'highlighter' ? { highlighterWidth: size } : { width: size };
}
export function stepSize(brush: BrushSettings, direction: -1 | 1): Partial<BrushSettings> {
  return sizePatch(brush.tool, toolSize(brush) + direction * SIZE_SPECS[brush.tool].step);
}
