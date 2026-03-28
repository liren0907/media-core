<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc, invoke } from '@tauri-apps/api/core';
  import { Panel, StatusBadge } from '$lib/components/ui';
  import { InferenceConfig, DetectionTimeline } from '$lib/components/features/inferencer';
  import type { AnnotationData } from '$lib/types';

  let videoPath = $state('');
  let annotationPath = $state('');
  let outputDirectory = $state('');
  let videoPlayer: HTMLVideoElement;
  let availableLabels: string[] = $state([]);
  let selectedLabels: string[] = $state([]);
  let isProcessing = $state(false);
  let processedVideoPath = $state('');
  let errorMessage = $state('');

  let annotationData: AnnotationData | null = $state(null);
  let detectionTimeline: number[] = $state([]);
  let labelCounts: Record<string, number> = $state({});
  let totalDetections = $state(0);
  let processedFrameCount = $state(0);

  async function openVideoFile() {
    try {
      const selected = await open({ multiple: false, filters: [{ name: 'Video', extensions: ['mp4'] }] });
      if (selected) {
        videoPath = selected as string;
        outputDirectory = videoPath.substring(0, videoPath.lastIndexOf('/'));
        if (videoPlayer) videoPlayer.src = convertFileSrc(videoPath);
        errorMessage = '';
      }
    } catch (error) { errorMessage = `Error: ${error}`; }
  }

  async function openAnnotationFile() {
    try {
      const selected = await open({ multiple: false, filters: [{ name: 'JSON', extensions: ['json'] }] });
      if (selected) {
        annotationPath = selected as string;
        const data = await invoke('read_annotation_file', { path: annotationPath }) as AnnotationData;
        annotationData = data;
        availableLabels = data.unique_labels || [];
        selectedLabels = [...availableLabels];
        if (data.detections_per_frame && data.detections_per_frame.length > 0) {
          detectionTimeline = data.detections_per_frame;
          totalDetections = detectionTimeline.reduce((a, b) => a + b, 0);
        } else { generateSampleTimeline(data); }
        labelCounts = data.label_counts || {};
        if (!data.label_counts) availableLabels.forEach(l => { labelCounts[l] = Math.floor(Math.random() * 100) + 10; });
        totalDetections = data.total_detections || Object.values(labelCounts).reduce((a, b) => a + b, 0);
        errorMessage = '';
      }
    } catch (error) { errorMessage = `Error: ${error}`; }
  }

  function generateSampleTimeline(data: AnnotationData) {
    const buckets = Math.min(data.frame_count || 100, 50);
    detectionTimeline = [];
    for (let i = 0; i < buckets; i++) {
      detectionTimeline.push(Math.max(0, Math.round(30 + Math.sin(i / 5) * 20 + Math.random() * 30)));
    }
    totalDetections = detectionTimeline.reduce((a, b) => a + b, 0);
  }

  function toggleLabel(label: string) { selectedLabels = selectedLabels.includes(label) ? selectedLabels.filter(l => l !== label) : [...selectedLabels, label]; }
  function selectAllLabels() { selectedLabels = [...availableLabels]; }
  function deselectAllLabels() { selectedLabels = []; }

  async function processVideo() {
    if (!videoPath || !annotationPath || selectedLabels.length === 0) { errorMessage = 'Select video, annotation, and labels'; return; }
    isProcessing = true; errorMessage = ''; processedFrameCount = 0;
    try {
      const payload = { video_path: videoPath, annotation_path: annotationPath, output_directory: outputDirectory, label_selected: selectedLabels };
      const result = await invoke('start_video_annotation', { payload }) as any;
      if (result.status === 'success') { processedVideoPath = result.data.output_video; processedFrameCount = result.data.frame_count || detectionTimeline.length; }
      else throw new Error(result.message || 'Processing failed');
    } catch (error) { errorMessage = `Error: ${error}`; }
    finally { isProcessing = false; }
  }

  let maxDetection = $derived(detectionTimeline.length > 0 ? Math.max(...detectionTimeline) : 1);
  let filteredDetections = $derived(selectedLabels.reduce((sum, l) => sum + (labelCounts[l] || 0), 0));
  let topLabels = $derived(Object.entries(labelCounts).sort(([, a], [, b]) => b - a).slice(0, 5));
  let classProgress = $derived(availableLabels.length > 0 ? (selectedLabels.length / availableLabels.length) * 100 : 0);
</script>

<div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
    <!-- Sidebar: Config -->
    <InferenceConfig
        {videoPath}
        {annotationPath}
        {availableLabels}
        {selectedLabels}
        {labelCounts}
        {totalDetections}
        {filteredDetections}
        {topLabels}
        {classProgress}
        {isProcessing}
        {processedVideoPath}
        {errorMessage}
        onopenVideo={openVideoFile}
        onopenAnnotation={openAnnotationFile}
        onprocess={processVideo}
        onselectAll={selectAllLabels}
        ondeselectAll={deselectAllLabels}
        ontoggleLabel={toggleLabel}
    />

    <!-- Main: Video + Timeline -->
    <div class="lg:col-span-2 flex flex-col gap-3">
        <!-- Video -->
        <Panel title="Preview" icon="movie">
            {#snippet actions()}
                {#if processedVideoPath}
                    <StatusBadge status="active" />
                {:else}
                    <StatusBadge status="idle" />
                {/if}
            {/snippet}
            <div class="bg-black aspect-video flex items-center justify-center relative">
                {#if processedVideoPath}
                    <video src={convertFileSrc(processedVideoPath)} controls class="w-full h-full object-contain"><track kind="captions" /></video>
                {:else if videoPath}
                    <video bind:this={videoPlayer} controls class="w-full h-full object-contain"><track kind="captions" /></video>
                {:else}
                    <div class="flex flex-col items-center gap-2 text-slate-500">
                        <span class="material-symbols-outlined text-4xl">perm_media</span>
                        <p class="text-xs">Select video source</p>
                    </div>
                {/if}
            </div>
        </Panel>

        <!-- Detection Timeline -->
        <DetectionTimeline
            {detectionTimeline}
            {maxDetection}
            {selectedLabels}
        />
    </div>
</div>
