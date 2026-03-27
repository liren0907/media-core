<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { PageContent, Panel, StatusBadge, ErrorAlert, EmptyState, RunButton, FormField } from '$lib/components/ui';
  import { FilePicker, DirPicker } from '$lib/components/form';
  import type { AnalysisResult } from '$lib/types';
  import { getFileName } from '$lib/utils/format';
  import { inputClass } from '$lib/utils/styles';

  type AnalysisMode = 'motion' | 'similarity' | 'compare';
  let activeMode: AnalysisMode = $state('motion');

  let motionVideoPath = $state("");
  let motionOutputDir = $state("");
  let motionAlgorithm = $state("frame_diff");
  let motionThreshold = $state(25.0);
  let motionMinArea = $state(500);

  let similarityInputDir = $state("");
  let similarityOutputDir = $state("");
  let similarityMethod = $state("phash");
  let similarityThreshold = $state(0.95);
  let similarityMinGroupSize = $state(2);

  let compareImage1 = $state("");
  let compareImage2 = $state("");
  let compareMethod = $state("phash");
  let compareThreshold = $state(0.95);

  let result: AnalysisResult | null = $state(null);
  let isProcessing = $state(false);
  let error = $state("");


  async function runMotionDetection() {
    if (!motionVideoPath) { error = "Please select a video file"; return; }
    isProcessing = true; error = ""; result = null;
    try { result = await invoke("execute_motion_pipeline", { config: { videoPath: motionVideoPath, outputDir: motionOutputDir || undefined, algorithm: motionAlgorithm, threshold: motionThreshold, minArea: motionMinArea } }); }
    catch (e) { error = `Motion detection failed: ${e}`; }
    isProcessing = false;
  }

  async function runSimilarityGrouping() {
    if (!similarityInputDir || !similarityOutputDir) { error = "Please select input and output directories"; return; }
    isProcessing = true; error = ""; result = null;
    try { result = await invoke("execute_similarity_pipeline", { config: { inputDir: similarityInputDir, outputDir: similarityOutputDir, method: similarityMethod, threshold: similarityThreshold, minGroupSize: similarityMinGroupSize } }); }
    catch (e) { error = `Similarity analysis failed: ${e}`; }
    isProcessing = false;
  }

  async function runImageComparison() {
    if (!compareImage1 || !compareImage2) { error = "Please select both images"; return; }
    isProcessing = true; error = ""; result = null;
    try { result = await invoke("execute_compare_pipeline", { config: { image1: compareImage1, image2: compareImage2, method: compareMethod, threshold: compareThreshold } }); }
    catch (e) { error = `Image comparison failed: ${e}`; }
    isProcessing = false;
  }

</script>

<svelte:head>
  <title>Media Analysis</title>
</svelte:head>

<PageContent>
    <!-- Mode Tabs -->
    <div class="flex gap-1.5">
        <button onclick={() => { activeMode = 'motion'; result = null; error = ''; }}
            class="flex items-center gap-1 px-3 py-1.5 rounded text-xs font-bold transition-colors {activeMode === 'motion' ? 'bg-[#137fec] text-white' : 'bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#1f2937]'}">
            <span class="material-symbols-outlined text-[14px]">motion_photos_on</span> Motion
        </button>
        <button onclick={() => { activeMode = 'similarity'; result = null; error = ''; }}
            class="flex items-center gap-1 px-3 py-1.5 rounded text-xs font-bold transition-colors {activeMode === 'similarity' ? 'bg-[#137fec] text-white' : 'bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#1f2937]'}">
            <span class="material-symbols-outlined text-[14px]">group_work</span> Similarity
        </button>
        <button onclick={() => { activeMode = 'compare'; result = null; error = ''; }}
            class="flex items-center gap-1 px-3 py-1.5 rounded text-xs font-bold transition-colors {activeMode === 'compare' ? 'bg-[#137fec] text-white' : 'bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#1f2937]'}">
            <span class="material-symbols-outlined text-[14px]">compare</span> Compare
        </button>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Configuration Panel -->
        <Panel title="Configuration" icon="tune">
            {#snippet actions()}
                <RunButton loading={isProcessing} onclick={() => { if (activeMode === 'motion') runMotionDetection(); else if (activeMode === 'similarity') runSimilarityGrouping(); else runImageComparison(); }} />
            {/snippet}
            <div class="p-3 flex flex-col gap-3">
                {#if activeMode === 'motion'}
                    <FilePicker bind:value={motionVideoPath} label="Video File" filters={[{ name: "Video", extensions: ["mp4", "avi", "mkv", "mov"] }]} placeholder="Select video..." />
                    <FormField label="Algorithm">
                        <select bind:value={motionAlgorithm} class="{inputClass} w-full">
                            <option value="frame_diff">Frame Difference</option>
                            <option value="mog2">MOG2</option>
                            <option value="knn">KNN</option>
                            <option value="optical_flow">Optical Flow</option>
                        </select>
                    </FormField>
                    <FormField label="Threshold ({motionThreshold})">
                        <input type="range" bind:value={motionThreshold} min="1" max="100" step="1" class="w-full" />
                    </FormField>
                    <FormField label="Min Area ({motionMinArea}px)">
                        <input type="range" bind:value={motionMinArea} min="100" max="5000" step="100" class="w-full" />
                    </FormField>

                {:else if activeMode === 'similarity'}
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

                {:else if activeMode === 'compare'}
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
                {/if}
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
                        {#if activeMode === 'motion' && result.motionEvents.length > 0}
                            <div class="flex flex-col gap-3">
                                <div class="flex items-center justify-between">
                                    <h4 class="text-xs font-bold text-slate-900 dark:text-white uppercase tracking-wider">Motion Events</h4>
                                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">{result.motionEvents.length}</span>
                                </div>
                                <table class="w-full text-xs font-mono">
                                    <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                                        <tr><th class="px-3 py-1.5 font-medium text-left">#</th><th class="px-3 py-1.5 font-medium text-left">START</th><th class="px-3 py-1.5 font-medium text-left">END</th><th class="px-3 py-1.5 font-medium text-left">DURATION</th><th class="px-3 py-1.5 font-medium text-left">TYPE</th></tr>
                                    </thead>
                                    <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                                        {#each result.motionEvents as event, i}
                                            <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50">
                                                <td class="px-3 py-1.5 text-slate-500">{i + 1}</td>
                                                <td class="px-3 py-1.5">{event.startFrame}</td>
                                                <td class="px-3 py-1.5">{event.endFrame}</td>
                                                <td class="px-3 py-1.5">{event.endFrame - event.startFrame}f</td>
                                                <td class="px-3 py-1.5"><StatusBadge status={event.eventType} /></td>
                                            </tr>
                                        {/each}
                                    </tbody>
                                </table>
                            </div>
                        {:else if activeMode === 'motion'}
                            <EmptyState icon="motion_photos_off" message="No motion detected" />
                        {/if}

                        {#if activeMode === 'similarity' && result.similarityGroups.length > 0}
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
                        {:else if activeMode === 'similarity'}
                            <EmptyState icon="group_off" message="No groups found" />
                        {/if}

                        {#if activeMode === 'compare' && result.imageComparison}
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
                        {:else if activeMode === 'compare'}
                            <EmptyState icon="compare" message="Select two images and run comparison" />
                        {/if}
                    {:else}
                        <EmptyState icon="analytics" message="Configure and click &quot;Run&quot;" />
                    {/if}
                </div>
            </Panel>
        </div>
    </div>
</PageContent>
