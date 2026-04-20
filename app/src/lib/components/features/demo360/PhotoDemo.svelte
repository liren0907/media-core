<script lang="ts">
  import Panel from '$lib/components/ui/Panel.svelte';
  import Panoramic360Viewer from './Panoramic360Viewer.svelte';
  import ControlsOverlay from './ControlsOverlay.svelte';

  let image: HTMLImageElement | null = $state(null);
  let fileName = $state('');
  let objectUrl: string | null = null;
  let loadError = $state<string | null>(null);

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

    if (objectUrl) URL.revokeObjectURL(objectUrl);
    objectUrl = URL.createObjectURL(file);

    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
      image = img;
      resetToken++;
    };
    img.onerror = () => {
      loadError = 'Failed to decode image';
      image = null;
    };
    img.src = objectUrl;
  }

  function clearImage() {
    image = null;
    fileName = '';
    if (objectUrl) {
      URL.revokeObjectURL(objectUrl);
      objectUrl = null;
    }
  }

  $effect(() => {
    return () => {
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  });
</script>

<div class="grid grid-cols-1 lg:grid-cols-[320px_1fr] gap-3">
  <div class="flex flex-col gap-3">
    <Panel title="Source Image" icon="image">
      <div class="p-3 flex flex-col gap-2">
        <label class="flex items-center justify-center gap-2 px-3 py-2 rounded bg-[#137fec] hover:bg-[#0f6ed1] text-white text-xs font-semibold cursor-pointer transition-colors">
          <span class="material-symbols-outlined text-[16px]">upload</span>
          Choose image file
          <input type="file" accept="image/*" class="hidden" onchange={onFileChange} />
        </label>
        {#if fileName}
          <div class="flex items-center justify-between gap-2 text-xs text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-[#1f2937] px-2 py-1.5 rounded">
            <span class="truncate" title={fileName}>{fileName}</span>
            <button onclick={clearImage} class="text-slate-400 hover:text-red-500 shrink-0" aria-label="Clear">
              <span class="material-symbols-outlined text-[14px]">close</span>
            </button>
          </div>
        {/if}
        {#if loadError}
          <div class="text-xs text-red-500">{loadError}</div>
        {/if}
        <p class="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">
          Use an <span class="font-semibold">equirectangular</span> 2:1 image (e.g. 4096×2048). Drag to look around, scroll to zoom.
        </p>
      </div>
    </Panel>

    <Panel title="Controls" icon="tune">
      <div class="p-3">
        <ControlsOverlay bind:fov bind:autoRotate bind:dragSensitivity {view} onReset={() => resetToken++} />
      </div>
    </Panel>
  </div>

  <div class="min-h-[500px] lg:min-h-0 lg:h-[calc(100vh-200px)]">
    <Panoramic360Viewer
      source={image}
      bind:fov
      {autoRotate}
      {dragSensitivity}
      {resetToken}
      onView={(v) => (view = v)}
    />
  </div>
</div>
