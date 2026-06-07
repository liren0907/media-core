<script lang="ts">
  import { StatCard, ProgressBar } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>StatCard · Gallery</title>
</svelte:head>

<GallerySection id="statcard" title="StatCard" description="Metric tile. Props: label, icon, iconColor?, value, sub?, extra? (snippet).">
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
    <Showcase label="basic" code={'<StatCard label="Files" icon="folder" value="1,234" />'}>
      <div class="w-full"><StatCard label="Files" icon="folder" value="1,234" /></div>
    </Showcase>
    <Showcase label="with sub" code={'<StatCard label="Speed" icon="bolt" value="12.3" sub="ms" />'}>
      <div class="w-full"><StatCard label="Speed" icon="bolt" value="12.3" sub="ms" /></div>
    </Showcase>
    <Showcase label="custom iconColor" code={'<StatCard … iconColor="text-[#137fec]" />'}>
      <div class="w-full"><StatCard label="Active" icon="check_circle" iconColor="text-[#137fec]" value="OK" /></div>
    </Showcase>
    <Showcase label="with extra slot" code={'<StatCard …>\n  {#snippet extra()}<ProgressBar percent={64} />{/snippet}\n</StatCard>'}>
      <div class="w-full">
        <StatCard label="Progress" icon="donut_large" value="64%">
          {#snippet extra()}
            <div class="mt-2"><ProgressBar percent={64} /></div>
          {/snippet}
        </StatCard>
      </div>
    </Showcase>
  </div>
</GallerySection>
