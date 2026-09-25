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
    <!-- opacity-20 instead of a composed `{color}/20` class — Tailwind can't
         see dynamically built class names, so that utility never gets generated -->
    <div
      class="w-full rounded-xs transition-all {color} {highlightLast && i === values.length - 1 ? '' : 'opacity-20'}"
      style="height: {value}%"
    ></div>
  {/each}
</div>
