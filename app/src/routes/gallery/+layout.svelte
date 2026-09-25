<script lang="ts">
  import { page } from '$app/state';
  import { theme, effectiveTheme } from '$lib/theme.svelte';
  import { galleryGroups, galleryLabel, type GalleryLink } from '$lib/components/features/gallery';
  import { Icon } from '$lib/components/ui';

  let { children } = $props();

  // Top-bar title reflects the current section (falls back to "Overview" on /gallery).
  const currentLabel = $derived(galleryLabel(page.url.pathname) ?? 'Overview');
  const overviewActive = $derived(page.url.pathname === '/gallery');

  // Sidebar filter: matching a group title keeps the whole group; otherwise
  // items are matched by label (or by any child label). Empty groups are dropped.
  let filter = $state('');
  const filterQ = $derived(filter.trim().toLowerCase());
  const filteredGroups = $derived.by(() => {
    const q = filterQ;
    if (!q) return galleryGroups;
    return galleryGroups
      .map((g) => {
        if (g.title.toLowerCase().includes(q)) return g;
        const items = g.items
          .map((i) => {
            if (i.label.toLowerCase().includes(q)) return i;
            const children = i.children?.filter((c) => c.label.toLowerCase().includes(q)) ?? [];
            return children.length ? { ...i, children } : null;
          })
          .filter((i): i is GalleryLink => i !== null);
        return { ...g, items };
      })
      .filter((g) => g.items.length > 0);
  });

  // Expand/collapse state for items with children (keyed by item id). Entering a
  // page auto-expands its entry; manual toggles are kept while navigating.
  let expanded = $state<Record<string, boolean>>({});
  $effect(() => {
    const path = page.url.pathname;
    for (const group of galleryGroups) {
      for (const item of group.items) {
        if (item.children?.length && item.href === path) expanded[item.id] = true;
      }
    }
  });

  // The nav is taller than the viewport — keep the active link in view.
  let navEl = $state<HTMLElement | null>(null);
  $effect(() => {
    void page.url.pathname;
    navEl?.querySelector('[aria-current="page"]')?.scrollIntoView({ block: 'nearest' });
  });

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

    <!-- Filter (pinned above the scrolling nav) -->
    <div class="px-3 pt-3 shrink-0">
      <div class="flex items-center gap-2 rounded-md border border-slate-200 dark:border-[#2a3441] bg-[#f6f7f8] dark:bg-[#101922] px-2 py-1 focus-within:border-[#137fec] transition-colors">
        <Icon name="search" class="text-[14px] text-slate-400 shrink-0" />
        <input
          bind:value={filter}
          onkeydown={(e) => { if (e.key === 'Escape') filter = ''; }}
          type="text"
          placeholder="Filter sections…"
          class="w-full bg-transparent text-nav outline-none placeholder:text-slate-400"
        />
        {#if filter}
          <button onclick={() => (filter = '')} aria-label="Clear filter" class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
            <Icon name="close" class="text-[12px]" />
          </button>
        {/if}
      </div>
    </div>

    <!-- Section nav (scrolls independently) -->
    <nav bind:this={navEl} class="flex-1 overflow-y-auto scrollbar-hide p-3 flex flex-col gap-4">
      <a
        href="/gallery"
        aria-current={overviewActive ? 'page' : undefined}
        class="flex items-center gap-2 px-2 py-1.5 rounded-md text-nav transition-colors {overviewActive ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}"
      >
        <Icon name="grid_view" class="text-[18px] shrink-0" />
        Overview
      </a>

      {#each filteredGroups as group (group.title)}
        <div class="flex flex-col gap-0.5">
          <div class="text-stat-label px-2 mb-1">{group.title}</div>
          {#each group.items as item (item.href)}
            {@const active = page.url.pathname === item.href}
            {@const isExpanded = filterQ
              ? (item.children ?? []).some((c) => c.label.toLowerCase().includes(filterQ))
              : !!expanded[item.id]}
            <div class="flex items-center gap-0.5">
              <a
                href={item.href}
                aria-current={active ? 'page' : undefined}
                class="flex-1 px-2 py-1.5 rounded-md text-nav transition-colors {active ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}"
              >
                {item.label}
              </a>
              {#if item.children?.length}
                <button
                  onclick={() => (expanded[item.id] = !expanded[item.id])}
                  aria-expanded={isExpanded}
                  aria-label="{isExpanded ? 'Collapse' : 'Expand'} {item.label}"
                  class="flex items-center justify-center size-6 shrink-0 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#283039] transition-colors"
                >
                  <Icon name="chevron_right" class="text-[14px] transition-transform {isExpanded ? 'rotate-90' : ''}" />
                </button>
              {/if}
            </div>
            {#if item.children?.length && isExpanded}
              <div class="ml-3 pl-2 border-l border-slate-200 dark:border-[#2a3441] flex flex-col gap-0.5">
                {#each item.children as child (child.href)}
                  {@const childActive = page.url.pathname + page.url.hash === child.href}
                  <a
                    href={child.href}
                    aria-current={childActive ? 'location' : undefined}
                    class="px-2 py-1 rounded-md text-nav transition-colors {childActive ? 'bg-[#137fec]/10 text-[#137fec]' : 'text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#283039] hover:text-slate-700 dark:hover:text-slate-200'}"
                  >
                    {child.label}
                  </a>
                {/each}
              </div>
            {/if}
          {/each}
        </div>
      {/each}

      {#if filter && filteredGroups.length === 0}
        <p class="text-caption px-2">No matches.</p>
      {/if}
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
