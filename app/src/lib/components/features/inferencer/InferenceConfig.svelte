<script lang="ts">
  import { Panel, StatCard, StatusBadge, ProgressBar, ErrorAlert, FormField } from '$lib/components/ui';
  import { inputClass } from '$lib/utils/styles';

  interface Props {
    videoPath: string;
    annotationPath: string;
    availableLabels: string[];
    selectedLabels: string[];
    labelCounts: Record<string, number>;
    totalDetections: number;
    filteredDetections: number;
    topLabels: [string, number][];
    classProgress: number;
    isProcessing: boolean;
    processedVideoPath: string;
    errorMessage: string;
    onopenVideo: () => void;
    onopenAnnotation: () => void;
    onprocess: () => void;
    onselectAll: () => void;
    ondeselectAll: () => void;
    ontoggleLabel: (label: string) => void;
  }

  let {
    videoPath,
    annotationPath,
    availableLabels,
    selectedLabels,
    labelCounts,
    totalDetections,
    filteredDetections,
    topLabels,
    classProgress,
    isProcessing,
    processedVideoPath,
    errorMessage,
    onopenVideo,
    onopenAnnotation,
    onprocess,
    onselectAll,
    ondeselectAll,
    ontoggleLabel,
  }: Props = $props();
</script>

<div class="flex flex-col gap-3">
    <!-- Input Sources -->
    <Panel title="Input Sources" icon="video_file">
        <div class="p-3 flex flex-col gap-3">
            <FormField label="Video" id="sourceVideoPath">
                <div class="flex gap-1.5">
                    <input id="sourceVideoPath" type="text" value={videoPath ? '...' + videoPath.slice(-25) : ''} readonly class="{inputClass} flex-1" placeholder="No file" />
                    <button onclick={onopenVideo} class="px-2 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold transition-colors">Browse</button>
                </div>
            </FormField>
            <FormField label="Annotation JSON" id="annotationDataPath">
                <div class="flex gap-1.5">
                    <input id="annotationDataPath" type="text" value={annotationPath ? '...' + annotationPath.slice(-25) : ''} readonly class="{inputClass} flex-1" placeholder="No file" />
                    <button onclick={onopenAnnotation} class="px-2 py-2 bg-slate-100 dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] rounded text-[10px] font-bold transition-colors">Load</button>
                </div>
            </FormField>
        </div>
    </Panel>

    <!-- Labels -->
    <Panel title="Object Classes" icon="label">
        {#snippet actions()}
            <div class="flex gap-1">
                <button onclick={onselectAll} class="text-[10px] font-bold text-[#137fec] hover:text-blue-400">All</button>
                <button onclick={ondeselectAll} class="text-[10px] font-bold text-slate-400 hover:text-slate-600 dark:hover:text-white">None</button>
            </div>
        {/snippet}
        <div class="p-3">
            {#if availableLabels.length > 0}
                <div class="flex flex-col gap-1 max-h-[200px] overflow-y-auto">
                    {#each availableLabels as label}
                        <label class="flex items-center gap-2 p-1.5 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441] hover:border-[#137fec]/50 cursor-pointer transition-colors text-xs">
                            <input type="checkbox" checked={selectedLabels.includes(label)} onchange={() => ontoggleLabel(label)} class="rounded border-slate-300 dark:border-slate-600 text-[#137fec] w-3.5 h-3.5" />
                            <span class="flex-1 font-mono text-slate-700 dark:text-slate-300 truncate">{label}</span>
                            {#if labelCounts[label]}<span class="text-[10px] text-slate-500 font-mono">{labelCounts[label]}</span>{/if}
                        </label>
                    {/each}
                </div>
            {:else}
                <div class="py-4 text-center text-[10px] text-slate-500">Load annotation file to see classes</div>
            {/if}
        </div>
    </Panel>

    <!-- Stats -->
    <Panel title="Stats" icon="analytics">
        <div class="p-3 flex flex-col gap-2 text-xs font-mono">
            <div class="flex justify-between"><span class="text-slate-500">Total Detections</span><span class="font-bold text-slate-900 dark:text-white">{totalDetections.toLocaleString()}</span></div>
            <div class="flex justify-between items-center">
                <span class="text-slate-500">Selected</span>
                <span class="font-bold">{selectedLabels.length}/{availableLabels.length}</span>
            </div>
            <ProgressBar percent={classProgress} />
            <div class="flex justify-between"><span class="text-slate-500">Filtered</span><span class="font-bold text-[#137fec]">{filteredDetections.toLocaleString()}</span></div>
            {#if topLabels.length > 0}
                <div class="border-t border-slate-100 dark:border-[#2a3441] pt-2 mt-1">
                    <span class="text-[10px] text-slate-500 uppercase tracking-wider">Top Classes</span>
                    {#each topLabels.slice(0, 3) as [label, count]}
                        <div class="flex justify-between mt-1"><span class="text-slate-600 dark:text-slate-400 truncate max-w-[100px]" title={label}>{label}</span><span class="text-slate-500">{count}</span></div>
                    {/each}
                </div>
            {/if}
            <div class="flex justify-between border-t border-slate-100 dark:border-[#2a3441] pt-2">
                <span class="text-slate-500">Status</span>
                <StatusBadge status={isProcessing ? 'connecting' : processedVideoPath ? 'active' : 'idle'} />
            </div>
        </div>
    </Panel>

    <!-- Action -->
    {#if errorMessage}
        <ErrorAlert message={errorMessage} />
    {/if}
    <button onclick={onprocess} disabled={isProcessing || !videoPath || !annotationPath} class="w-full py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-xs font-bold disabled:opacity-50 flex items-center justify-center gap-1.5 transition-colors">
        {#if isProcessing}<span class="material-symbols-outlined animate-spin text-[16px]">sync</span> Processing...{:else}<span class="material-symbols-outlined text-[16px]">play_arrow</span> Run Inference{/if}
    </button>
</div>
