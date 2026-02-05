<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  // Analysis mode
  type AnalysisMode = 'motion' | 'similarity' | 'compare';
  let activeMode: AnalysisMode = 'motion';

  // Motion detection state
  let motionVideoPath = "";
  let motionOutputDir = "";
  let motionAlgorithm = "frame_diff";
  let motionThreshold = 25.0;
  let motionMinArea = 500;

  // Similarity grouping state
  let similarityInputDir = "";
  let similarityOutputDir = "";
  let similarityMethod = "phash";
  let similarityThreshold = 0.95;
  let similarityMinGroupSize = 2;

  // Image comparison state
  let compareImage1 = "";
  let compareImage2 = "";
  let compareMethod = "phash";
  let compareThreshold = 0.95;

  // Results state
  interface MotionEvent {
    startFrame: number;
    endFrame: number;
    eventType: string;
  }

  interface SimilarityGroup {
    groupName: string;
    members: string[];
  }

  interface ImageComparisonResult {
    image1: string;
    image2: string;
    similarityScore: number;
    isDuplicate: boolean;
  }

  interface AnalysisResult {
    motionEvents: MotionEvent[];
    similarityGroups: SimilarityGroup[];
    imageComparison: ImageComparisonResult | null;
  }

  let result: AnalysisResult | null = null;
  let isProcessing = false;
  let error = "";

  // File selection handlers
  async function selectVideoFile() {
    const path = await open({
      filters: [{ name: "Video", extensions: ["mp4", "avi", "mkv", "mov"] }],
    });
    if (path) motionVideoPath = path as string;
  }

  async function selectMotionOutputDir() {
    const path = await open({ directory: true });
    if (path) motionOutputDir = path as string;
  }

  async function selectSimilarityInputDir() {
    const path = await open({ directory: true });
    if (path) similarityInputDir = path as string;
  }

  async function selectSimilarityOutputDir() {
    const path = await open({ directory: true });
    if (path) similarityOutputDir = path as string;
  }

  async function selectCompareImage1() {
    const path = await open({
      filters: [{ name: "Image", extensions: ["jpg", "jpeg", "png", "bmp"] }],
    });
    if (path) compareImage1 = path as string;
  }

  async function selectCompareImage2() {
    const path = await open({
      filters: [{ name: "Image", extensions: ["jpg", "jpeg", "png", "bmp"] }],
    });
    if (path) compareImage2 = path as string;
  }

  // Analysis execution handlers
  async function runMotionDetection() {
    if (!motionVideoPath) {
      error = "Please select a video file";
      return;
    }

    isProcessing = true;
    error = "";
    result = null;

    try {
      result = await invoke("execute_motion_pipeline", {
        config: {
          videoPath: motionVideoPath,
          outputDir: motionOutputDir || undefined,
          algorithm: motionAlgorithm,
          threshold: motionThreshold,
          minArea: motionMinArea,
        },
      });
    } catch (e) {
      error = `Motion detection failed: ${e}`;
    }

    isProcessing = false;
  }

  async function runSimilarityGrouping() {
    if (!similarityInputDir || !similarityOutputDir) {
      error = "Please select input and output directories";
      return;
    }

    isProcessing = true;
    error = "";
    result = null;

    try {
      result = await invoke("execute_similarity_pipeline", {
        config: {
          inputDir: similarityInputDir,
          outputDir: similarityOutputDir,
          method: similarityMethod,
          threshold: similarityThreshold,
          minGroupSize: similarityMinGroupSize,
        },
      });
    } catch (e) {
      error = `Similarity analysis failed: ${e}`;
    }

    isProcessing = false;
  }

  async function runImageComparison() {
    if (!compareImage1 || !compareImage2) {
      error = "Please select both images";
      return;
    }

    isProcessing = true;
    error = "";
    result = null;

    try {
      result = await invoke("execute_compare_pipeline", {
        config: {
          image1: compareImage1,
          image2: compareImage2,
          method: compareMethod,
          threshold: compareThreshold,
        },
      });
    } catch (e) {
      error = `Image comparison failed: ${e}`;
    }

    isProcessing = false;
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path;
  }
</script>

<svelte:head>
  <title>Media Analysis</title>
</svelte:head>

