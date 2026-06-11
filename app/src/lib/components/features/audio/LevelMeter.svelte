<script lang="ts">
  import { ProgressBar } from '$lib/components/ui';
  import { SparklineBar } from '$lib/components/data';

  interface Props {
    /** Instantaneous input level, 0–100 */
    level: number;
    /** Rolling level history, oldest first (0–100 each) */
    history: number[];
  }

  let { level, history }: Props = $props();
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-center justify-between">
    <span class="text-stat-label">Signal</span>
    <span class="text-meta tabular-nums">{Math.round(level)}%</span>
  </div>
  <ProgressBar percent={level} color="bg-[#137fec]" />
  <SparklineBar values={history} height="h-12" highlightLast={false} />
</div>
