<script lang="ts">
  import { GallerySection, LiveComponent } from '$lib/components/features/gallery';
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

<GallerySection
  id="dedup"
  title="Dedup"
  description="Feature components composed by the /dedup route. Presentational ones are driven by mock data; the tabs (Stats / Sources / Fingerprint / Compare) self-fetch from the backend."
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
