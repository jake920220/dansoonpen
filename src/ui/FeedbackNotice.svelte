<script lang="ts">
  import { t } from '../app/i18n';
  import { untrack } from 'svelte';
  import type { Feedback } from '../shared/types';
  import { noticeLifetime } from './feedback';
  let { feedback, undoToken, interactive, reduceMotion, onundo }: {
    feedback: Feedback | null; undoToken: number | null; interactive: boolean; reduceMotion: boolean; onundo: (token: number) => void;
  } = $props();
  let notice = $state<Feedback | null>(null);
  let noticeId = $derived(feedback?.id);
  $effect(() => {
    const id = noticeId;
    const next = untrack(() => feedback);
    const duration = noticeLifetime(next, Date.now());
    const visibleNotice = id !== undefined && duration > 0 ? next : null;
    notice = visibleNotice;
    if (!visibleNotice) return;
    const timer = setTimeout(() => notice = null, duration);
    return () => clearTimeout(timer);
  });
</script>
{#if notice}
  <div class="feedback-notice" class:motion={!reduceMotion} role="status" aria-live="polite">
    <span>{$t(notice.message)}</span>
    {#if undoToken !== null}
      {#if interactive}<button onclick={() => { if (undoToken !== null) onundo(undoToken); }}>{$t("삭제 되돌리기")}</button>
      {:else}<small>{$t("메뉴에서 ‘방금 지운 필기 되돌리기’")}</small>{/if}
    {/if}
  </div>
{/if}
<style>
  .feedback-notice { position: fixed; right: 20px; top: 24px; z-index: 25; display: flex; flex-direction: column; align-items: flex-start; gap: 8px; max-width: calc(100vw - 40px); padding: 13px 17px; border: 1px solid #566375; border-radius: 12px; background: #202833f2; color: #edf3fc; font-size: 13px; box-shadow: 0 6px 18px #0002; pointer-events: none; }
  .motion { animation: appear 120ms ease-out; }
  small { font-size: 11px; color: #b1bfd1; }
  button { pointer-events: auto; padding: 6px 10px; border: 1px solid #ffcf56; border-radius: 6px; background: #ffcf5610; color: #ffcf56; font-size: 12px; }
  @keyframes appear { from { opacity: 0; transform: translateY(-3px); } to { opacity: 1; transform: translateY(0); } }
</style>
