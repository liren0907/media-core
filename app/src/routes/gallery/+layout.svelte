<script lang="ts">
  import { page } from '$app/state';
  import { theme, effectiveTheme } from '$lib/theme.svelte';
  import { galleryGroups, galleryLabel } from '$lib/components/features/gallery';
  import { Icon } from '$lib/components/ui';

  let { children } = $props();

  // Top-bar title reflects the current section (falls back to "Overview" on /gallery).
  const currentLabel = $derived(galleryLabel(page.url.pathname) ?? 'Overview');
  const overviewActive = $derived(page.url.pathname === '/gallery');

  function toggleTheme() {
    if (theme.value === 'auto') {
      theme.set(effectiveTheme.value === 'dark' ? 'light' : 'dark');
    } else {
      theme.set(theme.value === 'dark' ? 'light' : 'dark');
    }
  }
</script>

<div class="flex h-screen w-full bg-[#f6f7f8] dark:bg-[#101922] text-slate-900 dark:text-slate-50 overflow-hidden">
  <!-- Gallery's own sidebar (replaces the app sidebar while inside /gallery) -->
  <aside class="flex flex-col w-60 shrink-0 border-r border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#1a222c]">
    <!-- Brand -->
    <div class="flex items-center gap-2 px-4 h-12 border-b border-slate-200 dark:border-[#2a3441] shrink-0">
      <div class="flex items-center justify-center size-7 rounded-lg bg-[#137fec] text-white shrink-0">
        <Icon name="palette" class="text-lg" />
      </div>
      <span class="text-card-title text-sm font-display">Gallery</span>
    </div>

    <!-- Section nav (scrolls independently) -->
    <nav class="flex-1 overflow-y-auto scrollbar-hide p-3 flex flex-col gap-4">
      <a
        href="/gallery"
        class="flex items-center gap-2 px-2 py-1.5 rounded-md text-nav transition-colors {overviewActive ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}"
      >
        <Icon name="grid_view" class="text-[18px] shrink-0" />
        Overview
      </a>

      {#each galleryGroups as group}
        <div class="flex flex-col gap-0.5">
          <div class="text-stat-label px-2 mb-1">{group.title}</div>
          {#each group.items as item (item.href)}
            {@const active = page.url.pathname === item.href}
            <a
              href={item.href}
              class="px-2 py-1.5 rounded-md text-nav transition-colors {active ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}"
            >
              {item.label}
            </a>
          {/each}
        </div>
      {/each}
    </nav>

    <!-- Back to the main app, pinned to the bottom -->
    <div class="p-3 border-t border-slate-200 dark:border-[#2a3441] shrink-0">
      <a
        href="/"
        class="flex items-center gap-2 px-2 py-1.5 rounded-md text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039] transition-colors"
      >
        <Icon name="arrow_back" class="text-[20px]" />
        <span class="text-nav">Back to App</span>
      </a>
    </div>
  </aside>

  <!-- Main -->
  <main class="flex-1 flex flex-col h-full overflow-hidden">
    <!-- Top bar -->
    <header class="flex items-center justify-between border-b border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#1a222c] px-6 h-12 shrink-0 z-10">
      <div class="flex items-center gap-2">
        <span class="text-card-title text-sm">{currentLabel}</span>
        <span class="px-2 py-0.5 rounded-full bg-[#137fec]/10 text-badge text-status-info border border-[#137fec]/20 uppercase tracking-wider">Design System</span>
      </div>
      <button
        onclick={toggleTheme}
        aria-label="Toggle theme"
        title="Toggle theme"
        class="flex items-center justify-center rounded-lg size-8 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039] transition-colors"
      >
        <Icon name={effectiveTheme.value === 'dark' ? 'light_mode' : 'dark_mode'} class="text-[18px]" />
      </button>
    </header>

    <!-- Section content -->
    <div class="flex-1 overflow-y-auto overflow-x-hidden scrollbar-hide p-6">
      {@render children()}
    </div>
  </main>
</div>
