<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, ToggleSwitch, RunButton, ProgressBar, EmptyState, ErrorAlert, Icon } from '$lib/components/ui';
  import ActionBar from '$lib/components/features/dedup/ActionBar.svelte';
  import CompareTab from '$lib/components/features/dedup/CompareTab.svelte';
  import ComparisonResults from '$lib/components/features/dedup/ComparisonResults.svelte';
  import DirectoryTree from '$lib/components/features/dedup/DirectoryTree.svelte';
  import DuplicateGroupCard from '$lib/components/features/dedup/DuplicateGroupCard.svelte';
  import FilePreview from '$lib/components/features/dedup/FilePreview.svelte';
  import FingerprintTab from '$lib/components/features/dedup/FingerprintTab.svelte';
  import ProgressPanel from '$lib/components/features/dedup/ProgressPanel.svelte';
  import SourcesTab from '$lib/components/features/dedup/SourcesTab.svelte';
  import StatsTab from '$lib/components/features/dedup/StatsTab.svelte';
  import type { DedupGroupExpanded, DedupMediaFile, DedupTreeNode } from '$lib/types';

  // What the /dedup route is actually made of, per the source — a sticky
  // Sources/Targets sidebar plus a results column, no TabBar.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    {
      name: 'Panel',
      href: '#panel',
      note: '“Sources” + “Targets” · sticky page sidebar',
      children: [
        { name: 'EmptyState', href: '#emptystate', note: 'until a Source is set' },
        { name: 'Icon', href: '/gallery/icons', note: 'scan / role / delete row actions' },
      ],
    },
    {
      name: 'Panel',
      href: '#panel',
      note: '“Files” · results column',
      children: [
        {
          name: 'DirectoryTree',
          note: 'features/dedup',
          children: [{ name: 'FilePreview', note: 'features/dedup' }],
        },
      ],
    },
    {
      name: 'ActionBar',
      note: 'features/dedup · results column',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'ToggleSwitch', href: '#toggleswitch', note: 'BLAKE3 / pHash / dHash' },
        { name: 'RunButton', href: '#runbutton', note: 'Fingerprint / Compare' },
        {
          name: 'ProgressPanel',
          note: 'features/dedup',
          children: [{ name: 'ProgressBar', href: '#progressbar' }],
        },
      ],
    },
    {
      name: 'ComparisonResults',
      note: 'features/dedup · results column',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'EmptyState', href: '#emptystate' },
        { name: 'FilePreview', note: 'features/dedup' },
      ],
    },
    { name: 'ErrorAlert', href: '#erroralert', note: 'conditional' },
  ];

  // Static demo state so every section renders fully without a backend.
  let demoBlake3 = $state(true);
  let demoPHash = $state(true);
  let demoDHash = $state(false);

  const panelCode = `<Panel title="Sources" icon="star">
  {#snippet actions()}
    <button onclick={() => addAndScanSource('source')} class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
      <Icon name="add" class="text-[14px]" /> Set
    </button>
  {/snippet}
  …
</Panel>`;

  const toggleCode = `<ToggleSwitch bind:checked={useBlake3} label="BLAKE3" />
<ToggleSwitch bind:checked={usePHash} label="pHash" />
<ToggleSwitch bind:checked={useDHash} label="dHash" />`;

  const runButtonCode = `<RunButton
  loading={isFingerprintRunning}
  disabled={isProcessing || !sourceId}
  label="Fingerprint"
  onclick={runFingerprint}
/>`;

  const progressBarCode = `<ProgressBar {percent} color="bg-[#137fec]" />`;

  const emptyStateCode = `<EmptyState icon="star" message="Set a master directory as Source" />`;

  const errorAlertCode = `{#if error}
  <ErrorAlert message={error} />
{/if}`;

  // Mock data for the assembled feature components.
  const mediaA: DedupMediaFile = { id: 'media_file:s1', filePath: '/photos/a.jpg', fileName: 'a.jpg', fileSize: 204800, fileType: 'image', parentDir: '/photos', contentHash: 'abc123', phash: null, dhash: null };
  const mediaB: DedupMediaFile = { id: 'media_file:t1', filePath: '/backup/a.jpg', fileName: 'a.jpg', fileSize: 204800, fileType: 'image', parentDir: '/backup', contentHash: 'abc123', phash: null, dhash: null };

  let mockGroups = $state<DedupGroupExpanded[]>([
    {
      id: 'duplicate_group:1', matchType: 'exact', similarityScore: 1, algorithm: 'blake3',
      members: [mediaA, mediaB], sourceMember: mediaA, targetMember: mediaB, createdAt: '2026-06-01T12:00:00Z',
    },
  ]);

  const groupCard: DedupGroupExpanded = {
    id: 'duplicate_group:2', matchType: 'similar', similarityScore: 0.92, algorithm: 'pHash',
    sourceMember: null, targetMember: null, createdAt: null,
    members: [
      { id: 'media_file:1', filePath: '/photos/a.jpg', fileName: 'a.jpg', fileSize: 204800, fileType: 'image', parentDir: '/photos', contentHash: null, phash: 'ph1', dhash: null },
      { id: 'media_file:2', filePath: '/photos/b.jpg', fileName: 'b.jpg', fileSize: 198000, fileType: 'image', parentDir: '/photos', contentHash: null, phash: 'ph2', dhash: null },
    ],
  };

  const treeNodes: DedupTreeNode[] = [
    {
      name: 'photos', path: '/photos', isDir: true, fileSize: null,
      children: [
        { name: 'a.jpg', path: '/photos/a.jpg', isDir: false, children: [], fileSize: 204800 },
        { name: 'b.jpg', path: '/photos/b.jpg', isDir: false, children: [], fileSize: 198000 },
      ],
    },
  ];
