<script lang="ts">
  import { Panel } from '$lib/components/ui';

  interface Props {
    detectionTimeline: number[];
    maxDetection: number;
    selectedLabels: string[];
  }

  let { detectionTimeline, maxDetection, selectedLabels }: Props = $props();

  let allSelected = $derived(selectedLabels.length > 0);
</script>

<Panel title="Detection Timeline" icon="timeline">
    {#snippet actions()}
        <div class="flex gap-3 items-center text-[10px]">
            <span class="flex items-center gap-1 text-slate-500"><span class="size-1.5 rounded-full bg-[#137fec]"></span> Detections/Frame</span>
            {#if detectionTimeline.length > 0}
                <span class="text-slate-500 font-mono">{detectionTimeline.length} segments</span>
            {/if}
        </div>
    {/snippet}
    <div class="p-3">
        <div class="flex items-end gap-[2px] h-28 overflow-hidden">
            {#if detectionTimeline.length > 0}
                {#each detectionTimeline as value, i}
                    <div class="flex-1 rounded-t-sm transition-all hover:opacity-100 group relative cursor-pointer {allSelected ? 'bg-[#137fec]' : 'bg-[#137fec]/60'}" style="height: {(value / maxDetection) * 100}%">
                        <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 px-1.5 py-0.5 bg-slate-800 text-white text-[10px] rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-10">
                            Frame {i + 1}: {value}
                        </div>
                    </div>
                {/each}
            {:else}
                <div class="w-full h-full flex items-center justify-center text-slate-500 text-xs">Load annotation to see timeline</div>
            {/if}
        </div>
        {#if detectionTimeline.length > 0}
            <div class="flex justify-between mt-1.5 text-meta">
                <span>Start</span><span>25%</span><span>50%</span><span>75%</span><span>End</span>
            </div>
        {/if}
    </div>
</Panel>
