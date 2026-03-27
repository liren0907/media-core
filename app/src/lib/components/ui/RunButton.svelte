<script lang="ts">
  interface Props {
    loading: boolean;
    disabled?: boolean;
    label?: string;
    onclick: () => void;
  }

  let { loading, disabled = false, label = 'Run', onclick }: Props = $props();
</script>

<button {onclick} disabled={loading || disabled}
  class="flex items-center gap-1 px-2 py-1 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold disabled:opacity-50 transition-colors">
  {#if loading}<span class="material-symbols-outlined animate-spin text-[14px]">sync</span>{:else}<span class="material-symbols-outlined text-[14px]">play_arrow</span>{/if}
  {label}
</button>