</script>

<svelte:head>
  <title>Dedup · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /dedup route is assembled — a sticky Sources/Targets sidebar beside the Files / ActionBar / results column. Blue chips jump to that component's section below (or to its gallery page)."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="panel"
    title="Panel"
    description="“Sources” and “Targets” put compact text actions (with inline icons) in the actions snippet; “Files” wraps the directory tree."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full max-w-sm">
        <Panel title="Sources" icon="star">
          {#snippet actions()}
            <button class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
              <Icon name="add" class="text-[14px]" /> Set
            </button>
          {/snippet}
          <div class="p-3 text-caption">Panel body — the source directory list lives here.</div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="toggleswitch"
    title="ToggleSwitch"
    description="ActionBar's algorithm pickers — BLAKE3 (exact), pHash and dHash (perceptual)."
    link={{ href: '/gallery/toggleswitch', label: 'Component page' }}
  >
    <Showcase code={toggleCode}>
      <ToggleSwitch bind:checked={demoBlake3} label="BLAKE3" />
      <ToggleSwitch bind:checked={demoPHash} label="pHash" />
      <ToggleSwitch bind:checked={demoDHash} label="dHash" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="runbutton"
    title="RunButton"
    description="ActionBar overrides the default label — one button per pipeline stage."
    link={{ href: '/gallery/runbutton', label: 'Component page' }}
  >
    <Showcase code={runButtonCode}>
      <RunButton loading={false} label="Fingerprint" onclick={() => {}} />
      <RunButton loading={false} label="Compare" onclick={() => {}} />
      <RunButton loading={true} label="Fingerprint" onclick={() => {}} />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="progressbar"
    title="ProgressBar"
    description="Inside ProgressPanel — scan, fingerprint, and compare stages all report through it."
    link={{ href: '/gallery/progressbar', label: 'Component page' }}
  >
    <Showcase code={progressBarCode}>
      <div class="w-full max-w-md">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-meta">Computing Fingerprints — 42 / 100 files</span>
          <span class="text-meta">42%</span>
        </div>
        <ProgressBar percent={42} color="bg-[#137fec]" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="emptystate"
    title="EmptyState"
    description="Guides setup before any source is scanned, and fills the results column before a compare run."
    link={{ href: '/gallery/emptystate', label: 'Component page' }}
  >
    <Showcase code={emptyStateCode}>
      <div class="w-full">
        <EmptyState icon="star" message="Set a master directory as Source" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="erroralert"
    title="ErrorAlert"
    description="Scan / fingerprint / compare failures surface in place."
    link={{ href: '/gallery/erroralert', label: 'Component page' }}
  >
    <Showcase code={errorAlertCode}>
      <div class="w-full">
        <ErrorAlert message="Scan failed: directory not readable" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="Every feature component in features/dedup rendered whole. Presentational ones are driven by mock data; the four *Tab components self-fetch and are kept for reference — the current route composes the sidebar + results layout directly."
  >
    <LiveComponent name="StatsTab" note="no props · self-fetches"><StatsTab /></LiveComponent>
    <LiveComponent name="SourcesTab" note="no props · self-fetches"><SourcesTab /></LiveComponent>
    <LiveComponent name="FingerprintTab" note="no props · self-fetches"><FingerprintTab /></LiveComponent>
    <LiveComponent name="CompareTab" note="no props · self-fetches"><CompareTab /></LiveComponent>
    <LiveComponent name="ActionBar" note="prop: sourceId">
      <ActionBar sourceId="scan_source:abc" activeSourceIds={['scan_source:abc']} onFingerprintDone={() => {}} onCompareDone={() => {}} />
    </LiveComponent>
    <LiveComponent name="ProgressPanel" note="title + percent">
      <ProgressPanel title="Computing Fingerprints" percent={42} message="42 / 100 files" detail="/photos/a.jpg" />
    </LiveComponent>
    <LiveComponent name="DuplicateGroupCard" note="prop: group">
      <DuplicateGroupCard group={groupCard} />
    </LiveComponent>
    <LiveComponent name="ComparisonResults" note="bindable: groups">
      <ComparisonResults bind:groups={mockGroups} onTrashDone={() => {}} />
    </LiveComponent>
    <LiveComponent name="DirectoryTree" note="prop: nodes">
      <DirectoryTree nodes={treeNodes} onPreviewToggle={() => {}} />
    </LiveComponent>
    <LiveComponent name="FilePreview" note="filePath + fileType + fileName">
      <FilePreview filePath="/photos/a.jpg" fileType="image" fileName="a.jpg" fileSize={204800} />
    </LiveComponent>
  </GallerySection>
</div>
