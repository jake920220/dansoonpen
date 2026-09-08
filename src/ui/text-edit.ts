import type { Point, TextAnnotation } from '../shared/types';

export interface TextDraft extends Point {
  value: string; color: string; fontSize: number; original?: TextAnnotation;
}
export function textDraftChanged(draft: TextDraft, value: string): boolean {
  const old = draft.original;
  return !old || old.text !== value || old.x !== draft.x || old.y !== draft.y
    || old.color !== draft.color || old.fontSize !== draft.fontSize;
}
export function constrainTextEditor(point: Point, width: number, height: number, viewport: { width: number; height: number }): Point {
  // Keep the move handle (above the text baseline) and completion controls reachable.
  return {
    x: Math.max(10, Math.min(point.x, Math.max(10, viewport.width - width - 10))),
    y: Math.max(38, Math.min(point.y, Math.max(38, viewport.height - height - 10))),
  };
}
