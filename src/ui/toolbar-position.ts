export interface ToolbarPosition { x: number; y: number; collapsed: boolean; detailed: boolean }
export const DEFAULT_POSITION: ToolbarPosition = { x: 0.5, y: 0, collapsed: false, detailed: false };
export function readPosition(value: string | null): ToolbarPosition {
  try {
    const p = JSON.parse(value ?? 'null');
    if (p && Number.isFinite(p.x) && Number.isFinite(p.y) && typeof p.collapsed === 'boolean') {
      return { x: Math.max(0, Math.min(1, p.x)), y: Math.max(0, Math.min(1, p.y)), collapsed: p.collapsed, detailed: p.detailed === true };
    }
  } catch { /* A damaged UI preference should never block drawing. */ }
  return { ...DEFAULT_POSITION };
}
export function toolbarBounds(viewport: number, size: number) {
  const min = Math.min(10, Math.max(0, (viewport - size) / 2));
  return { min, travel: Math.max(0, viewport - size - min * 2) };
}
