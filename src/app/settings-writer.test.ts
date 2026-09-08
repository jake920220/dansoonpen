import { afterEach, expect, it, vi } from 'vitest';
import { SettingsWriter } from './settings-writer';
import { DEFAULT_SETTINGS, defaultBrush, type AppSettings, type AppState } from '../shared/types';

afterEach(() => vi.useRealTimers());
const state = (settings = DEFAULT_SETTINGS): AppState => ({ annotationsVisible: true, settings, brush: defaultBrush(settings), mode: 'interact', activeDisplayId: null, revision: 1, error: null, displays: [] });
it('coalesces rapid edits and sends only the changed fields', async () => {
  vi.useFakeTimers();
  const save = vi.fn(async () => state());
  const writer = new SettingsWriter(save, vi.fn(), vi.fn(), vi.fn());
  writer.update({ color: '#ff6b6b' }); writer.update({ color: '#57d9c6' }); writer.update({ width: 12 });
  await vi.advanceTimersByTimeAsync(100);
  expect(save.mock.calls).toEqual([[{ color: '#57d9c6', width: 12 }]]);
});
it('keeps the latest optimistic color while a previous write completes, then releases it for external settings', async () => {
  let complete!: (state: AppState) => void;
  const save = vi.fn().mockImplementationOnce(() => new Promise<AppState>((resolve) => complete = resolve)).mockResolvedValue(state());
  let optimistic: Partial<AppSettings> = {};
  const changed = (next: Partial<AppSettings>) => optimistic = next;
  const writer = new SettingsWriter(save, vi.fn(), changed, vi.fn());
  writer.update({ color: '#ff6b6b' }); const first = writer.flush();
  await Promise.resolve();
  writer.update({ color: '#57d9c6' });
  complete(state()); await first;
  expect(optimistic).toEqual({ color: '#57d9c6' });
  await writer.flush(); expect(optimistic).toEqual({});
  expect(save.mock.calls[1]).toEqual([{ color: '#57d9c6' }]);
});
it('reports persistence failures and releases optimistic values so the saved settings can be restored', async () => {
  const failed = vi.fn(), changed = vi.fn();
  const writer = new SettingsWriter(async () => { throw new Error('disk full'); }, vi.fn(), changed, failed);
  writer.update({ color: '#ff6b6b' }); await writer.flush();
  expect(failed).toHaveBeenCalledOnce(); expect(changed).toHaveBeenLastCalledWith({});
});
