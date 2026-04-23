<script lang="ts">
	import "../app.css";
    import { page } from '$app/state';
    import { theme, effectiveTheme } from '$lib/theme.svelte';

    let { children } = $props();
    let isSidebarOpen = $state(true);
    let sidebarWidth = $state(256);
    let isResizing = $state(false);
    const SIDEBAR_MIN = 180;
    const SIDEBAR_MAX = 400;
    const SIDEBAR_COLLAPSED = 80;

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
                            <span class="material-symbols-outlined text-xl">hub</span>
                        </div>
                        <div class="flex flex-col overflow-hidden transition-opacity duration-200 {isSidebarOpen ? 'opacity-100' : 'opacity-0 w-0 hidden'}">
                            <h1 class="text-base font-bold leading-tight font-display whitespace-nowrap">Media Core</h1>
                            <p class="text-slate-500 dark:text-slate-400 text-xs font-normal whitespace-nowrap">v2.4.0 <span class="text-green-500">• Online</span></p>
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
                    {#each [
                        { href: '/', icon: 'dashboard', label: 'Dashboard', match: (p: string) => p === '/' },
                        { href: '/streams', icon: 'videocam', label: 'Streams', match: (p: string) => p.startsWith('/streams') },
                        { href: '/analysis', icon: 'query_stats', label: 'Analysis', match: (p: string) => p.startsWith('/analysis') },
                        { href: '/processing', icon: 'conversion_path', label: 'Processing', match: (p: string) => p.startsWith('/processing') },
                        { href: '/camera', icon: 'photo_camera', label: 'Camera', match: (p: string) => p.startsWith('/camera') },
                        { href: '/benchmark', icon: 'speed', label: 'Benchmark', match: (p: string) => p.startsWith('/benchmark') },
                        { href: '/dedup', icon: 'fingerprint', label: 'Dedup', match: (p: string) => p.startsWith('/dedup') },
                        { href: '/demo-360', icon: 'panorama_photosphere', label: '360° Demo', match: (p: string) => p.startsWith('/demo-360') },
                        { href: '/settings', icon: 'settings', label: 'Settings', match: (p: string) => p.startsWith('/settings') },
                    ] as item}
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

            <!-- Bottom Section: User Profile -->
            <div class="flex flex-col gap-2">
                <button onclick={toggleTheme} class="flex items-center gap-3 px-2 py-1.5 rounded-md transition-colors group relative text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]">
                    <span class="material-symbols-outlined shrink-0">{effectiveTheme.value === 'dark' ? 'light_mode' : 'dark_mode'}</span>
                    <span class="text-nav leading-normal whitespace-nowrap transition-opacity duration-200 {isSidebarOpen ? 'opacity-100' : 'opacity-0 w-0 hidden'}">
                        {effectiveTheme.value === 'dark' ? 'Light Mode' : 'Dark Mode'}
                    </span>
                    
                    {#if !isSidebarOpen}
                        <div class="absolute left-full ml-2 px-2 py-1 bg-slate-800 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-50 pointer-events-none">
                            {effectiveTheme.value === 'dark' ? 'Light Mode' : 'Dark Mode'}
                        </div>
                    {/if}
                </button>

            </div>
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
        <header class="flex items-center justify-between border-b border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#1a222c] px-8 py-4 shrink-0 z-10">
            <div class="flex items-center gap-4">
                <h2 class="text-page-title">
                    {#if page.url.pathname === '/'}
                        Dashboard
                    {:else if page.url.pathname.startsWith('/streams')}
                        Streams
                    {:else if page.url.pathname.startsWith('/analysis')}
                        Media Analysis
                    {:else if page.url.pathname.startsWith('/processing')}
                        Processing
                    {:else if page.url.pathname.startsWith('/camera')}
                        Camera Capture
                    {:else if page.url.pathname.startsWith('/benchmark')}
                        Benchmark
                    {:else if page.url.pathname.startsWith('/dedup')}
                        Deduplication
                    {:else if page.url.pathname.startsWith('/demo-360')}
                        360° Demo
                    {:else if page.url.pathname.startsWith('/settings')}
                        Settings
                    {:else}
                        Media Core
                    {/if}
                </h2>
                <span class="px-2 py-0.5 rounded-full bg-green-500/10 text-green-600 dark:text-green-400 text-xs font-bold border border-green-500/20">System Healthy</span>
            </div>
            
            <div class="flex items-center gap-4">
                <div class="relative w-64 hidden md:block">
                    <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none text-slate-500">
                        <span class="material-symbols-outlined text-[20px]">search</span>
                    </div>
                    <input class="block w-full rounded-lg border-0 py-2 pl-10 pr-3 text-slate-900 ring-1 ring-inset ring-slate-300 placeholder:text-slate-400 focus:ring-2 focus:ring-inset focus:ring-[#137fec] sm:text-sm sm:leading-6 dark:bg-[#283039] dark:ring-0 dark:text-white dark:placeholder:text-slate-500" placeholder="Search streams or tasks..." type="text"/>
                </div>
                
                <div class="flex gap-2">
                    <button class="flex items-center justify-center rounded-lg size-10 bg-slate-100 dark:bg-[#283039] text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors relative">
                        <span class="material-symbols-outlined text-[20px]">notifications</span>
                        <span class="absolute top-2.5 right-2.5 size-2 bg-[#137fec] rounded-full border-2 border-white dark:border-[#283039]"></span>
                    </button>
                    <button class="flex items-center justify-center rounded-lg size-10 bg-slate-100 dark:bg-[#283039] text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors">
                        <span class="material-symbols-outlined text-[20px]">help</span>
                    </button>
                </div>
            </div>
        </header>

        <!-- Scrollable Content Area -->
        <div class="flex-1 overflow-y-auto scrollbar-hide">
            {@render children()}
        </div>
    </main>
</div>
