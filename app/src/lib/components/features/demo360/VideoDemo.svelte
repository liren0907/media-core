<script lang="ts">
  import Panel from '$lib/components/ui/Panel.svelte';
  import Panoramic360Viewer from './Panoramic360Viewer.svelte';
  import ControlsOverlay from './ControlsOverlay.svelte';

  let video: HTMLVideoElement | null = $state(null);
  let fileName = $state('');
  let objectUrl: string | null = null;
  let loadError = $state<string | null>(null);

  let isPlaying = $state(false);
  let currentTime = $state(0);
  let duration = $state(0);
  let volume = $state(1);
  let muted = $state(true);

  let fov = $state(75);
  let autoRotate = $state(false);
  let dragSensitivity = $state(0.6);
  let resetToken = $state(0);
  let view = $state({ lon: 0, lat: 0, fov: 75 });

  function onFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    loadError = null;
    fileName = file.name;

    clearVideo();
    objectUrl = URL.createObjectURL(file);

    const v = document.createElement('video');
    v.crossOrigin = 'anonymous';
    v.loop = true;
    v.muted = muted;
    v.volume = volume;
    v.playsInline = true;
    v.preload = 'auto';
    v.src = objectUrl;

    v.onloadedmetadata = () => {
      duration = v.duration || 0;
      video = v;
      resetToken++;
      v.play().then(() => (isPlaying = true)).catch(() => {
        isPlaying = false;
      });
    };
    v.ontimeupdate = () => {
      currentTime = v.currentTime;
    };
    v.onplay = () => (isPlaying = true);
    v.onpause = () => (isPlaying = false);
    v.onerror = () => {
      loadError = 'Failed to load video (codec may be unsupported)';
      clearVideo();
    };
  }

  function clearVideo() {
    if (video) {
      video.pause();
      video.removeAttribute('src');
      video.load();
    }
    video = null;
    isPlaying = false;
    currentTime = 0;
    duration = 0;
    if (objectUrl) {
      URL.revokeObjectURL(objectUrl);
      objectUrl = null;
    }
  }

  function togglePlay() {
    if (!video) return;
    if (video.paused) video.play();
    else video.pause();
  }

  function onSeek(e: Event) {
    if (!video) return;
    const input = e.target as HTMLInputElement;
    video.currentTime = Number(input.value);
  }

  function onVolumeChange(e: Event) {
    if (!video) return;
    const input = e.target as HTMLInputElement;
    volume = Number(input.value);
    video.volume = volume;
    if (volume > 0 && muted) {
      muted = false;
      video.muted = false;
    }
  }

  function toggleMute() {
    if (!video) return;
    muted = !muted;
    video.muted = muted;
  }

  function fmtTime(s: number) {
    if (!isFinite(s)) return '0:00';
    const m = Math.floor(s / 60);
    const sec = Math.floor(s % 60).toString().padStart(2, '0');
    return `${m}:${sec}`;
  }

  $effect(() => {
    return () => {
      clearVideo();
    };
  });
</script>

<div class="grid grid-cols-1 lg:grid-cols-[320px_1fr] gap-3">
  <div class="flex flex-col gap-3">
    <Panel title="Source Video" icon="movie">
      <div class="p-3 flex flex-col gap-2">
        <label class="flex items-center justify-center gap-2 px-3 py-2 rounded bg-[#137fec] hover:bg-[#0f6ed1] text-white text-xs font-semibold cursor-pointer transition-colors">
          <span class="material-symbols-outlined text-[16px]">upload</span>
          Choose video file
          <input type="file" accept="video/*" class="hidden" onchange={onFileChange} />
        </label>
        {#if fileName}
          <div class="flex items-center justify-between gap-2 text-body bg-slate-50 dark:bg-[#1f2937] px-2 py-1.5 rounded">
            <span class="truncate" title={fileName}>{fileName}</span>
            <button onclick={clearVideo} class="text-muted hover:text-red-500 shrink-0" aria-label="Clear">
              <span class="material-symbols-outlined text-[14px]">close</span>
            </button>
          </div>
        {/if}
        {#if loadError}
          <div class="text-body text-status-error">{loadError}</div>
        {/if}
        <p class="text-caption leading-relaxed">
          Use an <span class="font-semibold">equirectangular</span> 2:1 MP4 (H.264) 360° video. Autoplay starts muted — unmute via the volume slider.
        </p>
      </div>
    </Panel>

    {#if video}
      <Panel title="Playback" icon="play_circle">
        <div class="p-3 flex flex-col gap-2">
          <div class="flex items-center gap-2">
            <button
              onclick={togglePlay}
              class="flex items-center justify-center size-8 rounded bg-[#137fec] hover:bg-[#0f6ed1] text-white transition-colors"
              aria-label={isPlaying ? 'Pause' : 'Play'}
            >
              <span class="material-symbols-outlined text-[18px]">{isPlaying ? 'pause' : 'play_arrow'}</span>
            </button>
            <span class="text-caption tabular-nums">{fmtTime(currentTime)} / {fmtTime(duration)}</span>
          </div>

          <input
            type="range"
            min="0"
            max={duration || 0}
            step="0.1"
            value={currentTime}
            onchange={onSeek}
            oninput={onSeek}
            class="w-full accent-[#137fec]"
            aria-label="Seek"
          />

          <div class="flex items-center gap-2">
            <button onclick={toggleMute} class="text-muted hover:text-slate-700 dark:hover:text-slate-200" aria-label={muted ? 'Unmute' : 'Mute'}>
              <span class="material-symbols-outlined text-[18px]">{muted || volume === 0 ? 'volume_off' : 'volume_up'}</span>
            </button>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={muted ? 0 : volume}
              oninput={onVolumeChange}
              class="flex-1 accent-[#137fec]"
              aria-label="Volume"
            />
          </div>
        </div>
      </Panel>
    {/if}

    <Panel title="Controls" icon="tune">
      <div class="p-3">
        <ControlsOverlay bind:fov bind:autoRotate bind:dragSensitivity {view} onReset={() => resetToken++} />
      </div>
    </Panel>
  </div>

  <div class="min-h-[500px] lg:min-h-0 lg:h-[calc(100vh-200px)]">
    <Panoramic360Viewer
      source={video}
      bind:fov
      {autoRotate}
      {dragSensitivity}
      {resetToken}
      onView={(v) => (view = v)}
    />
  </div>
</div>
