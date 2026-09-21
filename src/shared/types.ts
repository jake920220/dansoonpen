export type Language = 'ko' | 'en';
export type Mode = 'draw' | 'interact';
export type Tool = 'pen' | 'eraser' | 'text' | 'arrow' | 'highlighter';
export interface Point { x: number; y: number }
export interface StrokeAnnotation {
  kind: 'stroke'; id: string; points: Point[]; color: string; width: number; opacity?: number;
}
export interface TextAnnotation {
  kind: 'text'; id: string; x: number; y: number; text: string; color: string; fontSize: number;
}
export interface ArrowAnnotation { kind: 'arrow'; id: string; start: Point; end: Point; color: string; width: number }
export type Annotation = StrokeAnnotation | TextAnnotation | ArrowAnnotation;
export interface BrushSettings { eraserSize: number; tool: Tool; color: string; width: number; textSize: number; highlighterWidth: number; highlighterOpacity: number }
export interface BrushPreset { name: string; brush: BrushSettings }
export interface CursorSettings { color: string; size: number; showClicks: boolean }
export interface CursorFrame { displayId: string; x: number; y: number; visible: boolean; clicks: number; sequence: number }
export interface AppSettings {
  language: Language;
  eraserSize: number;
  cursor: CursorSettings;
  presets: BrushPreset[];
  version: number; color: string; width: number; textSize: number;
  quickColors: string[]; toggleShortcut: string; clearShortcut: string; visibilityShortcut: string; reduceMotion: boolean;
}
export interface DisplayInfo {
  id: string; name: string; x: number; y: number; width: number; height: number;
  scaleFactor: number; isPrimary: boolean; connected: boolean;
}
export interface Feedback { id: number; message: string; createdAtMs: number }
export interface AppState {
  feedback: Feedback | null; clearUndoToken: number | null;
  brushGeneration: number;
  cursorEnabled: boolean; annotationsVisible: boolean; mode: Mode; activeDisplayId: string | null; displays: DisplayInfo[];
  settings: AppSettings; brush: BrushSettings; revision: number; error: string | null;
}
export interface SceneSnapshot { displayId: string; revision: number; clearGeneration: number; annotations: Annotation[] }
export interface SceneUpdate { scene: SceneSnapshot; fadeOut: Annotation[]; fadeDurationMs: number }
export interface SceneEdit { displayId: string; clearGeneration: number; added: Annotation[]; removedIds: string[] }
export const DEFAULT_SETTINGS: AppSettings = {
  language: 'ko',
  cursor: { color: '#ffcf56', size: 48, showClicks: true },
  eraserSize: 48,
  version: 6, color: '#ffcf56', width: 12, textSize: 40,
  presets: [
    { name: '기본 강조', brush: { eraserSize: 48, tool: 'pen', highlighterWidth: 24, highlighterOpacity: 0.32, color: '#ffcf56', width: 12, textSize: 40 } },
    { name: '빨간 밑줄', brush: { eraserSize: 48, tool: 'pen', highlighterWidth: 24, highlighterOpacity: 0.32, color: '#ff6b6b', width: 4, textSize: 40 } },
    { name: '민트 메모', brush: { eraserSize: 48, tool: 'text', highlighterWidth: 24, highlighterOpacity: 0.32, color: '#57d9c6', width: 6, textSize: 44 } },
  ],
  visibilityShortcut: typeof navigator !== 'undefined' && /Mac/.test(navigator.platform) ? 'Alt+V' : 'Alt+Shift+V',
  quickColors: ['#ffcf56', '#ff6b6b', '#57d9c6', '#78a9ff', '#c4a0ff', '#ffffff'],
  toggleShortcut: typeof navigator !== 'undefined' && /Mac/.test(navigator.platform) ? 'Alt+Z' : 'Alt+Shift+Z',
  clearShortcut: typeof navigator !== 'undefined' && /Mac/.test(navigator.platform) ? 'Alt+X' : 'Alt+Shift+X', reduceMotion: false,
};

export function defaultBrush(settings: AppSettings): BrushSettings {
  return { eraserSize: settings.eraserSize, tool: 'pen', highlighterWidth: 24, highlighterOpacity: 0.32, color: settings.color, width: settings.width, textSize: settings.textSize };
}

export const TOOL_LABELS: Record<Tool, string> = { pen: '펜', eraser: '지우개', text: '텍스트', arrow: '화살표', highlighter: '형광펜' };
