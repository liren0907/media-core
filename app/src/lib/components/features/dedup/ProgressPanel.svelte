<script lang="ts">
  import { Panel, ProgressBar } from '$lib/components/ui';

  interface Props {
    title: string;
    percent: number;
    message?: string;
    detail?: string;
  }

  let { title, percent, message = '', detail = '' }: Props = $props();
</script>

<Panel {title} icon="hourglass_top">
  <div class="flex flex-col gap-2">
    <ProgressBar {percent} color="bg-[#137fec]" />
    <div class="flex items-center justify-between text-[10px]">
      <span class="text-muted">{message}</span>
      <span class="text-slate-500 font-mono">{percent.toFixed(0)}%</span>
    </div>
    {#if detail}
      <div class="text-meta truncate">{detail}</div>
    {/if}
  </div>
</Panel>
