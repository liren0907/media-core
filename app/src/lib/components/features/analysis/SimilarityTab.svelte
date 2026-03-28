<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Panel, ErrorAlert, EmptyState, RunButton, FormField } from '$lib/components/ui';
  import { DirPicker } from '$lib/components/form';
  import type { AnalysisResult } from '$lib/types';
  import { getFileName } from '$lib/utils/format';
  import { inputClass } from '$lib/utils/styles';

  let similarityInputDir = $state("");
  let similarityOutputDir = $state("");
  let similarityMethod = $state("phash");
  let similarityThreshold = $state(0.95);
  let similarityMinGroupSize = $state(2);

  let result: AnalysisResult | null = $state(null);
  let isProcessing = $state(false);
  let error = $state("");

  async function runSimilarityGrouping() {
    if (!similarityInputDir || !similarityOutputDir) { error = "Please select input and output directories"; return; }
    isProcessing = true; error = ""; result = null;
    try { result = await invoke("execute_similarity_pipeline", { config: { inputDir: similarityInputDir, outputDir: similarityOutputDir, method: similarityMethod, threshold: similarityThreshold, minGroupSize: similarityMinGroupSize } }); }
    catch (e) { error = `Similarity analysis failed: ${e}`; }
    isProcessing = false;
  }
</script>

<div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
    <!-- Configuration Panel -->
    <Panel title="Configuration" icon="tune">
        {#snippet actions()}
            <RunButton loading={isProcessing} onclick={runSimilarityGrouping} />
        {/snippet}
        <div class="p-3 flex flex-col gap-3">
            <DirPicker bind:value={similarityInputDir} label="Input Directory" />
            <DirPicker bind:value={similarityOutputDir} label="Output Directory" />
            <FormField label="Method">
                <select bind:value={similarityMethod} class="{inputClass} w-full">
                    <option value="phash">Perceptual Hash</option>
                    <option value="histogram">Histogram</option>
                    <option value="feature">Feature Matching</option>
                </select>
            </FormField>
            <FormField label="Threshold ({(similarityThreshold * 100).toFixed(0)}%)">
                <input type="range" bind:value={similarityThreshold} min="0.5" max="1" step="0.01" class="w-full" />
            </FormField>
        </div>
    </Panel>

    <!-- Results Panel -->
    <div class="lg:col-span-2">
        <Panel title="Results" icon="analytics">
            <div class="p-4 min-h-[300px]">
                {#if error}
                    <ErrorAlert message={error} />
                {:else if isProcessing}
                    <div class="flex flex-col items-center justify-center h-full gap-3 text-slate-500 py-12">
                        <span class="material-symbols-outlined text-3xl animate-spin">progress_activity</span>
                        <p class="text-xs">Analyzing...</p>
                    </div>
                {:else if result}
                    {#if result.similarityGroups.length > 0}
                        <div class="flex flex-col gap-3">
                            <div class="flex items-center justify-between">
                                <h4 class="text-xs font-bold text-slate-900 dark:text-white uppercase tracking-wider">Groups Found</h4>
                                <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">{result.similarityGroups.length}</span>
                            </div>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                                {#each result.similarityGroups as group}
                                    <div class="p-3 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441]">
                                        <h5 class="text-xs font-bold mb-1">{group.groupName}</h5>
                                        <p class="text-[10px] text-slate-500 mb-1.5">{group.members.length} images</p>
                                        <div class="flex flex-wrap gap-1">
                                            {#each group.members.slice(0, 5) as member}
                                                <span class="px-1.5 py-0.5 rounded bg-slate-200 dark:bg-[#2a3441] text-[10px] font-mono truncate max-w-[100px]" title={member}>{getFileName(member)}</span>
                                            {/each}
                                            {#if group.members.length > 5}
                                                <span class="px-1.5 py-0.5 rounded bg-slate-200 dark:bg-[#2a3441] text-[10px]">+{group.members.length - 5}</span>
                                            {/if}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {:else}
                        <EmptyState icon="group_off" message="No groups found" />
                    {/if}
                {:else}
                    <EmptyState icon="analytics" message="Configure and click &quot;Run&quot;" />
                {/if}
            </div>
        </Panel>
    </div>
</div>
