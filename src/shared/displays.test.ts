import { describe, expect, it } from 'vitest';
import { canDrawOn } from './displays';
import { DEFAULT_SETTINGS, defaultBrush, type AppState } from './types';

function session(): AppState {
  return {
    feedback: null, clearUndoToken: null, brushGeneration: 1, cursorEnabled: false,
    annotationsVisible: true, mode: 'draw', revision: 1, error: null,
    settings: DEFAULT_SETTINGS, brush: defaultBrush(DEFAULT_SETTINGS),
    displays: [
      { id: 'external', name: 'External', x: 0, y: 0, width: 2560, height: 1440, scaleFactor: 1, isPrimary: true, connected: true },
      { id: 'retina', name: 'Retina', x: 5120, y: 968, width: 2940, height: 1912, scaleFactor: 2, isPrimary: false, connected: true },
    ],
  };
}

describe('drawing on connected displays', () => {
  it('accepts input on both displays regardless of primary display or scale', () => {
    const state = session();
    expect(canDrawOn(state, 'external')).toBe(true);
    expect(canDrawOn(state, 'retina')).toBe(true);
    expect(canDrawOn(state, 'unknown')).toBe(false);
  });

  it('re-enables the external display after the observed wake disconnect sequence', () => {
    const state = session();
    state.displays[0].connected = false;
    state.displays[1].isPrimary = true;
    expect(canDrawOn(state, 'external')).toBe(false);
    expect(canDrawOn(state, 'retina')).toBe(true);
    state.displays[0].connected = true;
    state.displays[1].isPrimary = false;
    expect(canDrawOn(state, 'external')).toBe(true);
    expect(canDrawOn(state, 'retina')).toBe(true);
  });

  it('releases every display in interact mode and waits for initial state', () => {
    const state = session();
    state.mode = 'interact';
    for (const id of ['external', 'retina']) expect(canDrawOn(state, id)).toBe(false);
    expect(canDrawOn(null, 'external')).toBe(false);
  });
});