<div class="p-8 max-w-[1400px] w-full mx-auto space-y-8">
    <!-- Page Header -->
    <div class="flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div class="flex flex-col gap-2 max-w-2xl">
            <h2 class="text-slate-900 dark:text-white text-3xl font-bold font-display tracking-tight">Media Analysis</h2>
            <p class="text-slate-500 dark:text-[#9dabb9] text-base">Detect motion, find similar images, and compare media files using computer vision algorithms.</p>
        </div>
    </div>

    <!-- Mode Tabs -->
    <div class="flex gap-2">
        <button 
            on:click={() => { activeMode = 'motion'; result = null; error = ''; }}
            class="px-4 py-2 rounded-lg font-medium text-sm transition-colors {activeMode === 'motion' ? 'bg-[#137fec] text-white' : 'bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#1f2937]'}"
        >
            <span class="material-symbols-outlined text-sm align-middle mr-1">motion_photos_on</span>
            Motion Detection
        </button>
        <button 
            on:click={() => { activeMode = 'similarity'; result = null; error = ''; }}
            class="px-4 py-2 rounded-lg font-medium text-sm transition-colors {activeMode === 'similarity' ? 'bg-[#137fec] text-white' : 'bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#1f2937]'}"
        >
            <span class="material-symbols-outlined text-sm align-middle mr-1">group_work</span>
            Similarity Grouping
        </button>
        <button 
            on:click={() => { activeMode = 'compare'; result = null; error = ''; }}
            class="px-4 py-2 rounded-lg font-medium text-sm transition-colors {activeMode === 'compare' ? 'bg-[#137fec] text-white' : 'bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#1f2937]'}"
        >
            <span class="material-symbols-outlined text-sm align-middle mr-1">compare</span>
            Image Comparison
        </button>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Configuration Panel -->
        <div class="lg:col-span-1 flex flex-col rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] overflow-hidden shadow-sm">
            <div class="px-6 py-4 border-b border-slate-200 dark:border-[#283039] bg-slate-50 dark:bg-[#1a242d]">
                <h3 class="text-base font-bold text-slate-900 dark:text-white font-display">Configuration</h3>
            </div>
            
            <div class="p-6 space-y-4 flex-1">
                {#if activeMode === 'motion'}
                    <!-- Motion Detection Config -->
                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Video File</label>
                        <div class="flex gap-2">
                            <input type="text" readonly value={motionVideoPath ? getFileName(motionVideoPath) : ''} 
                                class="flex-1 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-3 py-2 text-sm" 
                                placeholder="Select video..." />
                            <button on:click={selectVideoFile} class="px-3 py-2 bg-slate-100 dark:bg-[#283039] rounded-lg text-sm font-medium">Browse</button>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Algorithm</label>
                        <select bind:value={motionAlgorithm} class="w-full bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-3 py-2 text-sm">
                            <option value="frame_diff">Frame Difference (Fast)</option>
                            <option value="mog2">MOG2 Background Subtraction</option>
                            <option value="knn">KNN Background Subtraction</option>
                            <option value="optical_flow">Optical Flow (Dense)</option>
                        </select>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Threshold ({motionThreshold})</label>
                        <input type="range" bind:value={motionThreshold} min="1" max="100" step="1"
                            class="w-full" />
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Min Area ({motionMinArea} px)</label>
                        <input type="range" bind:value={motionMinArea} min="100" max="5000" step="100"
                            class="w-full" />
                    </div>

                {:else if activeMode === 'similarity'}
                    <!-- Similarity Grouping Config -->
                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Input Directory</label>
                        <div class="flex gap-2">
                            <input type="text" readonly value={similarityInputDir ? getFileName(similarityInputDir) : ''} 
                                class="flex-1 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-3 py-2 text-sm" 
                                placeholder="Select folder..." />
                            <button on:click={selectSimilarityInputDir} class="px-3 py-2 bg-slate-100 dark:bg-[#283039] rounded-lg text-sm font-medium">Browse</button>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Output Directory</label>
                        <div class="flex gap-2">
                            <input type="text" readonly value={similarityOutputDir ? getFileName(similarityOutputDir) : ''} 
                                class="flex-1 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-3 py-2 text-sm" 
                                placeholder="Select folder..." />
                            <button on:click={selectSimilarityOutputDir} class="px-3 py-2 bg-slate-100 dark:bg-[#283039] rounded-lg text-sm font-medium">Browse</button>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Method</label>
                        <select bind:value={similarityMethod} class="w-full bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-3 py-2 text-sm">
                            <option value="phash">Perceptual Hash (Recommended)</option>
                            <option value="histogram">Histogram Comparison</option>
                            <option value="feature">Feature Matching (SIFT)</option>
                        </select>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Similarity Threshold ({(similarityThreshold * 100).toFixed(0)}%)</label>
                        <input type="range" bind:value={similarityThreshold} min="0.5" max="1" step="0.01"
                            class="w-full" />
                    </div>

                {:else if activeMode === 'compare'}
                    <!-- Image Comparison Config -->
                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Image 1</label>
                        <div class="flex gap-2">
                            <input type="text" readonly value={compareImage1 ? getFileName(compareImage1) : ''} 
                                class="flex-1 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-3 py-2 text-sm" 
                                placeholder="Select image..." />
                            <button on:click={selectCompareImage1} class="px-3 py-2 bg-slate-100 dark:bg-[#283039] rounded-lg text-sm font-medium">Browse</button>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Image 2</label>
                        <div class="flex gap-2">
                            <input type="text" readonly value={compareImage2 ? getFileName(compareImage2) : ''} 
                                class="flex-1 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-3 py-2 text-sm" 
                                placeholder="Select image..." />
                            <button on:click={selectCompareImage2} class="px-3 py-2 bg-slate-100 dark:bg-[#283039] rounded-lg text-sm font-medium">Browse</button>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Method</label>
                        <select bind:value={compareMethod} class="w-full bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-3 py-2 text-sm">
                            <option value="phash">Perceptual Hash</option>
                            <option value="histogram">Histogram</option>
                            <option value="feature">Feature Matching</option>
                        </select>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-slate-700 dark:text-slate-300">Duplicate Threshold ({(compareThreshold * 100).toFixed(0)}%)</label>
                        <input type="range" bind:value={compareThreshold} min="0.5" max="1" step="0.01"
                            class="w-full" />
                    </div>
                {/if}
            </div>

            <!-- Action Button -->
            <div class="p-6 pt-0">
                <button 
                    on:click={() => {
                        if (activeMode === 'motion') runMotionDetection();
                        else if (activeMode === 'similarity') runSimilarityGrouping();
                        else runImageComparison();
                    }}
                    disabled={isProcessing}
                    class="w-full py-3 rounded-lg bg-[#137fec] hover:bg-blue-600 text-white font-bold text-sm shadow-lg shadow-[#137fec]/20 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 transition-all"
                >
                    {#if isProcessing}
                        <span class="material-symbols-outlined animate-spin text-sm">sync</span>
                        Processing...
                    {:else}
                        <span class="material-symbols-outlined text-sm">play_arrow</span>
                        Run Analysis
                    {/if}
                </button>
            </div>
        </div>

        <!-- Results Panel -->
        <div class="lg:col-span-2 flex flex-col rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] overflow-hidden shadow-sm">
            <div class="px-6 py-4 border-b border-slate-200 dark:border-[#283039] bg-slate-50 dark:bg-[#1a242d]">
                <h3 class="text-base font-bold text-slate-900 dark:text-white font-display">Results</h3>
            </div>
            
            <div class="p-6 flex-1 overflow-auto">
                {#if error}
                    <div class="p-4 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-sm">
                        {error}
                    </div>
                {:else if isProcessing}
                    <div class="flex flex-col items-center justify-center h-full gap-4 text-slate-500">
                        <span class="material-symbols-outlined text-4xl animate-spin">progress_activity</span>
                        <p>Analyzing media...</p>
                    </div>
                {:else if result}
                    <!-- Motion Detection Results -->
                    {#if activeMode === 'motion' && result.motionEvents.length > 0}
                        <div class="space-y-4">
                            <div class="flex items-center justify-between">
                                <h4 class="font-bold text-slate-900 dark:text-white">Motion Events Detected</h4>
                                <span class="px-2 py-1 rounded bg-[#137fec]/10 text-[#137fec] text-sm font-bold">{result.motionEvents.length} events</span>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="w-full text-sm">
                                    <thead class="text-left text-slate-500 border-b border-slate-200 dark:border-[#283039]">
                                        <tr>
                                            <th class="pb-2 pr-4">#</th>
                                            <th class="pb-2 pr-4">Start Frame</th>
                                            <th class="pb-2 pr-4">End Frame</th>
                                            <th class="pb-2 pr-4">Duration</th>
                                            <th class="pb-2">Type</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-slate-100 dark:divide-[#283039]">
                                        {#each result.motionEvents as event, i}
                                            <tr>
                                                <td class="py-2 pr-4 font-mono text-slate-500">{i + 1}</td>
                                                <td class="py-2 pr-4 font-mono">{event.startFrame}</td>
                                                <td class="py-2 pr-4 font-mono">{event.endFrame}</td>
                                                <td class="py-2 pr-4 font-mono">{event.endFrame - event.startFrame} frames</td>
                                                <td class="py-2">
                                                    <span class="px-2 py-0.5 rounded bg-green-100 dark:bg-green-900/20 text-green-600 dark:text-green-400 text-xs font-medium">
                                                        {event.eventType}
                                                    </span>
                                                </td>
                                            </tr>
                                        {/each}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    {:else if activeMode === 'motion'}
                        <div class="flex flex-col items-center justify-center h-full gap-4 text-slate-500">
                            <span class="material-symbols-outlined text-4xl">motion_photos_off</span>
                            <p>No motion detected in the video</p>
                        </div>
                    {/if}

                    <!-- Similarity Grouping Results -->
                    {#if activeMode === 'similarity' && result.similarityGroups.length > 0}
                        <div class="space-y-4">
                            <div class="flex items-center justify-between">
                                <h4 class="font-bold text-slate-900 dark:text-white">Similarity Groups Found</h4>
                                <span class="px-2 py-1 rounded bg-[#137fec]/10 text-[#137fec] text-sm font-bold">{result.similarityGroups.length} groups</span>
                            </div>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                {#each result.similarityGroups as group}
                                    <div class="p-4 rounded-lg bg-slate-50 dark:bg-[#1a242d] border border-slate-200 dark:border-[#283039]">
                                        <h5 class="font-bold text-sm mb-2">{group.groupName}</h5>
                                        <p class="text-xs text-slate-500 mb-2">{group.members.length} similar images</p>
                                        <div class="flex flex-wrap gap-1">
                                            {#each group.members.slice(0, 5) as member}
                                                <span class="px-2 py-0.5 rounded bg-slate-200 dark:bg-[#283039] text-xs font-mono truncate max-w-[120px]" title={member}>
                                                    {getFileName(member)}
                                                </span>
                                            {/each}
                                            {#if group.members.length > 5}
                                                <span class="px-2 py-0.5 rounded bg-slate-200 dark:bg-[#283039] text-xs">+{group.members.length - 5} more</span>
                                            {/if}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {:else if activeMode === 'similarity'}
                        <div class="flex flex-col items-center justify-center h-full gap-4 text-slate-500">
                            <span class="material-symbols-outlined text-4xl">group_off</span>
                            <p>No similar image groups found</p>
                        </div>
                    {/if}

                    <!-- Image Comparison Results -->
                    {#if activeMode === 'compare' && result.imageComparison}
                        <div class="space-y-6">
                            <div class="text-center p-8 rounded-xl bg-slate-50 dark:bg-[#1a242d]">
                                <div class="text-6xl font-bold {result.imageComparison.isDuplicate ? 'text-green-500' : 'text-orange-500'}">
                                    {(result.imageComparison.similarityScore * 100).toFixed(1)}%
                                </div>
                                <div class="mt-2 text-lg font-medium text-slate-600 dark:text-slate-300">
                                    Similarity Score
                                </div>
                                <div class="mt-4">
                                    {#if result.imageComparison.isDuplicate}
                                        <span class="px-4 py-2 rounded-full bg-green-100 dark:bg-green-900/20 text-green-600 dark:text-green-400 font-bold">
                                            Likely Duplicate
                                        </span>
                                    {:else}
                                        <span class="px-4 py-2 rounded-full bg-orange-100 dark:bg-orange-900/20 text-orange-600 dark:text-orange-400 font-bold">
                                            Different Images
                                        </span>
                                    {/if}
                                </div>
                            </div>
                            
                            <div class="grid grid-cols-2 gap-4 text-sm">
                                <div class="p-3 rounded-lg bg-slate-50 dark:bg-[#1a242d]">
                                    <span class="text-slate-500">Image 1:</span>
                                    <span class="font-mono ml-2">{getFileName(result.imageComparison.image1)}</span>
                                </div>
                                <div class="p-3 rounded-lg bg-slate-50 dark:bg-[#1a242d]">
                                    <span class="text-slate-500">Image 2:</span>
                                    <span class="font-mono ml-2">{getFileName(result.imageComparison.image2)}</span>
                                </div>
                            </div>
                        </div>
                    {:else if activeMode === 'compare'}
                        <div class="flex flex-col items-center justify-center h-full gap-4 text-slate-500">
                            <span class="material-symbols-outlined text-4xl">compare</span>
                            <p>Select two images and run comparison</p>
                        </div>
                    {/if}
                {:else}
                    <div class="flex flex-col items-center justify-center h-full gap-4 text-slate-500">
                        <span class="material-symbols-outlined text-4xl">analytics</span>
                        <p>Configure analysis parameters and click "Run Analysis"</p>
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>
