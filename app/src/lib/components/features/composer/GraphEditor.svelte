<script lang="ts">
  /**
   * The graph editor: toolbar, canvas, inspector, status bar.
   *
   * This file holds SCREEN STATE only — which picker is open, what is selected, pan
   * and zoom, panel widths. Every rule about the graph itself lives in
   * `graph-controller.svelte.ts`, and whether a graph can run is decided by the Rust
   * validator, never re-derived here.
   */
  import { onDestroy, onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { Panel, ErrorAlert, EmptyState, RunButton, Icon, StatusBadge } from '$lib/components/ui';
  import { inputClass } from '$lib/utils/styles';
  import { formatFileSize } from '$lib/utils/format';
  import type { GraphProgressEvent } from '$lib/types';
  import { GraphController } from './graph-controller.svelte';
  import { layoutGraph, NODE_W } from './graph/layout';
  import { optionsAt, type InsertPoint } from './graph/node-options';
  import NodeCard from './graph/NodeCard.svelte';
  import NodeInspector from './graph/NodeInspector.svelte';

  const graph = new GraphController();

  // ---------------------------------------------------------- screen state
  let picker = $state<{ point: InsertPoint; gx: number; gy: number } | null>(null);
  let selected = $state('');
  let pan = $state({ x: 40, y: 40 });
  let zoom = $state(1);
  let sideWidth = $state(280);
  let canvasEl = $state<HTMLDivElement | null>(null);
  let containerEl = $state<HTMLDivElement | null>(null);

  const MIN_ZOOM = 0.5;
  const MAX_ZOOM = 1.5;
  const CANVAS_MIN_W = 340;
  const SIDE_MIN = 220;

  // Clamped in a derived value rather than written back, so a narrow window gives way
  // temporarily and the preferred width returns when there is room again.
  let containerWidth = $state(1200);
  const sideW = $derived(
    Math.max(SIDE_MIN, Math.min(sideWidth, Math.max(SIDE_MIN, containerWidth - CANVAS_MIN_W)))
  );

  const layout = $derived(
    layoutGraph(
      graph.nodes.map((n) => ({ id: n.id, hasSummary: graph.hasSummary(n.id) })),
      graph.edges
    )
  );

  const verdict = $derived(graph.verdict);
  const canRun = $derived(graph.nodes.length > 0 && verdict?.valid === true && !graph.running);

  // ------------------------------------------------------------- lifecycle
  let unlistenProgress: UnlistenFn | null = null;

  onMount(() => {
    graph.loadMethods();
  });

  $effect(() => {
    let cancelled = false;
    // Guarded on purpose: outside the Tauri shell `listen` throws synchronously, and
    // an unhandled throw in an effect takes the whole page down with it. Losing live
    // progress is a much smaller failure than a blank editor.
    try {
      listen<GraphProgressEvent>('graph:progress', (event) => {
        if (cancelled) return;
        // execute_graph mints the run id internally and only returns it at the end,
        // so the first progress event is how the UI learns what to cancel.
        graph.runId = event.payload.runId;
        graph.activeNodeId = event.payload.phase === 'start' ? event.payload.nodeId : '';
        graph.progress = event.payload.progressPercent;
      })
        .then((fn) => {
          if (cancelled) fn();
          else unlistenProgress = fn;
        })
        .catch(() => {});
    } catch {
      /* no live progress */
    }
    return () => {
      cancelled = true;
      unlistenProgress?.();
      unlistenProgress = null;
    };
  });

  // Revalidate on content change, not on every keystroke of unrelated screen state.
  let validateTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    const fingerprint = graph.snapshot;
    if (validateTimer) clearTimeout(validateTimer);
    validateTimer = setTimeout(() => {
      void fingerprint;
      graph.revalidate();
    }, 200);
    return () => {
      if (validateTimer) clearTimeout(validateTimer);
    };
  });

  // Wheel must be non-passive to stop the page scrolling underneath the canvas.
  $effect(() => {
    const el = canvasEl;
    if (!el) return;
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  });

  $effect(() => {
    const el = containerEl;
    if (!el || typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(() => {
      containerWidth = el.clientWidth;
    });
    observer.observe(el);
    containerWidth = el.clientWidth;
    return () => observer.disconnect();
  });

  $effect(() => {
    window.addEventListener('keydown', onKeydown);
    return () => window.removeEventListener('keydown', onKeydown);
  });

  onDestroy(() => {
    if (validateTimer) clearTimeout(validateTimer);
  });

  // ------------------------------------------------------------------ view
  function onWheel(event: WheelEvent) {
    // Decide before preventing: a wheel event coming from the picker popup must be
    // left alone, or the menu cannot scroll and the whole graph slides instead.
    if ((event.target as HTMLElement)?.closest?.('.pick-pop')) return;
    event.preventDefault();

    if (event.metaKey || event.ctrlKey) {
      const rect = canvasEl?.getBoundingClientRect();
      if (!rect) return;
      const px = event.clientX - rect.left;
      const py = event.clientY - rect.top;
      const next = clamp(zoom * Math.exp(-event.deltaY * 0.002), MIN_ZOOM, MAX_ZOOM);
      // Keep the point under the cursor fixed.
      pan = {
        x: px - ((px - pan.x) / zoom) * next,
        y: py - ((py - pan.y) / zoom) * next,
      };
      zoom = next;
    } else {
      pan = { x: pan.x - event.deltaX, y: pan.y - event.deltaY };
    }
  }

  let dragging = $state(false);
  let dragOrigin = { x: 0, y: 0, panX: 0, panY: 0 };

  function onCanvasPointerDown(event: PointerEvent) {
    // Panning only starts on empty space; anything clickable owns its own gesture.
    const target = event.target as HTMLElement;
    if (target.closest('button') || target.closest('.pick-pop')) return;
    dragging = true;
    dragOrigin = { x: event.clientX, y: event.clientY, panX: pan.x, panY: pan.y };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    picker = null;
  }

  function onCanvasPointerMove(event: PointerEvent) {
    if (!dragging) return;
    pan = {
      x: dragOrigin.panX + (event.clientX - dragOrigin.x),
      y: dragOrigin.panY + (event.clientY - dragOrigin.y),
    };
  }

  function onCanvasPointerUp(event: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  }

  function onKeydown(event: KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (target?.matches?.('input, select, textarea')) return;
    if (event.key === 'Escape') {
      picker = null;
      return;
    }
    if (!(event.metaKey || event.ctrlKey)) return;

    // Deliberately shadows the browser zoom: on a desktop canvas these keys mean the
    // canvas.
    if (event.key === '0') {
      event.preventDefault();
      resetView();
    } else if (event.key === '9') {
      event.preventDefault();
      fitView();
    } else if (event.key === '=' || event.key === '+') {
      event.preventDefault();
      zoom = clamp(zoom * 1.15, MIN_ZOOM, MAX_ZOOM);
    } else if (event.key === '-') {
      event.preventDefault();
      zoom = clamp(zoom / 1.15, MIN_ZOOM, MAX_ZOOM);
    }
  }

  function resetView() {
    zoom = 1;
    pan = { x: 40, y: 40 };
  }

  function fitView() {
    const rect = canvasEl?.getBoundingClientRect();
    if (!rect || layout.width === 0 || layout.height === 0) return resetView();
    const next = clamp(
      Math.min(rect.width / layout.width, rect.height / layout.height) * 0.9,
      MIN_ZOOM,
      MAX_ZOOM
    );
    zoom = next;
    pan = {
      x: (rect.width - layout.width * next) / 2,
      y: (rect.height - layout.height * next) / 2,
    };
  }

  function clamp(value: number, min: number, max: number) {
    return Math.max(min, Math.min(max, value));
  }

  // ---------------------------------------------------------------- growth
  function openPicker(point: InsertPoint, gx: number, gy: number) {
    picker = { point, gx, gy };
  }

  function place(ref: string) {
    if (!picker) return;
    const id = graph.insertAt(picker.point, ref);
    picker = null;
    selected = id;
  }

  function removeNode(nodeId: string) {
    graph.removeNode(nodeId);
    if (selected === nodeId) selected = '';
    picker = null;
  }

  function newGraph() {
    graph.clear();
    selected = '';
    picker = null;
    resetView();
  }

  const pickerSections = $derived(
    picker
      ? optionsAt(picker.point, {
          nodes: graph.nodes.map((n) => ({ id: n.id, ref: n.ref })),
          edges: graph.edges,
          methods: graph.methods,
        })
      : []
  );

  /** Growth handles only appear on free ends; a busy side is served by the on-edge `+`. */
  function handlesFor(nodeId: string) {
    const meta = graph.metaOfNode(nodeId);
    const parents = graph.parentsOf(nodeId).length;
    const children = graph.childrenOf(nodeId).length;
    return {
      after: children === 0 && meta?.kind !== 'sink',
      before: parents === 0 && meta?.kind !== 'source',
      below: parents > 0,
    };
  }

  // ------------------------------------------------------------------- run
  async function run() {
    selected = selected || '';
    await graph.run();
  }

  const resultRows = $derived.by(() => {
    const result = graph.result;
    if (!result) return [] as { label: string; value: string }[];
    const rows: { label: string; value: string }[] = [];
    if (result.metadata) rows.push({ label: 'Metadata', value: 'extracted' });
    if (result.analysis) {
      const a = result.analysis;
      rows.push({
        label: 'Analysis',
        value: `${a.motionEvents.length} motion · ${a.similarityGroups.length} groups${a.imageComparison ? ' · 1 comparison' : ''}`,
      });
    }
    if (result.extractedFrameCount > 0) {
      rows.push({ label: 'Frames (memory)', value: `${result.extractedFrameCount}` });
    }
    if (result.videoProcessResult) {
      const v = result.videoProcessResult;
      rows.push({ label: 'Frames (disk)', value: `${v.framesExtracted} in ${v.outputDir}` });
    }
    if (result.hlsResult) {
      const h = result.hlsResult;
      rows.push({ label: 'HLS', value: `${h.segmentCount} segments · ${h.playlistPath}` });
    }
    if (result.annotationResult) {
      rows.push({ label: 'Annotation', value: result.annotationResult.outputPath });
    }
    if (result.processResult) {
      const p = result.processResult;
      rows.push({
        label: 'Processed',
        value: `${p.filesProcessed} ok · ${p.filesFailed} failed · ${formatFileSize(p.totalSizeBytes)}`,
      });
    }
    return rows;
  });
</script>

<div bind:this={containerEl} class="flex flex-col gap-3">
  <!-- Toolbar -->
  <div
    class="flex items-center gap-2 px-3 py-2 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27]"
  >
    <Icon name="account_tree" class="text-[16px] text-slate-500" />
    <input
      bind:value={graph.label}
      placeholder="Untitled graph"
      class="{inputClass} w-56 py-1"
      aria-label="Graph name"
    />

    <div class="flex-1"></div>

    <button onclick={newGraph} class="tool-btn" title="Clear the canvas">
      <Icon name="note_add" class="text-[14px]" /> New
    </button>
    <div class="w-px h-4 bg-slate-200 dark:bg-[#2a3441]"></div>
    <button onclick={() => (zoom = clamp(zoom / 1.15, MIN_ZOOM, MAX_ZOOM))} class="tool-btn" title="Zoom out (⌘−)">
      <Icon name="zoom_out" class="text-[14px]" />
    </button>
    <span class="text-[10px] font-mono text-slate-500 w-9 text-center">{Math.round(zoom * 100)}%</span>
    <button onclick={() => (zoom = clamp(zoom * 1.15, MIN_ZOOM, MAX_ZOOM))} class="tool-btn" title="Zoom in (⌘+)">
      <Icon name="zoom_in" class="text-[14px]" />
    </button>
    <button onclick={fitView} class="tool-btn" title="Fit to view (⌘9)">
      <Icon name="center_focus_strong" class="text-[14px]" />
    </button>
    <button onclick={resetView} class="tool-btn" title="Reset view (⌘0)">
      <Icon name="restart_alt" class="text-[14px]" />
    </button>
    <div class="w-px h-4 bg-slate-200 dark:bg-[#2a3441]"></div>

    {#if graph.running}
      <button
        onclick={() => graph.cancel()}
        class="flex items-center gap-1 px-2 py-1 bg-red-500 hover:bg-red-600 text-white rounded text-[10px] font-bold transition-colors"
      >
        <Icon name="stop" class="text-[14px]" /> Cancel
      </button>
    {:else}
      <RunButton loading={false} disabled={!canRun} onclick={run} />
    {/if}
  </div>

  {#if graph.loadError}
    <ErrorAlert message={graph.loadError} />
  {/if}

  <!-- Canvas + inspector.
       Sized against the viewport rather than stretched to fill: the page's scroll
       container is not a flex parent, so a `flex-1` here would collapse to content
       height. The page scrolls normally once a result panel appears below. -->
  <div class="min-h-[60vh] flex gap-3">
    <div
      class="relative flex-1 min-w-0 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-slate-50 dark:bg-[#111418] overflow-hidden"
    >
      <div
        bind:this={canvasEl}
        role="presentation"
        class="ed-canvas absolute inset-0 overflow-hidden {dragging ? 'cursor-grabbing' : 'cursor-grab'}"
        onpointerdown={onCanvasPointerDown}
        onpointermove={onCanvasPointerMove}
        onpointerup={onCanvasPointerUp}
        onpointercancel={onCanvasPointerUp}
      >
        {#if graph.nodes.length === 0}
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="flex flex-col items-center gap-3">
              <EmptyState icon="account_tree" message="Start by adding a media source" />
              <button
                onclick={() => openPicker({ kind: 'root' }, 60, 60)}
                class="flex items-center gap-1 px-3 py-1.5 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[11px] font-bold whitespace-nowrap transition-colors"
              >
                <Icon name="add" class="text-[14px]" /> Add source
              </button>
            </div>
          </div>
        {/if}

        <!-- Transformed coordinate plane: cards and connections share it, so they can
             never drift apart. -->
        <div
          class="ed-pan absolute top-0 left-0 origin-top-left"
          style="transform: translate({pan.x}px, {pan.y}px) scale({zoom});"
        >
          <svg
            width={layout.width}
            height={layout.height}
            class="absolute top-0 left-0 pointer-events-none overflow-visible"
          >
            {#each layout.edges as edge (edge.from + '→' + edge.to)}
              <path
                d={edge.path}
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                class="text-slate-300 dark:text-slate-600"
              />
            {/each}
          </svg>

          {#each layout.boxes as box (box.id)}
            {@const meta = graph.metaOfNode(box.id)}
            {@const handles = handlesFor(box.id)}
            <div class="absolute" style="left: {box.x}px; top: {box.y}px;">
              <NodeCard
                label={graph.labelOfNode(box.id)}
                category={meta?.category ?? 'process'}
                summary={graph.summaryOf(box.id)}
                width={box.w}
                height={box.h}
                selected={selected === box.id}
                active={graph.activeNodeId === box.id}
                requiresFfmpeg={meta?.requiresFfmpeg ?? false}
                onselect={() => {
                  selected = box.id;
                  picker = null;
                }}
              />
            </div>

            {#if handles.before}
              <button
                class="plus-anchor absolute"
                style="left: {box.x - 26}px; top: {box.cy - 9}px;"
                title="Insert before"
                aria-label="Insert before {graph.labelOfNode(box.id)}"
                onclick={() => openPicker({ kind: 'before', nodeId: box.id }, box.x - 26, box.cy)}
              >
                <Icon name="add" class="text-[12px]" />
              </button>
            {/if}
            {#if handles.after}
              <button
                class="plus-anchor absolute"
                style="left: {box.x + box.w + 8}px; top: {box.cy - 9}px;"
                title="Insert after"
                aria-label="Insert after {graph.labelOfNode(box.id)}"
                onclick={() =>
                  openPicker({ kind: 'after', nodeId: box.id }, box.x + box.w + 8, box.cy)}
              >
                <Icon name="add" class="text-[12px]" />
              </button>
            {/if}
            {#if handles.below}
              <button
                class="plus-anchor absolute"
                style="left: {box.cx - 9}px; top: {box.y + box.h + 6}px;"
                title="Add a parallel branch"
                aria-label="Add a branch beside {graph.labelOfNode(box.id)}"
                onclick={() =>
                  openPicker({ kind: 'below', nodeId: box.id }, box.cx, box.y + box.h + 6)}
              >
                <Icon name="add" class="text-[12px]" />
              </button>
            {/if}
          {/each}

          {#each layout.edges as edge (edge.from + '/' + edge.to)}
            <button
              class="plus-anchor absolute"
              style="left: {edge.mx - 9}px; top: {edge.my - 9}px;"
              title="Insert between"
              aria-label="Insert between two nodes"
              onclick={() => openPicker({ kind: 'edge', from: edge.from, to: edge.to }, edge.mx, edge.my)}
            >
              <Icon name="add" class="text-[12px]" />
            </button>
          {/each}
        </div>

        <!-- The picker lives outside the transformed plane so it never scales. -->
        {#if picker}
          <div
            class="pick-pop absolute z-20 w-64 max-h-72 overflow-y-auto rounded-lg border border-slate-200 dark:border-[#2a3441]
                   bg-white dark:bg-[#161e27] shadow-xl"
            style="left: {Math.min(pan.x + picker.gx * zoom + 14, Math.max(0, (canvasEl?.clientWidth ?? 600) - 270))}px;
                   top: {Math.max(4, pan.y + picker.gy * zoom - 10)}px;"
          >
            {#each pickerSections as section (section.title)}
              <div class="px-2 pt-2 pb-1 text-stat-label sticky top-0 bg-white dark:bg-[#161e27]">
                {section.title}
              </div>
              {#each section.rows as row (row.meta.id)}
                <button
                  disabled={!!row.blocked}
                  onclick={() => place(row.meta.id)}
                  class="w-full text-left px-2 py-1.5 flex flex-col gap-0.5 transition-colors
                    {row.blocked
                    ? 'opacity-40 cursor-not-allowed'
                    : 'hover:bg-slate-50 dark:hover:bg-[#1f2937]'}"
                >
                  <span class="text-[11px] font-medium text-slate-900 dark:text-white">
                    {row.meta.label}
                  </span>
                  {#if row.blocked}
                    <span class="text-[10px] text-slate-500">{row.blocked}</span>
                  {:else if row.needsParam}
                    <span class="text-[10px] text-amber-600 dark:text-amber-400">
                      Needs “{row.needsParam}” filled in
                    </span>
                  {:else}
                    <span class="text-[10px] text-slate-400 truncate">{row.meta.description}</span>
                  {/if}
                </button>
              {/each}
            {/each}
            {#if pickerSections.length === 0}
              <p class="px-3 py-4 text-[11px] text-slate-500">Nothing can be placed here.</p>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <!-- Inspector -->
    <div
      style="width: {sideW}px"
      class="shrink-0 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] overflow-y-auto"
    >
      {#if selected && graph.nodes.some((n) => n.id === selected)}
        <NodeInspector {graph} selectedId={selected} onremove={removeNode} />
      {:else}
        <div class="p-4">
          <EmptyState icon="tune" message="Select a node to edit its settings" />
        </div>
      {/if}
    </div>
  </div>

  <!-- Status bar -->
  <div
    class="flex items-center gap-3 px-3 py-1.5 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] text-[11px]"
  >
    <span class="text-slate-500 font-mono">
      {graph.nodes.length} node{graph.nodes.length === 1 ? '' : 's'} ·
      {graph.edges.length} edge{graph.edges.length === 1 ? '' : 's'}
    </span>

    {#if graph.running}
      <span class="flex items-center gap-1 text-[#137fec]">
        <Icon name="progress_activity" class="text-[13px] animate-spin" />
        Running {Math.round(graph.progress)}%
      </span>
    {:else if graph.nodes.length === 0}
      <span class="text-slate-400">Empty canvas</span>
    {:else if graph.validating}
      <span class="text-slate-400">Checking…</span>
    {:else if verdict && !verdict.valid}
      <span class="flex items-center gap-1 text-red-600 dark:text-red-400">
        <Icon name="error" class="text-[13px]" />
        {verdict.reason}
      </span>
    {:else if verdict?.valid}
      <span class="flex items-center gap-1 text-emerald-600 dark:text-emerald-400">
        <Icon name="check_circle" class="text-[13px]" /> Ready to run
      </span>
    {/if}

    {#if verdict?.warnings?.length}
      <span class="flex items-center gap-1 text-amber-600 dark:text-amber-400" title={verdict.warnings.join('\n')}>
        <Icon name="warning" class="text-[13px]" />
        {verdict.warnings.length} warning{verdict.warnings.length === 1 ? '' : 's'}
      </span>
    {/if}
  </div>

  {#if verdict?.warnings?.length}
    <ul class="flex flex-col gap-1">
      {#each verdict.warnings as warning (warning)}
        <li class="text-[11px] text-amber-600 dark:text-amber-400 flex items-start gap-1.5">
          <Icon name="warning" class="text-[12px] mt-0.5 shrink-0" />
          <span>{warning}</span>
        </li>
      {/each}
    </ul>
  {/if}

  <!-- Result -->
  {#if graph.runError}
    <ErrorAlert message={graph.runError} />
  {:else if graph.result}
    <Panel title="Result" icon="analytics">
      {#snippet actions()}
        <StatusBadge status="completed" />
      {/snippet}
      <div class="p-3 flex flex-col gap-1.5">
        {#if resultRows.length === 0}
          <p class="text-xs text-slate-500">The graph ran, but produced nothing to show.</p>
        {:else}
          {#each resultRows as row (row.label)}
            <div class="flex gap-3 text-xs">
              <span class="w-32 shrink-0 text-slate-500">{row.label}</span>
              <span class="flex-1 min-w-0 font-mono text-slate-900 dark:text-white break-all">
                {row.value}
              </span>
            </div>
          {/each}
        {/if}
        {#each graph.result.warnings as warning (warning)}
          <p class="text-[11px] text-amber-600 dark:text-amber-400">{warning}</p>
        {/each}
      </div>
    </Panel>
  {/if}
</div>

<style>
  :global(.tool-btn) {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 10px;
    font-weight: 700;
    color: rgb(100 116 139);
    transition: background-color 150ms;
  }
  :global(.tool-btn:hover) {
    background-color: rgb(241 245 249);
  }
  :global(.dark .tool-btn:hover) {
    background-color: rgb(31 41 55);
  }

  .plus-anchor {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 9999px;
    border: 1px solid rgb(203 213 225);
    background: white;
    color: rgb(100 116 139);
    transition:
      background-color 150ms,
      color 150ms,
      border-color 150ms;
  }
  .plus-anchor:hover {
    background: #137fec;
    border-color: #137fec;
    color: white;
  }
  :global(.dark) .plus-anchor {
    border-color: rgb(71 85 105);
    background: #161e27;
    color: rgb(148 163 184);
  }
</style>
