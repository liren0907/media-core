<script lang="ts">
  import { onDestroy } from 'svelte';

  interface Props {
    code: string;
  }

  let { code }: Props = $props();

  let copied = $state(false);
  let timer: ReturnType<typeof setTimeout> | null = null;

  async function copy() {
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => (copied = false), 1500);
    } catch (e) {
      console.error('Copy failed:', e);
    }
  }

  onDestroy(() => {
    if (timer) clearTimeout(timer);
  });
</script>

<div class="relative">
  <pre class="text-code whitespace-pre-wrap break-words rounded-md bg-slate-50 dark:bg-[#0d1117] border border-slate-200 dark:border-[#2a3441] p-3 pr-12 text-slate-700 dark:text-slate-300"><code>{code}</code></pre>
  <button
    onclick={copy}
    title="Copy to clipboard"
    aria-label="Copy to clipboard"
    class="absolute top-2 right-2 flex items-center gap-1 px-1.5 py-1 rounded text-[10px] font-bold text-slate-400 hover:text-[#137fec] hover:bg-[#137fec]/10 transition-colors"
  >
    <span class="material-symbols-outlined text-[14px]">{copied ? 'check' : 'content_copy'}</span>
    {#if copied}Copied{/if}
  </button>
</div>
