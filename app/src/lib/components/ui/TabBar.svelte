<script lang="ts">
  interface Tab {
    id: string;
    label: string;
    icon?: string;
  }

  interface Props {
    tabs: Tab[];
    activeTab: string;
    onchange: (tabId: string) => void;
  }

  let { tabs, activeTab, onchange }: Props = $props();
</script>

<div class="flex gap-1.5">
  {#each tabs as tab}
    <button onclick={() => onchange(tab.id)}
      class="flex items-center gap-1 px-3 py-1.5 rounded text-xs font-bold transition-colors {activeTab === tab.id ? 'bg-[#137fec] text-white' : 'bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#1f2937]'}">
      {#if tab.icon}<span class="material-symbols-outlined text-[14px]">{tab.icon}</span>{/if}
      {tab.label}
    </button>
  {/each}
</div>
