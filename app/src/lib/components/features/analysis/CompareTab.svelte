<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Panel, StatusBadge, ErrorAlert, EmptyState, RunButton, FormField } from '$lib/components/ui';
  import { FilePicker } from '$lib/components/form';
  import type { AnalysisResult } from '$lib/types';
  import { getFileName } from '$lib/utils/format';
  import { inputClass } from '$lib/utils/styles';

  let compareImage1 = $state("");
  let compareImage2 = $state("");
  let compareMethod = $state("phash");
  let compareThreshold = $state(0.95);

  let result: AnalysisResult | null = $state(null);
  let isProcessing = $state(false);
  let error = $state("");

  async function runImageComparison() {
    if (!compareImage1 || !compareImage2) { error = "Please select both images"; return; }
    isProcessing = true; error = ""; result = null;
    try { result = await invoke("execute_compare_pipeline", { config: { image1: compareImage1, image2: compareImage2, method: compareMethod, threshold: compareThreshold } }); }
    catch (e) { error = `Image comparison failed: ${e}`; }
    isProcessing = false;
  }
</script>

<div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
    <!-- Configuration Panel -->
    <Panel title="Configuration" icon="tune">
        {#snippet actions()}
            <RunButton loading={isProcessing} onclick={runImageComparison} />
        {/snippet}
        <div class="p-3 flex flex-col gap-3">
            <FilePicker bind:value={compareImage1} label="Image 1" filters={[{ name: "Image", extensions: ["jpg", "jpeg", "png", "bmp"] }]} />
            <FilePicker bind:value={compareImage2} label="Image 2" filters={[{ name: "Image", extensions: ["jpg", "jpeg", "png", "bmp"] }]} />
            <FormField label="Method">
                <select bind:value={compareMethod} class="{inputClass} w-full">
                    <option value="phash">Perceptual Hash</option>
                    <option value="histogram">Histogram</option>
                    <option value="feature">Feature Matching</option>
                </select>
            </FormField>
            <FormField label="Threshold ({(compareThreshold * 100).toFixed(0)}%)">
                <input type="range" bind:value={compareThreshold} min="0.5" max="1" step="0.01" class="w-full" />
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
                    {#if result.imageComparison}
                        <div class="flex flex-col gap-4">
                            <div class="text-center p-6 rounded-lg bg-slate-50 dark:bg-[#1f2937]/50">
                                <div class="text-4xl font-bold font-display {result.imageComparison.isDuplicate ? 'text-green-500' : 'text-orange-500'}">
                                    {(result.imageComparison.similarityScore * 100).toFixed(1)}%
                                </div>
                                <div class="mt-1 text-xs text-slate-500">Similarity Score</div>
                                <div class="mt-3">
                                    <StatusBadge status={result.imageComparison.isDuplicate ? 'duplicate' : 'different'} colorMap={{
                                        duplicate: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
                                        different: 'bg-orange-500/10 text-orange-600 dark:text-orange-400 border-orange-500/20',
                                    }} />
                                </div>
                            </div>
                            <div class="grid grid-cols-2 gap-2 text-xs">
                                <div class="p-2 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441]">
                                    <span class="text-slate-500">Image 1:</span> <span class="font-mono">{getFileName(result.imageComparison.image1)}</span>
                                </div>
                                <div class="p-2 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441]">
                                    <span class="text-slate-500">Image 2:</span> <span class="font-mono">{getFileName(result.imageComparison.image2)}</span>
                                </div>
                            </div>
                        </div>
                    {:else}
                        <EmptyState icon="compare" message="Select two images and run comparison" />
                    {/if}
                {:else}
                    <EmptyState icon="analytics" message="Configure and click &quot;Run&quot;" />
                {/if}
            </div>
        </Panel>
    </div>
</div>
