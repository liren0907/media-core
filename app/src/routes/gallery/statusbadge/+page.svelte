<script lang="ts">
  import { StatusBadge } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>StatusBadge · Gallery</title>
</svelte:head>

<GallerySection id="statusbadge" title="StatusBadge" description="Colored status pill. Props: status, colorMap? (override colors).">
  <Showcase label="default color map" code={'<StatusBadge status="active" />'}>
    <StatusBadge status="active" />
    <StatusBadge status="idle" />
    <StatusBadge status="connecting" />
    <StatusBadge status="error" />
    <StatusBadge status="unknown" />
  </Showcase>
  <Showcase label="custom colorMap" code={'<StatusBadge status="live" colorMap={{ live: "bg-purple-500/10 text-purple-600 border-purple-500/20" }} />'}>
    <StatusBadge status="live" colorMap={{ live: 'bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-500/20' }} />
  </Showcase>
</GallerySection>
