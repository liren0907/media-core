<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Panel, StatusBadge, ErrorAlert, EmptyState, RunButton, FormField } from '$lib/components/ui';
  import { FilePicker } from '$lib/components/form';
  import type { AnalysisResult } from '$lib/types';
  import { inputClass } from '$lib/utils/styles';

  let motionVideoPath = $state("");
  let motionOutputDir = $state("");
  let motionAlgorithm = $state("frame_diff");
  let motionThreshold = $state(25.0);
  let motionMinArea = $state(500);

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
</script>

<div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
    <!-- Configuration Panel -->
    <Panel title="Configuration" icon="tune">
        {#snippet actions()}
            <RunButton loading={isProcessing} onclick={runMotionDetection} />
        {/snippet}
        <div class="p-3 flex flex-col gap-3">
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
                    {#if result.motionEvents.length > 0}
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
                    {:else}
                        <EmptyState icon="motion_photos_off" message="No motion detected" />
                    {/if}
                {:else}
                    <EmptyState icon="analytics" message="Configure and click &quot;Run&quot;" />
                {/if}
            </div>
        </Panel>
    </div>
</div>
