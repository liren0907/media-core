<script lang="ts">
  import { onDestroy } from 'svelte';
  import { convertFileSrc, invoke } from '@tauri-apps/api/core';
  import { Panel, StatusBadge, EmptyState, ErrorAlert, Icon } from '$lib/components/ui';
  import LevelMeter from './LevelMeter.svelte';
  import type { RecordingInfo } from '$lib/types';

  const HISTORY_LEN = 48;

  // WKWebView (macOS) records AAC in MP4; Chromium records Opus in WebM.
  const MIME_CANDIDATES = [
    { mime: 'audio/mp4', ext: 'm4a' },
    { mime: 'audio/webm;codecs=opus', ext: 'webm' },
    { mime: 'audio/webm', ext: 'webm' },
  ];

  let isRecording = $state(false);
  let isPaused = $state(false);
  let recordingTime = $state(0);
  let level = $state(0);
  let history = $state<number[]>(Array(HISTORY_LEN).fill(0));
  let saving = $state(false);
  let error = $state('');
  let success = $state('');

  let recordings = $state<RecordingInfo[]>([]);
  let listError = $state('');
  let playingFile = $state('');
  let player = $state<HTMLAudioElement | null>(null);
  let selected = $state<string[]>([]);
  let renaming = $state('');
  let renameValue = $state('');

  const allSelected = $derived(recordings.length > 0 && selected.length === recordings.length);

  let mediaRecorder: MediaRecorder | null = null;
  let audioChunks: Blob[] = [];
  let stream: MediaStream | null = null;
  let audioContext: AudioContext | null = null;
  let analyser: AnalyserNode | null = null;
  let rafId = 0;
  let timerId: ReturnType<typeof setInterval> | undefined;
  let activeExt = 'webm';

  const recorderColors: Record<string, string> = {
    recording: 'bg-red-500/10 text-red-600 dark:text-red-400 border-red-500/20',
    paused: 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20',
    standby: 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20',
  };
  const recorderStatus = $derived(isRecording ? (isPaused ? 'paused' : 'recording') : 'standby');

  function formatTime(totalSeconds: number): string {
    const mins = Math.floor(totalSeconds / 60);
    const secs = totalSeconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  function formatSize(info: RecordingInfo): string {
    return info.sizeMb >= 1 ? `${info.sizeMb.toFixed(2)} MB` : `${info.sizeKb.toFixed(1)} KB`;
  }

  async function refreshRecordings() {
    try {
      recordings = await invoke<RecordingInfo[]>('list_audio_recordings');
      listError = '';
    } catch (e) {
      listError = String(e);
    }
  }
  refreshRecordings();

  function monitorLevel() {
    if (!analyser) return;
    const data = new Uint8Array(analyser.frequencyBinCount);
    let frame = 0;
    const tick = () => {
      if (!analyser || !isRecording) return;
      if (!isPaused) {
        analyser.getByteFrequencyData(data);
        let sum = 0;
        for (const value of data) sum += value;
        level = Math.round((sum / data.length / 255) * 100);
        frame += 1;
        if (frame % 4 === 0) history = [...history.slice(1), level];
      }
      rafId = requestAnimationFrame(tick);
    };
    rafId = requestAnimationFrame(tick);
  }

  async function startRecording() {
    error = '';
    success = '';
    if (typeof MediaRecorder === 'undefined' || !navigator.mediaDevices?.getUserMedia) {
      error = 'Audio recording is not supported in this environment.';
      return;
    }
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true },
      });
    } catch (e) {
      error = `Microphone access failed: ${e}`;
      return;
    }

    audioContext = new AudioContext();
    const source = audioContext.createMediaStreamSource(stream);
    analyser = audioContext.createAnalyser();
    analyser.fftSize = 256;
    source.connect(analyser);

    const picked = MIME_CANDIDATES.find((c) => MediaRecorder.isTypeSupported(c.mime));
    activeExt = picked?.ext ?? 'webm';
    mediaRecorder = picked ? new MediaRecorder(stream, { mimeType: picked.mime }) : new MediaRecorder(stream);
    audioChunks = [];
    mediaRecorder.ondataavailable = (event) => {
      if (event.data.size > 0) audioChunks.push(event.data);
    };
    mediaRecorder.onstop = () => {
      void saveRecording();
    };
    mediaRecorder.start(1000);

    isRecording = true;
    isPaused = false;
    recordingTime = 0;
    timerId = setInterval(() => {
      if (!isPaused) recordingTime += 1;
    }, 1000);
    monitorLevel();
  }

  function pauseRecording() {
    if (mediaRecorder?.state === 'recording') {
      mediaRecorder.pause();
      isPaused = true;
    }
  }

  function resumeRecording() {
    if (mediaRecorder?.state === 'paused') {
      mediaRecorder.resume();
      isPaused = false;
    }
  }

  function stopRecording() {
    if (!mediaRecorder || mediaRecorder.state === 'inactive') return;
    mediaRecorder.stop();
    isRecording = false;
    isPaused = false;
    cleanupCapture();
  }

  function cleanupCapture() {
    if (rafId) cancelAnimationFrame(rafId);
    rafId = 0;
    if (timerId) clearInterval(timerId);
    timerId = undefined;
    stream?.getTracks().forEach((track) => track.stop());
    stream = null;
    void audioContext?.close();
    audioContext = null;
    analyser = null;
    level = 0;
    history = Array(HISTORY_LEN).fill(0);
  }

  async function saveRecording() {
    saving = true;
    try {
      const blob = new Blob(audioChunks);
      audioChunks = [];
      const buffer = new Uint8Array(await blob.arrayBuffer());
      const stamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
      const filename = `recording_${stamp}.${activeExt}`;
      const info = await invoke<RecordingInfo>('save_audio_recording', {
        audioData: Array.from(buffer),
        filename,
      });
      success = `Saved ${info.filename} (${formatSize(info)})`;
      await refreshRecordings();
    } catch (e) {
      error = `Failed to save recording: ${e}`;
    } finally {
      saving = false;
    }
  }

  function togglePlay(rec: RecordingInfo) {
    if (!player) return;
    if (playingFile === rec.filename) {
      player.pause();
      player.currentTime = 0;
      playingFile = '';
      return;
    }
    player.src = convertFileSrc(rec.filePath);
    playingFile = rec.filename;
    player.play().catch((e) => {
      playingFile = '';
      error = `Playback failed: ${e}`;
    });
  }

  async function removeRecording(rec: RecordingInfo) {
    if (playingFile === rec.filename && player) {
      player.pause();
      playingFile = '';
    }
    try {
      await invoke('delete_audio_recording', { filename: rec.filename });
      selected = selected.filter((f) => f !== rec.filename);
      await refreshRecordings();
    } catch (e) {
      listError = String(e);
    }
  }

  function toggleSelect(filename: string) {
    selected = selected.includes(filename)
      ? selected.filter((f) => f !== filename)
      : [...selected, filename];
  }

  function toggleSelectAll() {
    selected = allSelected ? [] : recordings.map((r) => r.filename);
  }

  async function removeSelected() {
    if (playingFile && selected.includes(playingFile) && player) {
      player.pause();
      playingFile = '';
    }
    try {
      for (const filename of selected) {
        await invoke('delete_audio_recording', { filename });
      }
      listError = '';
    } catch (e) {
      listError = String(e);
    }
    selected = [];
    await refreshRecordings();
  }

  function splitName(filename: string): { stem: string; ext: string } {
    const dot = filename.lastIndexOf('.');
    return dot > 0 ? { stem: filename.slice(0, dot), ext: filename.slice(dot) } : { stem: filename, ext: '' };
  }

  function startRename(rec: RecordingInfo) {
    renaming = rec.filename;
    renameValue = splitName(rec.filename).stem;
  }

  function cancelRename() {
    renaming = '';
    renameValue = '';
  }

  async function commitRename() {
    const oldFilename = renaming;
    const stem = renameValue.trim();
    if (!stem || stem === splitName(oldFilename).stem) {
      cancelRename();
      return;
    }
    const newFilename = `${stem}${splitName(oldFilename).ext}`;
    if (playingFile === oldFilename && player) {
      player.pause();
      playingFile = '';
    }
    try {
      await invoke('rename_audio_recording', { oldFilename, newFilename });
      selected = selected.map((f) => (f === oldFilename ? newFilename : f));
      listError = '';
    } catch (e) {
      listError = String(e);
    }
    cancelRename();
    await refreshRecordings();
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  onDestroy(() => {
    if (isRecording) stopRecording();
    else cleanupCapture();
    player?.pause();
  });
</script>

<div class="grid grid-cols-1 lg:grid-cols-2 gap-4 items-start">
  <Panel title="Recorder" icon="mic">
    {#snippet actions()}
      <StatusBadge status={recorderStatus} colorMap={recorderColors} />
    {/snippet}
    <div class="p-4 flex flex-col gap-5">
      <div class="text-center">
        <span class="font-mono text-4xl font-bold tabular-nums text-slate-900 dark:text-white">
          {formatTime(recordingTime)}
        </span>
      </div>

      <LevelMeter {level} {history} />

      <div class="flex flex-wrap justify-center gap-2.5">
        {#if !isRecording}
          <button onclick={startRecording} class="flex items-center gap-1.5 px-4 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-xs font-bold transition-colors">
            <Icon name="mic" class="text-[14px]" /> Start Recording
          </button>
        {:else}
          {#if !isPaused}
            <button onclick={pauseRecording} class="flex items-center gap-1.5 px-4 py-2 bg-amber-500 hover:bg-amber-600 text-white rounded text-xs font-bold transition-colors">
              <Icon name="pause" class="text-[14px]" /> Pause
            </button>
          {:else}
            <button onclick={resumeRecording} class="flex items-center gap-1.5 px-4 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-xs font-bold transition-colors">
              <Icon name="play_arrow" class="text-[14px]" /> Resume
            </button>
          {/if}
          <button onclick={stopRecording} class="flex items-center gap-1.5 px-4 py-2 bg-red-500 hover:bg-red-600 text-white rounded text-xs font-bold transition-colors">
            <Icon name="stop" class="text-[14px]" /> Stop & Save
          </button>
        {/if}
      </div>

      {#if error}
        <ErrorAlert message={error} />
      {/if}
      {#if saving}
        <p class="text-meta text-center">Saving recording…</p>
      {:else if success}
        <p class="text-caption text-status-success text-center">{success}</p>
      {/if}
    </div>
  </Panel>

  <Panel title="Recordings" icon="audio_file">
    {#snippet actions()}
      <div class="flex items-center gap-3">
        {#if selected.length > 0}
          <button onclick={removeSelected} class="flex items-center gap-1 text-stat-label text-red-500 hover:text-red-400 transition-colors">
            <Icon name="delete" class="text-[14px]" /> Delete ({selected.length})
          </button>
        {/if}
        <button onclick={refreshRecordings} class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
          <Icon name="sync" class="text-[14px]" /> Refresh
        </button>
      </div>
    {/snippet}
    {#if listError}
      <div class="p-3">
        <ErrorAlert message={listError} />
      </div>
    {:else if recordings.length === 0}
      <EmptyState icon="mic" message="No recordings yet — hit Start Recording" />
    {:else}
      <div class="flex items-center gap-3 px-4 py-2 border-b border-slate-200 dark:border-[#2a3441]">
        <input
          type="checkbox"
          checked={allSelected}
          onchange={toggleSelectAll}
          aria-label="Select all recordings"
          class="accent-[#137fec]"
        />
        <span class="text-meta">
          {selected.length > 0 ? `${selected.length} selected` : `${recordings.length} recording${recordings.length === 1 ? '' : 's'}`}
        </span>
      </div>
      <ul class="divide-y divide-slate-200 dark:divide-[#2a3441]">
        {#each recordings as rec (rec.filename)}
          <li class="flex items-center gap-3 px-4 py-2.5">
            <input
              type="checkbox"
              checked={selected.includes(rec.filename)}
              onchange={() => toggleSelect(rec.filename)}
              aria-label="Select {rec.filename}"
              class="accent-[#137fec]"
            />
            <button
              onclick={() => togglePlay(rec)}
              title={playingFile === rec.filename ? 'Stop playback' : 'Play'}
              class="text-[#137fec] hover:text-blue-400 transition-colors"
            >
              <Icon name={playingFile === rec.filename ? 'stop' : 'play_arrow'} class="text-[18px]" />
            </button>
            <div class="flex-1 min-w-0">
              {#if renaming === rec.filename}
                <div class="flex items-center gap-1.5">
                  <input
                    use:focusOnMount
                    bind:value={renameValue}
                    onkeydown={(e) => {
                      if (e.key === 'Enter') void commitRename();
                      else if (e.key === 'Escape') cancelRename();
                    }}
                    aria-label="New recording name"
                    class="flex-1 min-w-0 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-2 py-1 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]"
                  />
                  <span class="text-meta">{splitName(rec.filename).ext}</span>
                  <button onclick={commitRename} title="Save name" class="text-status-success hover:text-green-400 transition-colors">
                    <Icon name="check" class="text-[16px]" />
                  </button>
                  <button onclick={cancelRename} title="Cancel rename" class="text-slate-400 hover:text-slate-500 transition-colors">
                    <Icon name="close" class="text-[16px]" />
                  </button>
                </div>
              {:else}
                <p class="text-body truncate">{rec.filename}</p>
                <p class="text-meta">{formatSize(rec)} · {rec.createdDate}</p>
              {/if}
            </div>
            {#if renaming !== rec.filename}
              <button
                onclick={() => startRename(rec)}
                title="Rename recording"
                class="text-slate-400 hover:text-[#137fec] transition-colors"
              >
                <Icon name="draw" class="text-[16px]" />
              </button>
              <button
                onclick={() => removeRecording(rec)}
                title="Delete recording"
                class="text-slate-400 hover:text-red-500 transition-colors"
              >
                <Icon name="delete" class="text-[16px]" />
              </button>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </Panel>
</div>

<audio bind:this={player} onended={() => (playingFile = '')} class="hidden"></audio>
