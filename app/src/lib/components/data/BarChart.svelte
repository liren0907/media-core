<script lang="ts">
  interface BarItem {
    label: string;
    value: number;
    maxValue: number;
    color?: string;
  }

  interface Props {
    items: BarItem[];
    formatValue?: (v: number) => string;
  }

  let { items, formatValue = (v) => `${v.toFixed(1)}ms` }: Props = $props();
</script>

<div class="flex flex-col gap-1.5">
  {#each items as item}
    <div class="flex items-center gap-2 text-[10px]">
      <span class="text-slate-500 w-24 truncate font-mono" title={item.label}>{item.label}</span>
      <div class="flex-1 h-4 bg-slate-100 dark:bg-[#1f2937] rounded overflow-hidden">
        <div
          class="{item.color || 'bg-[#137fec]'} h-full rounded transition-all"
          style="width: {item.maxValue > 0 ? (item.value / item.maxValue) * 100 : 0}%"
        ></div>
      </div>
      <span class="text-slate-700 dark:text-slate-300 font-mono w-16 text-right">{formatValue(item.value)}</span>
    </div>
  {/each}
</div>
