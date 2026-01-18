<script>
	import "../app.css";
    import { page } from '$app/stores';
</script>

<div class="flex h-screen w-full bg-[#f6f7f8] dark:bg-[#101922] text-slate-900 dark:text-slate-50 overflow-hidden font-body">
    <!-- Side Navigation -->
    <aside class="flex w-64 flex-col border-r border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#1a222c]">
        <div class="flex flex-col h-full justify-between p-4">
            <div class="flex flex-col gap-6">
                <!-- Branding -->
                <div class="flex items-center gap-3 px-2">
                    <div class="flex items-center justify-center size-10 rounded-lg bg-[#137fec] text-white">
                        <span class="material-symbols-outlined text-2xl">hub</span>
                    </div>
                    <div class="flex flex-col">
                        <h1 class="text-base font-bold leading-tight font-display">Media Core</h1>
                        <p class="text-slate-500 dark:text-slate-400 text-xs font-normal">v2.4.0 <span class="text-green-500">• Online</span></p>
                    </div>
                </div>

                <!-- Navigation Links -->
                <nav class="flex flex-col gap-2">
                    <a href="/" class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors group {$page.url.pathname === '/' ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}">
                        <span class="material-symbols-outlined {$page.url.pathname === '/' ? 'filled' : ''}">dashboard</span>
                        <span class="text-sm font-medium leading-normal font-display">Dashboard</span>
                    </a>

                    <a href="/stream" class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors group {$page.url.pathname.startsWith('/stream') ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}">
                        <span class="material-symbols-outlined">videocam</span>
                        <span class="text-sm font-medium leading-normal font-display">Streams</span>
                    </a>

                    <a href="/inferencer" class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors group {$page.url.pathname.startsWith('/inferencer') ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}">
                        <span class="material-symbols-outlined">bar_chart</span>
                        <span class="text-sm font-medium leading-normal font-display">Analytics</span>
                    </a>

                    <a href="/annotator" class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors group {$page.url.pathname.startsWith('/annotator') ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}">
                        <span class="material-symbols-outlined">edit_square</span>
                        <span class="text-sm font-medium leading-normal font-display">Annotator</span>
                    </a>

                    <a href="/hlsViewer" class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors group {$page.url.pathname.startsWith('/hlsViewer') ? 'bg-[#137fec] text-white' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#283039]'}">
                        <span class="material-symbols-outlined">play_circle</span>
                        <span class="text-sm font-medium leading-normal font-display">Player</span>
                    </a>
                </nav>
            </div>

            <!-- Bottom User Profile -->
            <div class="border-t border-slate-200 dark:border-[#2a3441] pt-4">
                <div class="flex items-center gap-3 px-2">
                    <div class="bg-center bg-no-repeat bg-cover rounded-full size-9 shrink-0 bg-gradient-to-br from-blue-500 to-indigo-600"></div>
                    <div class="flex flex-col overflow-hidden">
                        <p class="text-sm font-medium text-slate-900 dark:text-white truncate font-display">Admin User</p>
                        <p class="text-xs text-slate-500 dark:text-slate-400 truncate">admin@mediacore.io</p>
                    </div>
                </div>
            </div>
        </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col h-full overflow-hidden relative">
        <!-- Header -->
        <header class="flex items-center justify-between border-b border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#1a222c] px-8 py-4 shrink-0 z-10">
            <div class="flex items-center gap-4">
                <h2 class="text-xl font-bold font-display tracking-tight text-slate-900 dark:text-white">
                    {#if $page.url.pathname === '/'}
                        Dashboard Overview
                    {:else if $page.url.pathname.startsWith('/stream')}
                        Stream Management
                    {:else if $page.url.pathname.startsWith('/inferencer')}
                        Video Analytics Console
                    {:else if $page.url.pathname.startsWith('/annotator')}
                        Video Annotator
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
            <slot />
        </div>
    </main>
</div>
