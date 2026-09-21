import { describe, expect, it } from 'vitest';
import { get } from 'svelte/store';
import { readFileSync, readdirSync } from 'node:fs';
import { parse } from 'svelte/compiler';
import catalog from '../locales/en.json';
import { language, setLanguage, t, translate } from './i18n';

describe('language selection', () => {
  it('switches existing subscribers and preserves placeholder values verbatim', () => {
    const notices: string[] = [];
    const stop = t.subscribe(fn => notices.push(fn('전체 지우기')));
    setLanguage('en'); setLanguage('ko'); stop();
    expect(notices).toEqual(['전체 지우기', 'Clear all', '전체 지우기']);
    expect(get(language)).toBe('ko');
    expect(translate('en', '텍스트 색상 {0}', ['#ffcf56'])).toBe('Text color #ffcf56');
    expect(translate('en', '프리셋 {0} {1}', [1, '내 강의 {0} <b>'])).toBe('Preset 1 내 강의 {0} <b>');
    expect(translate('en', 'unknown technical detail')).toBe('unknown technical detail');
  });
  it('localizes native error templates without changing technical details', () => {
    expect(translate('en', '설정 저장에 실패했습니다: /tmp/강의.json: permission denied'))
      .toBe('Could not save settings: /tmp/강의.json: permission denied');
    expect(translate('ko', '설정 파일이 너무 큽니다.')).toBe('설정 파일이 너무 큽니다.');
  });
  it('keeps placeholder sets and nonempty values in every translation', () => {
    for (const [key, value] of Object.entries(catalog)) {
      expect(value.trim(), key).not.toBe('');
      const tokens = (s: string) => [...s.matchAll(/\{[^}]*\}/g)].map(m => m[0]).sort();
      expect(tokens(value), key).toEqual(tokens(key));
    }
  });
  it('covers every literal translation call in the Svelte UI', () => {
    const keys = new Set<string>();
    const walk = (node: unknown): void => {
      if (!node || typeof node !== 'object') return;
      const n = node as Record<string, any>;
      if (n.type === 'CallExpression' && n.callee?.name === '$t' && n.arguments[0]?.type === 'Literal') keys.add(n.arguments[0].value);
      for (const [key, value] of Object.entries(n)) if (!['loc', 'metadata', 'parent'].includes(key)) {
        if (Array.isArray(value)) value.forEach(walk); else walk(value);
      }
    };
    for (const file of readdirSync('src/ui').filter(f => f.endsWith('.svelte'))) walk(parse(readFileSync(`src/ui/${file}`, 'utf8'), { modern: true }));
    for (const key of keys) expect(Object.hasOwn(catalog, key), key).toBe(true);
    expect(keys.size).toBeGreaterThan(150);
  });
});
