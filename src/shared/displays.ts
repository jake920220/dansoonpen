import type { AppState } from './types';

export function canDrawOn(state: AppState | null, displayId: string): boolean {
  return state?.mode === 'draw' && state.displays.some((display) => display.id === displayId && display.connected);
}
