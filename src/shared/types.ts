export type Mode = 'draw' | 'interact';
export type Tool = 'pen' | 'eraser' | 'text';
export interface Point { x: number; y: number }
export interface StrokeAnnotation {
  kind: 'stroke'; id: string; points: Point[]; color: string; width: number;
}
export interface TextAnnotation {
  kind: 'text'; id: string; x: number; y: number; text: string; color: string; fontSize: number;
}
export type Annotation = StrokeAnnotation | TextAnnotation;
export interface AppSettings {
  version: number; color: string; width: number; textSize: number;
  quickColors: string[]; toggleShortcut: string; clearShortcut: string; reduceMotion: boolean;
}
export interface DisplayInfo {
  id: string; name: string; x: number; y: number; width: number; height: number;
  scaleFactor: number; isPrimary: boolean; connected: boolean;
}
export interface AppState {
  mode: Mode; activeDisplayId: string | null; displays: DisplayInfo[];
  settings: AppSettings; revision: number; error: string | null;
}
export interface SceneSnapshot { displayId: string; revision: number; clearGeneration: number; annotations: Annotation[] }
export interface SceneUpdate { scene: SceneSnapshot; fadeOut: Annotation[]; fadeDurationMs: number }
export interface SceneEdit { displayId: string; clearGeneration: number; added: Annotation[]; removedIds: string[] }
export const DEFAULT_SETTINGS: AppSettings = {
  version: 2, color: '#ffcf56', width: 12, textSize: 28,
  quickColors: ['#ffcf56', '#ff6b6b', '#57d9c6', '#78a9ff', '#c4a0ff', '#ffffff'],
  toggleShortcut: typeof navigator !== 'undefined' && /Mac/.test(navigator.platform) ? 'Alt+Z' : 'Alt+Shift+Z',
  clearShortcut: typeof navigator !== 'undefined' && /Mac/.test(navigator.platform) ? 'Alt+X' : 'Alt+Shift+X', reduceMotion: false,
};
