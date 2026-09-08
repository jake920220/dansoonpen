import { expect, it } from 'vitest';
import { noticeLifetime } from './feedback';

it('does not revive expired status messages when a window reconnects', () => {
  const feedback = { id: 1, message: '그리기 시작', createdAtMs: 1000 };
  expect(noticeLifetime(feedback, 1000)).toBe(3500);
  expect(noticeLifetime(feedback, 4200)).toBe(300);
  expect(noticeLifetime(feedback, 5000)).toBe(0);
  expect(noticeLifetime(feedback, -10000)).toBe(3500);
  expect(noticeLifetime(null, 0)).toBe(0);
});
