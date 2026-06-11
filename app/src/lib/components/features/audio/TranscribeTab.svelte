<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { Panel, RunButton, ProgressBar, ErrorAlert, EmptyState, FormField, StatusBadge, Icon } from '$lib/components/ui';
  import { FilePicker } from '$lib/components/form';
  import { onAsrProgress, onAsrModelProgress } from '$lib/events';
  import { saveFile } from '$lib/utils/file-dialog';
  import { inputClass } from '$lib/utils/styles';
  import type { AsrModelInfo, RecordingInfo, TranscriptionResult } from '$lib/types';

  // --- Source ---
  let audioPath = $state('');
  let recordings = $state<RecordingInfo[]>([]);
  // Formats symphonia decodes with our feature set (no Opus — .webm recordings
  // from Chromium-based webviews won't transcribe; macOS records .m4a)
  const audioFilters = [{ name: 'Audio', extensions: ['wav', 'mp3', 'flac', 'm4a', 'aac', 'mp4', 'ogg'] }];

  // --- Model & language ---
  let models = $state<AsrModelInfo[]>([]);
  let selectedModel = $state('base');
  let language = $state('auto');
  let setupError = $state('');
  const languages = [
    { id: 'auto', label: 'Auto-detect' },
    { id: 'en', label: 'English' },
    { id: 'zh', label: '中文' },
    { id: 'ja', label: '日本語' },
    { id: 'ko', label: '한국어' },
    { id: 'es', label: 'Español' },
    { id: 'fr', label: 'Français' },
    { id: 'de', label: 'Deutsch' },
  ];
  const currentModel = $derived(models.find((m) => m.name === selectedModel));

  // --- Download state ---
  let downloading = $state(false);
  let downloadPercent = $state(0);
  let downloadedMb = $state(0);

  // --- Transcription state ---
  let running = $state(false);
  let progress = $state(0);
  let liveSegments = $state<{ start: number; end: number; text: string }[]>([]);
  let result = $state<TranscriptionResult | null>(null);
  let error = $state('');
  let resultView = $state<'text' | 'srt' | 'json'>('text');
  const resultViews: ('text' | 'srt' | 'json')[] = ['text', 'srt', 'json'];

  const modelColors: Record<string, string> = {
    ready: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
    'not downloaded': 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20',
  };

  let unlisteners: UnlistenFn[] = [];

  onMount(() => {
    void (async () => {
      try {
        unlisteners.push(
          await onAsrProgress((e) => {
            if (e.kind === 'progress' && e.progress != null) {
              progress = e.progress;
            } else if (e.kind === 'segment' && e.text != null) {
              liveSegments = [...liveSegments, { start: e.startSeconds ?? 0, end: e.endSeconds ?? 0, text: e.text }];
            } else if (e.kind === 'complete') {
              progress = 100;
            }
          })
        );
        unlisteners.push(
          await onAsrModelProgress((e) => {
            downloadedMb = e.downloadedBytes / (1024 * 1024);
            if (e.percent != null) downloadPercent = e.percent;
          })
        );
      } catch (e) {
        // Event listeners need the Tauri runtime; in a plain browser the tab is display-only.
        console.warn('ASR event listeners unavailable:', e);
      }
      await Promise.all([refreshModels(), refreshRecordings()]);
    })();
  });
  onDestroy(() => unlisteners.forEach((unlisten) => unlisten()));

  async function refreshModels() {
    try {
      models = await invoke<AsrModelInfo[]>('list_asr_models');
      setupError = '';
    } catch (e) {
      setupError = String(e);
    }
  }

  async function refreshRecordings() {
    try {
      recordings = await invoke<RecordingInfo[]>('list_audio_recordings');
    } catch {
      recordings = [];
    }
  }

  function handleRecordingSelect(e: Event) {
    const value = (e.currentTarget as HTMLSelectElement).value;
    if (value) audioPath = value;
  }

  async function downloadModel() {
    downloading = true;
    downloadPercent = 0;
    downloadedMb = 0;
    error = '';
    try {
      await invoke('download_asr_model', { model: selectedModel });
      await refreshModels();
    } catch (e) {
      error = String(e);
    } finally {
      downloading = false;
    }
  }

  async function transcribe() {
    running = true;
    error = '';
    result = null;
    liveSegments = [];
    progress = 0;
    try {
      result = await invoke<TranscriptionResult>('transcribe_audio', {
        audioPath,
        model: selectedModel,
        language: language === 'auto' ? null : language,
      });
      resultView = 'text';
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  async function cancel() {
    try {
      await invoke('cancel_asr');
    } catch (e) {
      error = String(e);
    }
  }

  const resultText = $derived(
    result == null
      ? ''
      : resultView === 'text'
        ? result.text
        : resultView === 'srt'
          ? result.srt
          : JSON.stringify(result.json, null, 2)
  );

  async function exportResult(view: 'text' | 'srt' | 'json') {
    if (!result) return;
    const spec = {
      text: { ext: 'txt', name: 'Text', contents: result.text },
      srt: { ext: 'srt', name: 'SubRip', contents: result.srt },
      json: { ext: 'json', name: 'JSON', contents: JSON.stringify(result.json, null, 2) },
    }[view];
    try {
      const base = audioPath.split('/').pop()?.replace(/\.[^.]+$/, '') || 'transcript';
      const path = await saveFile([{ name: spec.name, extensions: [spec.ext] }], `${base}.${spec.ext}`);
      if (path) await invoke('save_text_file', { path, contents: spec.contents });
    } catch (e) {
      error = String(e);
    }
  }

  function formatClock(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }
</script>

<div class="flex flex-col gap-4">
  {#if setupError}
    <ErrorAlert message={setupError} />
  {/if}

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 items-start">
    <Panel title="Source" icon="audio_file">
      <div class="p-4 flex flex-col gap-4">
        <FilePicker bind:value={audioPath} label="Audio File" filters={audioFilters} />
        {#if recordings.length > 0}
          <FormField label="Or use a recording" id="asr-recording">
            <select id="asr-recording" class="{inputClass} w-full" onchange={handleRecordingSelect}>
              <option value="">Select a recording…</option>
              {#each recordings as rec (rec.filename)}
                <option value={rec.filePath}>{rec.filename}</option>
              {/each}
            </select>
          </FormField>
        {/if}
      </div>
    </Panel>

    <Panel title="Model" icon="tune">
      {#snippet actions()}
        {#if currentModel}
          <StatusBadge status={currentModel.downloaded ? 'ready' : 'not downloaded'} colorMap={modelColors} />
        {/if}
      {/snippet}
      <div class="p-4 flex flex-col gap-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <FormField label="Whisper Model" id="asr-model">
            <select id="asr-model" bind:value={selectedModel} class="{inputClass} w-full">
              {#each models as m (m.name)}
                <option value={m.name}>{m.name} ({m.approxMb} MB){m.downloaded ? ' ✓' : ''}</option>
              {/each}
            </select>
          </FormField>
          <FormField label="Language" id="asr-language">
            <select id="asr-language" bind:value={language} class="{inputClass} w-full">
              {#each languages as lang (lang.id)}
                <option value={lang.id}>{lang.label}</option>
              {/each}
            </select>
          </FormField>
        </div>
        {#if currentModel && !currentModel.downloaded}
          <div class="flex flex-col gap-2">
            <div class="flex items-center gap-3">
              <RunButton loading={downloading} label="Download" onclick={downloadModel} />
              {#if downloading}
                <button onclick={cancel} class="text-stat-label text-red-500 hover:text-red-400 transition-colors">Cancel</button>
                <span class="text-meta tabular-nums">{downloadedMb.toFixed(0)} / ~{currentModel.approxMb} MB</span>
              {/if}
            </div>
            {#if downloading}
              <ProgressBar percent={downloadPercent} />
            {/if}
          </div>
        {/if}
      </div>
    </Panel>
  </div>

  <Panel title="Transcribe" icon="graphic_eq">
    {#snippet actions()}
      <div class="flex items-center gap-3">
        {#if running}
          <button onclick={cancel} class="text-stat-label text-red-500 hover:text-red-400 transition-colors">Cancel</button>
        {/if}
        <RunButton loading={running} disabled={!audioPath || !currentModel?.downloaded} label="Transcribe" onclick={transcribe} />
      </div>
    {/snippet}
    {#if running}
      <div class="p-4 flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <span class="text-meta">Transcribing — live segments stream in below</span>
          <span class="text-meta tabular-nums">{progress}%</span>
        </div>
        <ProgressBar percent={progress} />
        {#if liveSegments.length > 0}
          <div class="max-h-44 overflow-y-auto flex flex-col gap-1 rounded border border-slate-200 dark:border-[#2a3441] bg-slate-50 dark:bg-[#0d1117] p-3">
            {#each liveSegments as seg, i (i)}
              <p class="text-caption">
                <span class="text-code text-slate-400">[{formatClock(seg.start)} → {formatClock(seg.end)}]</span>
                {seg.text}
              </p>
            {/each}
          </div>
        {/if}
      </div>
    {:else if error}
      <div class="p-4">
        <ErrorAlert message={error} />
      </div>
    {:else if result}
      <div class="p-4 flex flex-col gap-3">
        <div class="flex items-center justify-between flex-wrap gap-2">
          <div class="flex gap-1.5">
            {#each resultViews as view (view)}
              <button
                onclick={() => (resultView = view)}
                class="px-2.5 py-1 rounded text-[10px] font-bold uppercase transition-colors {resultView === view
                  ? 'bg-[#137fec] text-white'
                  : 'bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#1f2937]'}"
              >
                {view}
              </button>
            {/each}
          </div>
          <div class="flex gap-3">
            <button onclick={() => exportResult('text')} class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
              <Icon name="save" class="text-[14px]" /> TXT
            </button>
            <button onclick={() => exportResult('srt')} class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
              <Icon name="save" class="text-[14px]" /> SRT
            </button>
            <button onclick={() => exportResult('json')} class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
              <Icon name="save" class="text-[14px]" /> JSON
            </button>
          </div>
        </div>
        <pre class="text-code whitespace-pre-wrap max-h-96 overflow-y-auto rounded border border-slate-200 dark:border-[#2a3441] bg-slate-50 dark:bg-[#0d1117] p-3">{resultText}</pre>
      </div>
    {:else}
      <EmptyState icon="graphic_eq" message="Pick an audio file and a downloaded model, then hit Transcribe" />
    {/if}
  </Panel>
</div>
