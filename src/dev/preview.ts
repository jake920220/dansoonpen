// Development-only browser adapter. Never used to claim native overlay support.
import { DEFAULT_SETTINGS, defaultBrush, type Annotation, type AppState, type SceneEdit, type SceneSnapshot, type SceneUpdate } from '../shared/types';

type Listener = (payload: unknown) => void;
const listeners = new Map<string, Set<Listener>>();
const displayId = 'preview-display';
let state: AppState = {
  cursorEnabled: false, annotationsVisible: true,
  mode: new URLSearchParams(location.search).get('view') === 'overlay' ? 'draw' : 'interact',
  activeDisplayId: displayId, revision: 1, error: null,
  settings: structuredClone(DEFAULT_SETTINGS), brush: defaultBrush(DEFAULT_SETTINGS),
  displays: [{ id: displayId, name: '미리보기 화면', x: 0, y: 0, width: window.innerWidth, height: window.innerHeight, scaleFactor: devicePixelRatio, isPrimary: true, connected: true }],
};
let scene: SceneSnapshot = { displayId, revision: 0, clearGeneration: 0, annotations: [] };
let history: Annotation[][] = [];
let future: Annotation[][] = [];
const emit = (name: string, payload: unknown) => listeners.get(name)?.forEach((fn) => fn(structuredClone(payload)));
const updateState = () => { state.revision++; emit('brush-state', state); return structuredClone(state); };
function updateScene(annotations: Annotation[], fadeOut: Annotation[] = []): SceneSnapshot {
  scene = { ...scene, revision: scene.revision + 1, annotations };
  emit('brush-scene', { scene, fadeOut, fadeDurationMs: state.settings.reduceMotion ? 0 : 350 } satisfies SceneUpdate);
  return structuredClone(scene);
}

export const preview = {
  listen<T>(name: string, fn: (payload: T) => void): () => void {
    const set = listeners.get(name) ?? new Set<Listener>();
    set.add(fn as Listener); listeners.set(name, set);
    return () => { set.delete(fn as Listener); };
  },
  async invoke<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
    args = JSON.parse(JSON.stringify(args)); // Match the native JSON boundary, including Svelte proxies.
    let result: unknown;
    switch (command) {
      case 'get_cursor': result = null; break;
      case 'toggle_cursor': state.cursorEnabled = !state.cursorEnabled; result = updateState(); break;
      case 'get_state': result = structuredClone(state); break;
      case 'get_scene': result = structuredClone(scene); break;
      case 'set_mode': state.mode = args.mode as AppState['mode']; if (state.mode === 'draw') state.annotationsVisible = true; result = updateState(); break;
      case 'select_display': state.activeDisplayId = String(args.displayId); result = updateState(); break;
      case 'update_settings': {
        const patch = structuredClone(args.settings as Partial<AppState['settings']>);
        state.settings = { ...state.settings, ...patch };
        for (const key of ['color', 'width', 'textSize'] as const) {
          if (key in patch) Object.assign(state.brush, { [key]: patch[key] });
        }
        result = updateState(); break;
      }
      case 'update_brush': state.brush = { ...state.brush, ...structuredClone(args.brush as Partial<AppState['brush']>) }; result = updateState(); break;
      case 'update_preset': {
        const preset = state.settings.presets[Number(args.index)];
        if (typeof args.name === 'string') preset.name = args.name;
        if (args.brush) preset.brush = structuredClone(args.brush as AppState['brush']);
        result = updateState(); break;
      }
      case 'capture_shortcut': break;
      case 'apply_edit': {
        const edit = args.edit as SceneEdit;
        if (edit.clearGeneration < scene.clearGeneration) { result = structuredClone(scene); break; }
        history.push(scene.annotations); history = history.slice(-128); future = [];
        result = updateScene([...scene.annotations.filter((a) => !edit.removedIds.includes(a.id)), ...edit.added]); break;
      }
      case 'toggle_annotations': state.annotationsVisible = !state.annotationsVisible; if (!state.annotationsVisible) state.mode = 'interact'; result = updateState(); break;
      case 'clear_current':
      case 'clear_all':
        if (scene.annotations.length) { history.push(scene.annotations); future = []; }
        scene.clearGeneration++;
        updateScene([], scene.annotations);
        state.mode = 'interact'; updateState();
        break;
      case 'undo': { const previous = history.pop(); if (previous) { future.push(scene.annotations); updateScene(previous); } break; }
      case 'redo': { const next = future.pop(); if (next) { history.push(scene.annotations); updateScene(next); } break; }
      case 'show_control': location.href = '/?view=control&tab=settings'; break;
      case 'quit_app': state.mode = 'interact'; result = updateState(); break;
      default: throw new Error(`Unknown preview command: ${command}`);
    }
    return result as T;
  },
};
