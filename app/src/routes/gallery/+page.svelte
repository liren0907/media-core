<script lang="ts">
  import { galleryGroups } from '$lib/components/features/gallery';
  import { Icon } from '$lib/components/ui';
</script>

<svelte:head>
  <title>Component Gallery</title>
</svelte:head>

<div class="max-w-4xl flex flex-col gap-8">
  <!-- Intro -->
  <div class="flex flex-col gap-2">
    <h1 class="text-page-title flex items-center gap-2">
      <Icon name="palette" class="text-status-info" />
      Component Gallery
    </h1>
    <p class="text-body text-slate-500 dark:text-slate-400 max-w-2xl">
      A living reference of this app's pages, design tokens, and UI primitives. Pick a
      section from the left, or jump in below. Use the theme toggle (top-right) to preview light &amp; dark.
    </p>
  </div>

  <!-- Grouped quick links -->
  {#each galleryGroups as group}
    <div class="flex flex-col gap-3">
      <h2 class="text-section-title">{group.title}</h2>
      <div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
        {#each group.items as item (item.href)}
          <a
            href={item.href}
            class="group rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] p-4 flex items-center justify-between gap-2 hover:border-[#137fec] hover:shadow-xs transition-all"
          >
            <span class="text-card-title text-sm">{item.label}</span>
            <Icon name="arrow_forward" class="text-[18px] text-slate-300 dark:text-slate-600 group-hover:text-[#137fec] transition-colors" />
          </a>
        {/each}
      </div>
    </div>
  {/each}
</div>
