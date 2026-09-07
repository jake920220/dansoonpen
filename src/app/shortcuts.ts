export const isMac = typeof navigator !== 'undefined' && /Mac/.test(navigator.platform);
export const settingsShortcut = isMac ? 'Command+Comma' : 'Control+Comma';

type KeyEvent = Pick<KeyboardEvent, 'code' | 'key' | 'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey' | 'repeat' | 'isComposing'>;
export function recordedShortcut(event: KeyEvent): string | null {
  // Physical codes also work with Korean input and Option-produced characters.
  if (event.repeat || event.isComposing) return null;
  const code = event.code;
  const key = /^Key[A-Z]$/.test(code) ? code.slice(3)
    : /^Digit[0-9]$/.test(code) ? code.slice(5)
    : /^(F([1-9]|1[0-9]|2[0-4])|Space|Comma|Period|Slash|Backslash|Semicolon|Quote|BracketLeft|BracketRight|Backquote|Minus|Equal|ArrowUp|ArrowDown|ArrowLeft|ArrowRight|Home|End|PageUp|PageDown|Delete|Backspace)$/.test(code) ? code : null;
  if (!key || !(event.metaKey || event.ctrlKey || event.altKey)) return null;
  return [event.ctrlKey && 'Control', event.altKey && 'Alt', event.shiftKey && 'Shift', event.metaKey && 'Super', key].filter(Boolean).join('+');
}
export function isSettingsShortcut(event: KeyEvent): boolean {
  return (isMac ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey)
    && !event.altKey && !event.shiftKey && event.code === 'Comma';
}
export function matchesShortcut(event: KeyEvent, shortcut: string): boolean {
  return recordedShortcut(event) === shortcut.replace(/Command/g, 'Super');
}
