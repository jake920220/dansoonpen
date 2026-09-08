import type { AppSettings, AppState } from '../shared/types';

/** Coalesce sliders and retain only the latest unsaved value for each field. */
export class SettingsWriter<T extends object = AppSettings> {
  private pending: Partial<T> = {};
  private optimistic: Partial<T> = {};
  private versions = new Map<keyof T, number>();
  private sequence = 0;
  private epoch = 0;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private edits: Promise<void> = Promise.resolve();
  constructor(
    private save: (patch: Partial<T>) => Promise<AppState>,
    private accept: (state: AppState) => void,
    private changed: (pending: Partial<T>) => void,
    private failed: (error: unknown) => void,
  ) {}
  update(patch: Partial<T>) {
    for (const key of Object.keys(patch) as (keyof T)[]) this.versions.set(key, ++this.sequence);
    this.pending = { ...this.pending, ...patch };
    this.optimistic = { ...this.optimistic, ...patch };
    this.changed(this.optimistic);
    clearTimeout(this.timer);
    this.timer = setTimeout(() => { void this.flush(); }, 100);
  }
  invalidate() {
    this.epoch++;
    clearTimeout(this.timer);
    this.pending = {}; this.optimistic = {}; this.versions.clear();
    this.changed({});
  }
  flush(): Promise<void> {
    clearTimeout(this.timer);
    const patch = this.pending;
    if (!Object.keys(patch).length) return this.edits;
    this.pending = {};
    const versions = new Map(this.versions);
    const epoch = this.epoch;
    this.edits = this.edits.then(async () => {
      if (epoch !== this.epoch) return;
      try { this.accept(await this.save(patch)); }
      catch (error) { this.failed(error); }
      finally {
        if (epoch !== this.epoch) return;
        const remaining = { ...this.optimistic };
        for (const key of Object.keys(patch) as (keyof T)[]) {
          if (versions.get(key) === this.versions.get(key)) delete remaining[key];
        }
        this.optimistic = remaining;
        this.changed(remaining);
      }
    });
    return this.edits;
  }
}
