import { derived, writable } from 'svelte/store';
import english from '../locales/en.json';
import type { Language } from '../shared/types';

// Korean source messages are the fallback; user content never passes through this table.
const catalog: Record<string, string> = english;
export const language = writable<Language>('ko');
// Native errors include technical details in named placeholders. Translate their
// message template while leaving captured paths and OS error text intact.
const nativeErrors = Object.entries(catalog).filter(([key]) => /\{[a-z]*\}/.test(key)).map(([key, translated]) => {
  const names: string[] = [];
  const parts = key.split(/(\{[a-z]*\})/g).map(part => {
    if (/^\{[a-z]*\}$/.test(part)) { names.push(part); return '([\\s\\S]*?)'; }
    return part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  });
  return { pattern: new RegExp('^' + parts.join('') + '$'), names, translated };
});
function englishMessage(key: string): string {
  if (catalog[key] !== undefined) return catalog[key];
  for (const { pattern, names, translated } of nativeErrors) {
    const match = pattern.exec(key);
    if (match) return translated.replace(/\{[a-z]*\}/g, name => match[names.indexOf(name) + 1] ?? name);
  }
  return key;
}
export function translate(locale: Language, key: string, values: readonly unknown[] = []): string {
  const template = locale === 'en' ? englishMessage(key) : key;
  return template.replace(/\{(\d+)\}/g, (placeholder, index: string) =>
    Number(index) < values.length ? String(values[Number(index)]) : placeholder);
}
export const t = derived(language, (locale) => (key: string, values: readonly unknown[] = []) => translate(locale, key, values));
export function setLanguage(locale: Language) {
  language.set(locale);
  if (typeof document !== 'undefined') document.documentElement.lang = locale;
}
