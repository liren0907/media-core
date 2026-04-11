<script lang="ts">
  interface Props {
    values: number[];
    color?: string;
    highlightLast?: boolean;
    height?: string;
  }

  let { values, color = 'bg-[#137fec]', highlightLast = true, height = 'h-5' }: Props = $props();
</script>

<div class="{height} w-full flex items-end gap-0.5 mt-2">
  {#each values as value, i}
    <div
      class="w-full rounded-sm transition-all {highlightLast && i === values.length - 1 ? color : `${color}/20`}"
      style="height: {value}%"
    ></div>
  {/each}
</div>
