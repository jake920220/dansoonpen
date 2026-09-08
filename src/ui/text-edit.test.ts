import { expect, it } from 'vitest';
import { constrainTextEditor, textDraftChanged, type TextDraft } from './text-edit';

const draft: TextDraft = {
  x: 100, y: 200, value: '한글\n메모', color: '#ffcf56', fontSize: 40,
  original: { kind: 'text', id: 'a', x: 100, y: 200, text: '한글\n메모', color: '#ffcf56', fontSize: 40 },
};
it('detects position and style-only edits while unchanged completion adds no history', () => {
  expect(textDraftChanged(draft, draft.value)).toBe(false);
  for (const patch of [{ x: 101 }, { y: 201 }, { color: '#ffffff' }, { fontSize: 64 }]) {
    expect(textDraftChanged({ ...draft, ...patch }, draft.value)).toBe(true);
  }
  expect(textDraftChanged(draft, '조합 완료')).toBe(true);
  expect(draft.original?.x).toBe(100);
});
it('keeps movement controls reachable at all viewport edges and after a smaller display resize', () => {
  expect(constrainTextEditor({ x: -50, y: -20 }, 200, 150, { width: 800, height: 600 })).toEqual({ x: 10, y: 38 });
  expect(constrainTextEditor({ x: 900, y: 800 }, 200, 150, { width: 800, height: 600 })).toEqual({ x: 590, y: 440 });
  expect(constrainTextEditor({ x: 900, y: 800 }, 1000, 700, { width: 320, height: 200 })).toEqual({ x: 10, y: 38 });
});
