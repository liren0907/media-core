<script lang="ts">
  import { GallerySection, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import NodeCard from '$lib/components/features/composer/graph/NodeCard.svelte';
  import ParamField from '$lib/components/features/composer/graph/ParamField.svelte';
  import SavedGraphRow from '$lib/components/features/composer/SavedGraphRow.svelte';
  import { NODE_H, NODE_H_SUM, NODE_W } from '$lib/components/features/composer/graph/layout';
  import type { GraphCategory, GraphIssue, GraphListItem, GraphParamSpec } from '$lib/types';

  // What the /composer route is actually made of, per the source. The shared UI
  // pieces are used as they come, so their chips go to their own pages.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    {
      name: 'GraphEditor',
      note: 'features/composer · the whole editor, not mounted here (needs the Tauri backend)',
      children: [
        {
          name: 'toolbar',
          note: 'name field · Unsaved / Saved · time · Open ▾ · Save / Save as / New · zoom',
          children: [
            { name: 'Icon', href: '/gallery/icons', note: 'every toolbar button' },
            { name: 'SavedGraphRow', href: '#savedgraphrow', note: 'features/composer · one per saved graph in Open ▾' },
            { name: 'RunButton', href: '/gallery/runbutton', note: 'Run; a Cancel button takes its place during a run' },
          ],
        },
        { name: 'ErrorAlert', href: '/gallery/erroralert', note: 'method list failed to load · save / open failures' },
        {
          name: 'canvas',
          note: 'hand-rolled pan + zoom; layout.ts places every card',
          children: [
            { name: 'EmptyState', href: '/gallery/emptystate', note: '“Start by adding a media source” + Add source' },
            { name: 'NodeCard', href: '#nodecard', note: 'features/composer/graph · one per node' },
            { name: '+ handles', note: 'before / after / below a card, and on every connection' },
            { name: 'picker', note: 'hand-rolled popover — the methods that fit at that +' },
          ],
        },
        {
          name: 'side column',
          note: '280px, narrower when the window is',
          children: [
            {
              name: 'NodeInspector',
              note: 'features/composer/graph · inputs and parameters of the selected node',
              children: [{ name: 'ParamField', href: '#paramfield', note: 'features/composer/graph · one per declared parameter' }],
            },
            { name: 'EmptyState', href: '/gallery/emptystate', note: 'nothing selected' },
          ],
        },
        {
          name: 'status bar',
          note: 'node count · Ready to run / the error (click to jump to its node) / warning count / Cancelled',
        },
        { name: 'issue list', note: 'one row per warning; a row about a node selects it and centres it' },
        {
          name: 'result',
          children: [
            { name: 'ErrorAlert', href: '/gallery/erroralert', note: 'failed run' },
            {
              name: 'Panel',
              href: '/gallery/panel',
              note: '“Result”',
              children: [{ name: 'StatusBadge', href: '/gallery/statusbadge', note: 'completed / cancelled, in actions' }],
            },
          ],
        },
      ],
    },
  ];

  // ---------------------------------------------------------------- NodeCard
  // Static demo cards in every state, worded as the real manifest and validator
  // would word them.
  interface DemoCard {
    caption: string;
    label: string;
    category: GraphCategory;
    summary: string;
    selected?: boolean;
    active?: boolean;
    requiresFfmpeg?: boolean;
    issue?: GraphIssue;
  }

  const demoCards: DemoCard[] = [
    { caption: 'Normal', label: 'File', category: 'source', summary: 'File path: clip.mp4' },
    { caption: 'Selected', label: 'Detect motion', category: 'analysis', summary: 'Threshold: 30', selected: true },
    { caption: 'Running', label: 'Extract frames (disk)', category: 'extract', summary: 'Output directory: frames', active: true },
    { caption: 'Needs ffmpeg', label: 'Convert to HLS', category: 'convert', summary: 'Output directory: hls', requiresFfmpeg: true },
    {
      caption: 'Error',
      label: 'Group similar images',
      category: 'analysis',
      summary: 'Input directory: from upstream',
      issue: { severity: 'error', nodeId: 'n2', message: "'Group similar images' needs 'Output directory' to be set" },
    },
    {
      caption: 'Warning',
      label: 'Extract metadata',
      category: 'extract',
      summary: 'Include thumbnail',
      issue: { severity: 'warning', nodeId: 'n1', message: "'Extract metadata' ignores unknown parameter 'windowLines'" },
    },
    {
      caption: 'Selected + error',
      label: 'Annotate video',
      category: 'annotate',
      summary: '',
      selected: true,
      issue: { severity: 'error', nodeId: 'n3', message: "'Annotate video' needs 'Output video' to be set" },
    },
  ];

  // ---------------------------------------------------------------- ParamField
  // One declaration per kind, copied from the manifest. The real list comes from
  // `list_graph_methods`, which a browser cannot reach.
  interface DemoParam {
    spec: GraphParamSpec;
    canInherit?: boolean;
  }

  const demoParams: DemoParam[] = [
    { spec: { key: 'includeThumbnail', kind: 'bool', label: 'Include thumbnail', default: false, required: false, output: false } },
    { spec: { key: 'interval', kind: 'int', label: 'Frame interval', default: 30, min: 1, required: false, output: false } },
    { spec: { key: 'threshold', kind: 'float', label: 'Threshold', default: 25, min: 0, required: false, output: false } },
    { spec: { key: 'url', kind: 'string', label: 'Stream URL', required: true, output: false } },
    { spec: { key: 'path', kind: 'path', label: 'File path', required: true, output: false } },
    {
      spec: {
        key: 'inputDir',
        kind: 'dir',
        label: 'Input directory',
        required: false,
        fallbackFrom: 'videoProcessOutputDir',
        hint: 'Leave blank to use the upstream frame output directory.',
        output: false,
      },
      canInherit: true,
    },
    {
      spec: {
        key: 'algorithm',
        kind: 'enum',
        label: 'Algorithm',
        default: 'frame_diff',
        options: [
          { value: 'frame_diff', label: 'Frame difference (fastest)' },
          { value: 'mog2', label: 'MOG2 background subtraction' },
          { value: 'knn', label: 'KNN background subtraction' },
          { value: 'optical_flow', label: 'Optical flow' },
        ],
        required: false,
        output: false,
      },
    },
  ];

  let demoValues = $state<Record<string, unknown>>({ path: '/tmp/clip.mp4' });

  // ---------------------------------------------------------------- SavedGraphRow
  const MINUTE = 60_000;
  const DAY = 24 * 60 * MINUTE;
  const now = Date.now();

  const demoRows: { item: GraphListItem; open: boolean }[] = [
    {
      item: { id: '6f1c2a9e-4b7d-4f0a-9c3e-2d8b5a1e7f40', label: 'Frames then groups', nodeCount: 3, modifiedMs: now - 5 * MINUTE, valid: true },
      open: false,
    },
    {
      item: {
        id: '0b9e4d2c-8a1f-4e6b-b7c5-3f2a9d8e1c60',
        label: 'Half-drawn HLS',
        nodeCount: 2,
        modifiedMs: now - DAY,
        valid: false,
        reason: "'Convert to HLS' needs 'Output directory' to be set",
      },
      open: false,
    },
    {
      item: {
        id: 'a3f9c2e1-5d6b-4c8a-9e7f-1b2d3c4e5f60',
        label: '',
        nodeCount: 0,
        modifiedMs: now - 3 * DAY,
        valid: false,
        broken: 'expected value at line 1 column 1',
      },
      open: false,
    },
    {
      item: { id: 'c7d8e9f0-1a2b-4c3d-8e4f-5a6b7c8d9e00', label: 'Motion report', nodeCount: 4, modifiedMs: now - 60 * MINUTE, valid: true },
      open: true,
    },
  ];

  let lastRowClick = $state('');

  const nodeCardCode = `<!-- box comes from layoutGraph(); its w / h are NODE_W and NODE_H or NODE_H_SUM -->
<NodeCard
  label={graph.labelOfNode(box.id)}
  category={meta?.category ?? 'process'}
  summary={graph.summaryOf(box.id)}
  width={box.w}
  height={box.h}
  selected={selected === box.id}
  active={graph.activeNodeId === box.id}
  requiresFfmpeg={meta?.requiresFfmpeg ?? false}
  issue={issueByNode.get(box.id) ?? null}
  onselect={() => (selected = box.id)}
/>`;

  const paramFieldCode = `<!-- NodeInspector: one field per parameter the manifest declares -->
{#each meta.params as spec (spec.key)}
  <ParamField
    {spec}
    value={values[spec.key]}
    canInherit={canInherit(spec.fallbackFrom)}
    onchange={(v) => graph.setParam(selectedId, spec.key, v)}
  />
{/each}`;

  const savedGraphRowCode = `<!-- Inside the Open ▾ popover; savedList comes from list_graphs -->
{#each savedList as item (item.id)}
  <SavedGraphRow
    {item}
    open={item.id === graph.openId}
    onopen={() => openSaved(item)}
    ondelete={() => deleteSaved(item)}
  />
{/each}`;
</script>

