import { describe, expect, it } from 'vitest';
import { canvasKey, recordedShortcut } from './shortcuts';

const key = (extra: Partial<KeyboardEvent> = {}) => ({ code: 'KeyD', key: 'd', metaKey: false, ctrlKey: false, altKey: false, shiftKey: false, repeat: false, isComposing: false, ...extra });
describe('shortcut recording', () => {
  it('keeps tool keys available with Korean input without intercepting composition', () => {
    expect(canvasKey(key({ code: 'KeyP', key: 'ㅔ' }))).toBe('p');
    expect(canvasKey(key({ code: 'Digit2', key: '2' }))).toBe('2');
    expect(canvasKey(key({ code: 'BracketRight' }))).toBe(']');
    expect(canvasKey(key({ code: 'KeyT', isComposing: true }))).toBe('');
  });
  it('uses the physical key in Korean and Option text layouts', () => {
    expect(recordedShortcut(key({ key: 'ㅇ', metaKey: true, shiftKey: true }))).toBe('Shift+Super+D');
    expect(recordedShortcut(key({ code: 'KeyZ', key: 'Ω', altKey: true }))).toBe('Alt+Z');
  });
  it('rejects typing, modifiers alone, repeats and composition', () => {
    expect(recordedShortcut(key())).toBeNull();
    expect(recordedShortcut(key({ shiftKey: true }))).toBeNull();
    expect(recordedShortcut(key({ code: 'AltLeft', key: 'Alt', altKey: true }))).toBeNull();
    expect(recordedShortcut(key({ altKey: true, repeat: true }))).toBeNull();
    expect(recordedShortcut(key({ altKey: true, isComposing: true }))).toBeNull();
  });
  it('records punctuation and named keys as native accelerators', () => {
    expect(recordedShortcut(key({ code: 'Comma', key: ',', ctrlKey: true }))).toBe('Control+Comma');
    expect(recordedShortcut(key({ code: 'F6', key: 'F6', ctrlKey: true, altKey: true }))).toBe('Control+Alt+F6');
  });
});
