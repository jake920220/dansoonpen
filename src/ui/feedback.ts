import type { Feedback } from '../shared/types';

export function noticeLifetime(feedback: Feedback | null, now: number): number {
  if (!feedback || !Number.isFinite(feedback.createdAtMs) || !Number.isFinite(now)) return 0;
  return Math.max(0, Math.min(3500, feedback.createdAtMs + 3500 - now));
}
