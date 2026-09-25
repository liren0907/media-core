<script lang="ts">
  import Icon from './Icon.svelte';

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
  {#if loading}<Icon name="sync" class="animate-spin text-[14px]" />{:else}<Icon name="play_arrow" class="text-[14px]" />{/if}
  {label}
</button>
