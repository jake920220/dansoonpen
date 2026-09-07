import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AppSettings, AppState, Mode, SceneEdit, SceneSnapshot, SceneUpdate } from '../shared/types';

export const native = isTauri();
const preview = !native && import.meta.env.DEV ? import('../dev/preview').then((m) => m.preview) : null;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (native) return invoke<T>(command, args);
  if (preview) return (await preview).invoke<T>(command, args);
  throw new Error('My Brush 데스크톱 앱에서 열어 주세요.');
}

async function subscribe<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  if (native) return listen<T>(event, ({ payload }) => handler(payload));
  if (preview) return (await preview).listen(event, handler);
  return () => {};
}

export const bridge = {
  getState: () => call<AppState>('get_state'),
  setMode: (mode: Mode) => call<AppState>('set_mode', { mode }),
  selectDisplay: (displayId: string) => call<AppState>('select_display', { displayId }),
  updateSettings: (settings: AppSettings) => call<AppState>('update_settings', { settings }),
  getScene: (displayId: string) => call<SceneSnapshot>('get_scene', { displayId }),
  applyEdit: (edit: SceneEdit) => call<SceneSnapshot>('apply_edit', { edit }),
  clearAll: () => call<void>('clear_all'),
  undo: () => call<void>('undo'),
  redo: () => call<void>('redo'),
  showControl: () => call<void>('show_control'),
  quit: () => call<void>('quit_app'),
  onState: (handler: (state: AppState) => void) => subscribe('brush-state', handler),
  onScene: (handler: (update: SceneUpdate) => void) => subscribe('brush-scene', handler),
};

export function message(error: unknown): string { return error instanceof Error ? error.message : String(error); }
export function shortcutLabel(shortcut: string): string {
  const mac = /Mac/.test(navigator.platform);
  return shortcut.split('+').map((key) => ({ Alt: mac ? '⌥' : 'Alt', Shift: mac ? '⇧' : 'Shift', Control: mac ? '⌃' : 'Ctrl', Super: mac ? '⌘' : 'Win', Command: '⌘' }[key] ?? key)).join(mac ? ' ' : ' + ');
}
