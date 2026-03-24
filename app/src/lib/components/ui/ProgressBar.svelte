<script lang="ts">
  interface Props {
    percent: number;
    color?: string;
  }

  let { percent, color = 'bg-[#137fec]' }: Props = $props();
</script>

<div class="w-full bg-slate-100 dark:bg-[#1f2937] rounded-full h-1.5 overflow-hidden">
  <div class="{color} h-1.5 rounded-full transition-all" style="width: {percent}%"></div>
</div>