<svelte:head>
  <title>Composer · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /composer route is assembled — a single GraphEditor holding a toolbar, a canvas of node cards, an inspector, a status bar and the run result. The editor itself is not mounted here: it loads the method list from the Tauri backend as it opens and takes over ⌘S, ⌘0, ⌘9, ⌘+ and ⌘− for the whole window. The full editor is at /composer in the app."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="nodecard"
    title="NodeCard"
    description="features/composer/graph — one card per node, sized by layout.ts. A validation message about the node marks the card: red frame and red ⚠ for an error, amber ⚠ for a warning; hover the card for the message. An error keeps its red frame while selected, with the selection shown as the blue ring. The ⓘ only says the method needs ffmpeg."
  >
    <Showcase code={nodeCardCode}>
      {#each demoCards as card (card.caption)}
        <div class="flex flex-col gap-1.5">
          <NodeCard
            label={card.label}
            category={card.category}
            summary={card.summary}
            width={NODE_W}
            height={card.summary ? NODE_H_SUM : NODE_H}
            selected={card.selected ?? false}
            active={card.active ?? false}
            requiresFfmpeg={card.requiresFfmpeg ?? false}
            issue={card.issue ?? null}
            onselect={() => {}}
          />
          <span class="text-meta">{card.caption}</span>
        </div>
      {/each}
    </Showcase>
  </GallerySection>

  <GallerySection
    id="paramfield"
    title="ParamField"
    description="features/composer/graph — one field per declared parameter, the control picked by its kind. The placeholder says what happens when it is left blank: Required, the default, or From upstream when an upstream node can supply it. Browse opens the Tauri file dialog (in-app only)."
  >
    <Showcase code={paramFieldCode}>
      <div class="w-full max-w-xl grid grid-cols-1 sm:grid-cols-2 gap-4">
        {#each demoParams as param (param.spec.key)}
          <div class="flex flex-col gap-1">
            <span class="text-meta font-mono">{param.spec.kind}</span>
            <ParamField
              spec={param.spec}
              value={demoValues[param.spec.key]}
              canInherit={param.canInherit ?? false}
              onchange={(v) => (demoValues[param.spec.key] = v)}
            />
          </div>
        {/each}
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="savedgraphrow"
    title="SavedGraphRow"
    description="features/composer — one saved graph in the Open ▾ list: name, node count and when it was saved. A graph that cannot run as things stand gets an amber ⚠ (hover for why); a file that cannot be read back shows its file name and can only be deleted; the graph on the canvas is tinted and tagged OPEN."
  >
    <Showcase code={savedGraphRowCode}>
      <div class="flex flex-col gap-2">
        <div
          class="w-80 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] shadow-xl overflow-hidden"
        >
          {#each demoRows as row (row.item.id)}
            <SavedGraphRow
              item={row.item}
              open={row.open}
              onopen={() => (lastRowClick = `open ${row.item.label}`)}
              ondelete={() => (lastRowClick = `delete ${row.item.broken ? 'unreadable file' : row.item.label}`)}
            />
          {/each}
        </div>
        <p class="text-meta">Last click: {lastRowClick || 'none'}</p>
      </div>
    </Showcase>
  </GallerySection>
</div>
