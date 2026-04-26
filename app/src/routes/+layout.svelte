<script lang="ts">
	import "../app.css";
    import { page } from '$app/state';
    import { theme, effectiveTheme } from '$lib/theme.svelte';
    import { navItems } from '$lib/nav';
    import { provideHeaderActions } from '$lib/header.svelte';
    import Breadcrumb from '$lib/components/ui/Breadcrumb.svelte';

    let { children } = $props();
    let isSidebarOpen = $state(true);
    let sidebarWidth = $state(256);
    let isResizing = $state(false);
    const SIDEBAR_MIN = 180;
    const SIDEBAR_MAX = 400;
    const SIDEBAR_COLLAPSED = 80;

    const headerActions = provideHeaderActions();

    function toggleSidebar() {
        isSidebarOpen = !isSidebarOpen;
        if (isSidebarOpen) {
            sidebarWidth = 256;
        }
    }

    function toggleTheme() {
        if (theme.value === 'auto') {
            theme.set(effectiveTheme.value === 'dark' ? 'light' : 'dark');
        } else {
            theme.set(theme.value === 'dark' ? 'light' : 'dark');
        }
    }

    function onResizeStart(e: MouseEvent) {
        e.preventDefault();
        isResizing = true;

        const onMouseMove = (e: MouseEvent) => {
            sidebarWidth = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, e.clientX));
        };

        const onMouseUp = () => {
            isResizing = false;
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);
        };

        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('mouseup', onMouseUp);
    }
</script>

<div class="flex h-screen w-full bg-[#f6f7f8] dark:bg-[#101922] text-slate-900 dark:text-slate-50 overflow-hidden {isResizing ? 'select-none' : ''}">
    <!-- Side Navigation -->
    <aside
        class="flex flex-col border-r border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#1a222c] transition-all duration-300 ease-in-out relative shrink-0"
        style="width: {isSidebarOpen ? sidebarWidth : SIDEBAR_COLLAPSED}px"
    >
        <div class="flex flex-col h-full justify-between p-3">
            <div class="flex flex-col gap-2">
                <!-- Branding -->
                <div class="flex items-center justify-between px-2">
                    <div class="flex items-center gap-2">
                        <div class="flex items-center justify-center size-8 rounded-lg bg-[#137fec] text-white shrink-0">
                            <span class="material-symbols-outlined text-xl">movie</span>
                        </div>
                        <div class="flex flex-col overflow-hidden transition-opacity duration-200 {isSidebarOpen ? 'opacity-100' : 'opacity-0 w-0 hidden'}">
                            <h1 class="text-base font-bold leading-tight font-display whitespace-nowrap">Media Core</h1>
                        </div>
                    </div>

                    <button onclick={toggleSidebar} class="p-1 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#283039] transition-colors {isSidebarOpen ? '' : 'hidden'}">
                        <span class="material-symbols-outlined text-[20px]">first_page</span>
                    </button>
                </div>

                <!-- Collapsed Toggle Button (Centered when sidebar is closed) -->
                <button onclick={toggleSidebar} class="mx-auto p-2 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#283039] transition-colors {!isSidebarOpen ? 'block' : 'hidden'}">
                    <span class="material-symbols-outlined text-[24px]">last_page</span>
                </button>

                <!-- Navigation Links -->
                <nav class="flex flex-col gap-1">
                    {#each navItems as item (item.href)}
                        {@const active = item.match(page.url.pathname)}
                        <a href={item.href} class="flex items-center gap-3 px-2 py-1.5 rounded-md transition-colors group relative {active ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}">
                            <span class="material-symbols-outlined {active ? 'filled' : ''} shrink-0">{item.icon}</span>
                            <span class="text-nav leading-normal whitespace-nowrap transition-opacity duration-200 {isSidebarOpen ? 'opacity-100' : 'opacity-0 w-0 hidden'}">{item.label}</span>
                            {#if !isSidebarOpen}
                                <div class="absolute left-full ml-2 px-2 py-1 bg-slate-800 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-50 pointer-events-none">
                                    {item.label}
                                </div>
                            {/if}
                        </a>
                    {/each}
                </nav>
            </div>

            <!-- Bottom Section: reserved for future user profile / shortcuts -->
            <div class="flex flex-col gap-2"></div>
        </div>
        <!-- Drag Handle -->
        {#if isSidebarOpen}
            <div
                onmousedown={onResizeStart}
                class="absolute top-0 -right-1 w-2 h-full cursor-col-resize z-20 flex justify-center"
            >
                <div class="w-[2px] h-full bg-slate-300 dark:bg-slate-600 hover:bg-[#137fec] transition-colors {isResizing ? 'bg-[#137fec]' : ''}"></div>
            </div>
        {/if}
    </aside>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col h-full overflow-hidden relative">
        <!-- Header -->
        <header class="flex items-center justify-between border-b border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#1a222c] px-6 py-1.5 shrink-0 z-10">
            <Breadcrumb />

            <div class="flex items-center gap-2">
                {#if headerActions.current}
                    {@render headerActions.current()}
                    <div class="h-5 w-px bg-slate-200 dark:bg-slate-700 mx-1"></div>
                {/if}
                <button
                    onclick={toggleTheme}
                    aria-label="Toggle theme"
                    title="Toggle theme"
                    class="flex items-center justify-center rounded-lg size-8 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039] transition-colors"
                >
                    <span class="material-symbols-outlined text-[18px]">{effectiveTheme.value === 'dark' ? 'light_mode' : 'dark_mode'}</span>
                </button>
            </div>
        </header>

        <!-- Scrollable Content Area -->
        <div class="flex-1 overflow-y-auto scrollbar-hide">
            {@render children()}
        </div>
    </main>
</div>
