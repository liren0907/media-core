<script lang="ts">
    import { page } from '$app/state';
    import { buildBreadcrumb } from '$lib/nav';

    const segments = $derived(buildBreadcrumb(page.url.pathname));
</script>

<nav class="flex items-center gap-1.5" aria-label="Breadcrumb">
    {#each segments as seg, i (seg.href)}
        {#if i > 0}
            <span class="material-symbols-outlined text-[18px] text-slate-300 dark:text-slate-600 select-none" aria-hidden="true">chevron_right</span>
        {/if}
        {#if i === segments.length - 1}
            <span class="text-base font-bold font-display tracking-tight text-slate-900 dark:text-white" aria-current="page">{seg.label}</span>
        {:else}
            <a
                href={seg.href}
                class="text-sm font-medium font-display text-slate-500 dark:text-slate-400 hover:text-[#137fec] dark:hover:text-[#137fec] transition-colors"
            >
                {seg.label}
            </a>
        {/if}
    {/each}
</nav>
