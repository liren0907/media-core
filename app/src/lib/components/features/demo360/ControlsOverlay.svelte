<script lang="ts">
  import ToggleSwitch from '$lib/components/ui/ToggleSwitch.svelte';

  interface Props {
    fov: number;
    autoRotate: boolean;
    dragSensitivity: number;
    view: { lon: number; lat: number; fov: number };
    onReset: () => void;
  }

  let {
    fov = $bindable(),
    autoRotate = $bindable(),
    dragSensitivity = $bindable(),
    view,
    onReset,
  }: Props = $props();
</script>

<div class="flex flex-col gap-3">
  <div>
    <div class="flex items-center justify-between mb-1">
      <label class="text-xs font-semibold text-slate-600 dark:text-slate-300" for="fov-slider">Field of View</label>
      <span class="text-xs text-slate-500 tabular-nums">{fov.toFixed(0)}°</span>
    </div>
    <input
      id="fov-slider"
      type="range"
      min="30"
      max="90"
      step="1"
      bind:value={fov}
      class="w-full accent-[#137fec]"
    />
  </div>

  <div>
    <div class="flex items-center justify-between mb-1">
      <label class="text-xs font-semibold text-slate-600 dark:text-slate-300" for="drag-sens-slider">Drag sensitivity</label>
      <span class="text-xs text-slate-500 tabular-nums">{dragSensitivity.toFixed(2)}×</span>
    </div>
    <input
      id="drag-sens-slider"
      type="range"
      min="0.1"
      max="2"
      step="0.05"
      bind:value={dragSensitivity}
      class="w-full accent-[#137fec]"
    />
    <div class="flex justify-between text-[10px] text-slate-400 mt-0.5">
      <span>slow</span><span>natural</span><span>fast</span>
    </div>
  </div>

  <ToggleSwitch bind:checked={autoRotate} label="Auto-rotate" />

  <button
    onclick={onReset}
    class="flex items-center justify-center gap-1 px-3 py-1.5 rounded bg-slate-100 dark:bg-[#283039] text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors text-xs font-semibold"
  >
    <span class="material-symbols-outlined text-[14px]">restart_alt</span>
    Reset view
  </button>

  <div class="border-t border-slate-200 dark:border-[#2a3441] pt-2 grid grid-cols-3 gap-1 text-[10px] tabular-nums text-slate-500 dark:text-slate-400">
    <div><span class="text-slate-400">lon</span> {view.lon.toFixed(1)}°</div>
    <div><span class="text-slate-400">lat</span> {view.lat.toFixed(1)}°</div>
    <div><span class="text-slate-400">fov</span> {view.fov.toFixed(0)}°</div>
  </div>
</div>
