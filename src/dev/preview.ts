// Development-only browser adapter. Never used to claim native overlay support.
import { DEFAULT_SETTINGS, defaultBrush, type Annotation, type AppState, type SceneEdit, type SceneSnapshot, type SceneUpdate } from '../shared/types';

type Listener = (payload: unknown) => void;
const listeners = new Map<string, Set<Listener>>();
const displayId = 'preview-display';
let state: AppState = {
  feedback: null, clearUndoToken: null,
  brushGeneration: 0,
  cursorEnabled: false, annotationsVisible: true,
  mode: new URLSearchParams(location.search).get('view') === 'overlay' ? 'draw' : 'interact',
  activeDisplayId: displayId, revision: 1, error: null,
  settings: structuredClone(DEFAULT_SETTINGS), brush: defaultBrush(DEFAULT_SETTINGS),
  displays: [{ id: displayId, name: '미리보기 화면', x: 0, y: 0, width: window.innerWidth, height: window.innerHeight, scaleFactor: devicePixelRatio, isPrimary: true, connected: true }],
};
let scene: SceneSnapshot = { displayId, revision: 0, clearGeneration: 0, annotations: [] };
let history: Annotation[][] = [];
let historyRevision = 0;
function feedback(message: string) { state.feedback = { id: state.revision + 1, message, createdAtMs: Date.now() }; }
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
      case 'set_mode': {
        if (args.mode !== state.mode) {
          if (args.mode === 'draw') { state.brushGeneration++; state.brush = defaultBrush(state.settings); }
          feedback(args.mode === 'draw' ? '그리기 시작 · 기본 펜' : '앱 조작 가능 · 필기는 유지됩니다');
        }
        state.mode = args.mode as AppState['mode']; if (state.mode === 'draw') state.annotationsVisible = true; result = updateState(); break;
      }
      case 'select_display': state.activeDisplayId = String(args.displayId); result = updateState(); break;
      case 'update_settings': {
        const patch = structuredClone(args.settings as Partial<AppState['settings']>);
        state.settings = { ...state.settings, ...patch };
        for (const key of ['color', 'width', 'textSize', 'eraserSize'] as const) {
          if (key in patch) Object.assign(state.brush, { [key]: patch[key] });
        }
        result = updateState(); break;
      }
      case 'update_brush': if (args.brushGeneration !== state.brushGeneration) { result = structuredClone(state); break; } state.brush = { ...state.brush, ...structuredClone(args.brush as Partial<AppState['brush']>) }; result = updateState(); break;
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
        history.push(scene.annotations); history = history.slice(-128); future = []; historyRevision++;
        const replacements = new Map(edit.added.map((a) => [a.id, a]));
        const ids = new Set(scene.annotations.map((a) => a.id));
        result = updateScene([
          ...scene.annotations.filter((a) => !edit.removedIds.includes(a.id) || replacements.has(a.id)).map((a) => replacements.get(a.id) ?? a),
          ...edit.added.filter((a) => !ids.has(a.id)),
        ], scene.annotations.filter((a) => edit.removedIds.includes(a.id) && !replacements.has(a.id))); break;
      }
      case 'toggle_annotations': state.annotationsVisible = !state.annotationsVisible; if (!state.annotationsVisible) state.mode = 'interact'; result = updateState(); break;
      case 'clear_current':
      case 'clear_all': {
        const hadInk = scene.annotations.length > 0;
        if (hadInk) { history.push(scene.annotations); future = []; historyRevision++; }
        scene.clearGeneration++;
        updateScene([], scene.annotations);
        state.clearUndoToken = hadInk ? historyRevision : null;
        feedback(hadInk ? '필기를 지웠습니다' : '지울 필기가 없습니다'); updateState();
        break;
      }
      case 'undo_clear': {
        if (args.token !== state.clearUndoToken || args.token !== historyRevision) throw new Error('새 작업이 있어 이전 삭제를 바로 되돌릴 수 없습니다.');
        const previous = history.pop(); if (previous) { future.push(scene.annotations); historyRevision++; updateScene(previous); }
        state.clearUndoToken = null; state.annotationsVisible = true; feedback('지운 필기를 되돌렸습니다'); updateState(); break;
      }
      case 'undo': { const previous = history.pop(); if (previous) { future.push(scene.annotations); historyRevision++; updateScene(previous); } break; }
      case 'redo': { const next = future.pop(); if (next) { history.push(scene.annotations); historyRevision++; updateScene(next); } break; }
      case 'show_control': location.href = '/?view=control&tab=settings'; break;
      case 'quit_app': state.mode = 'interact'; result = updateState(); break;
      default: throw new Error(`Unknown preview command: ${command}`);
    }
    if (state.clearUndoToken !== null && state.clearUndoToken !== historyRevision) { state.clearUndoToken = null; updateState(); }
    return result as T;
  },
};
