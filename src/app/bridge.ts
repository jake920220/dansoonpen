import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ToolbarPreferences, ToolbarState, AppSettings, AppState, CursorFrame, BrushSettings, Mode, SceneEdit, SceneSnapshot, SceneUpdate } from '../shared/types';

export const native = isTauri();
const preview = !native && import.meta.env.DEV ? import('../dev/preview').then((m) => m.preview) : null;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (native) return invoke<T>(command, args);
  if (preview) return (await preview).invoke<T>(command, args);
  throw new Error('DansoonPen 데스크톱 앱에서 열어 주세요.');
}

async function subscribe<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  if (native) return listen<T>(event, ({ payload }) => handler(payload));
  if (preview) return (await preview).listen(event, handler);
  return () => {};
}

export const bridge = {
  getToolbar: () => call<ToolbarState>('get_toolbar'),
  onToolbar: (handler: (state: ToolbarState) => void) => subscribe('brush-toolbar', handler),
  beginToolbarDrag: (drag: { token: string; offsetX: number; offsetY: number; width: number; height: number }) => call<void>('begin_toolbar_drag', { drag }),
  moveToolbar: (token: string, point: { x: number; y: number }, finish = false) => call<void>('move_toolbar', { token, point, finish }),
  updateToolbar: (position: ToolbarPreferences) => call<void>('update_toolbar', { position }),
  toggleCursor: () => call<void>('toggle_cursor'),
  getCursor: () => call<CursorFrame | null>('get_cursor'),
  onCursor: (handler: (frame: CursorFrame) => void) => subscribe('brush-cursor', handler),
  getState: () => call<AppState>('get_state'),
  setMode: (mode: Mode) => call<AppState>('set_mode', { mode }),
  updateSettings: (settings: Partial<AppSettings>) => call<AppState>('update_settings', { settings }),
  updateBrush: (brush: Partial<BrushSettings>, brushGeneration: number) => call<AppState>('update_brush', { brush, brushGeneration }),
  updatePreset: (index: number, name?: string, brush?: BrushSettings) => call<AppState>('update_preset', { index, name, brush }),
  captureShortcut: (active: boolean) => call<void>('capture_shortcut', { active }),
  getScene: (displayId: string) => call<SceneSnapshot>('get_scene', { displayId }),
  applyEdit: (edit: SceneEdit) => call<SceneSnapshot>('apply_edit', { edit }),
  toggleAnnotations: () => call<void>('toggle_annotations'),
  clearCurrent: () => call<void>('clear_current'),
  clearAll: () => call<void>('clear_all'),
  undo: () => call<void>('undo'),
  undoClear: (token: number) => call<void>('undo_clear', { token }),
  redo: () => call<void>('redo'),
  showControl: () => call<void>('show_control'),
  quit: () => call<void>('quit_app'),
  onState: (handler: (state: AppState) => void) => subscribe('brush-state', handler),
  onOpenSettings: (handler: () => void) => subscribe('brush-open-settings', handler),
  onScene: (handler: (update: SceneUpdate) => void) => subscribe('brush-scene', handler),
};

export function message(error: unknown): string { return error instanceof Error ? error.message : String(error); }
export function shortcutLabel(shortcut: string): string {
  const mac = /Mac/.test(navigator.platform);
  return shortcut.split('+').map((key) => ({ Alt: mac ? '⌥' : 'Alt', Shift: mac ? '⇧' : 'Shift', Control: mac ? '⌃' : 'Ctrl', Super: mac ? '⌘' : 'Win', Command: '⌘', Comma: ',', Period: '.', Slash: '/', Backslash: '\\', BracketLeft: '[', BracketRight: ']', Backquote: '`', Semicolon: ';', Quote: "'", Minus: '-', Equal: '=', Space: 'Space', ArrowUp: '↑', ArrowDown: '↓', ArrowLeft: '←', ArrowRight: '→' }[key] ?? key)).join(mac ? ' ' : ' + ');
}
