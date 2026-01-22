<script>
    import { theme, effectiveTheme } from '$lib/theme';
    import { onMount } from 'svelte';

    // Options for the dropdown
    const languages = [
        { code: 'en', name: 'English' },
        { code: 'zh-TW', name: '繁體中文' },
        { code: 'ja', name: '日本語' }
    ];
    
    let selectedLanguage = 'en'; // Placeholder
</script>

<svelte:head>
    <title>Settings | Media Core</title>
</svelte:head>

<div class="max-w-4xl mx-auto p-8 flex flex-col gap-8">
    <div class="flex flex-col gap-2">
        <h1 class="text-3xl font-bold text-slate-900 dark:text-white font-display">Settings</h1>
        <p class="text-slate-500 dark:text-slate-400">Manage your application preferences and configurations.</p>
    </div>

    <!-- Theme Settings -->
    <div class="bg-white dark:bg-[#161e27] rounded-xl border border-slate-200 dark:border-[#2a3441] p-6 shadow-sm">
        <div class="flex items-center gap-3 mb-6">
            <div class="p-2 bg-blue-500/10 rounded-lg">
                <span class="material-symbols-outlined text-blue-500">palette</span>
            </div>
            <div>
                <h2 class="text-lg font-bold text-slate-900 dark:text-white font-display">Theme Settings</h2>
                <p class="text-sm text-slate-500">Choose your preferred interface appearance</p>
            </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- Light Mode -->
            <button 
                on:click={() => theme.set('light')}
                class="relative flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all {$theme === 'light' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
            >
                <div class="w-full aspect-video bg-[#f6f7f8] rounded-lg border border-slate-200 shadow-sm flex items-center justify-center overflow-hidden">
                    <div class="w-3/4 h-3/4 bg-white rounded shadow-sm flex flex-col gap-2 p-2">
                        <div class="w-1/3 h-2 bg-slate-200 rounded"></div>
                        <div class="w-full h-16 bg-slate-100 rounded"></div>
                    </div>
                </div>
                <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-slate-600 dark:text-slate-400">light_mode</span>
                    <span class="font-medium text-slate-900 dark:text-white">Light Mode</span>
                </div>
                {#if $theme === 'light'}
                    <div class="absolute top-3 right-3 text-[#137fec]">
                        <span class="material-symbols-outlined filled">check_circle</span>
                    </div>
                {/if}
            </button>

            <!-- Dark Mode -->
            <button 
                on:click={() => theme.set('dark')}
                class="relative flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all {$theme === 'dark' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
            >
                <div class="w-full aspect-video bg-[#101922] rounded-lg border border-slate-700 shadow-sm flex items-center justify-center overflow-hidden">
                    <div class="w-3/4 h-3/4 bg-[#1a222c] rounded shadow-sm flex flex-col gap-2 p-2">
                        <div class="w-1/3 h-2 bg-slate-700 rounded"></div>
                        <div class="w-full h-16 bg-slate-800 rounded"></div>
                    </div>
                </div>
                <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-slate-600 dark:text-slate-400">dark_mode</span>
                    <span class="font-medium text-slate-900 dark:text-white">Dark Mode</span>
                </div>
                {#if $theme === 'dark'}
                    <div class="absolute top-3 right-3 text-[#137fec]">
                        <span class="material-symbols-outlined filled">check_circle</span>
                    </div>
                {/if}
            </button>

            <!-- Auto Mode -->
            <button 
                on:click={() => theme.set('auto')}
                class="relative flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all {$theme === 'auto' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
            >
                <div class="w-full aspect-video bg-gradient-to-br from-[#f6f7f8] to-[#101922] rounded-lg border border-slate-200 dark:border-[#2a3441] shadow-sm flex items-center justify-center">
                    <span class="material-symbols-outlined text-4xl text-slate-400">brightness_auto</span>
                </div>
                <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-slate-600 dark:text-slate-400">settings_brightness</span>
                    <span class="font-medium text-slate-900 dark:text-white">Auto System</span>
                </div>
                {#if $theme === 'auto'}
                    <div class="absolute top-3 right-3 text-[#137fec]">
                        <span class="material-symbols-outlined filled">check_circle</span>
                    </div>
                {/if}
            </button>
        </div>

        <!-- Current Status Indicator -->
        <div class="mt-6 flex items-center gap-2 text-sm text-slate-500 bg-slate-50 dark:bg-[#1a222c] p-3 rounded-lg border border-slate-100 dark:border-[#2a3441]">
            <span class="material-symbols-outlined text-[18px]">info</span>
            <p>
                Current appearance is 
                <span class="font-bold text-slate-900 dark:text-white uppercase">{$effectiveTheme}</span>
                {#if $theme === 'auto'}
                    (determined by system settings)
                {/if}
            </p>
        </div>
    </div>

    <!-- Language Settings (Placeholder) -->
    <div class="bg-white dark:bg-[#161e27] rounded-xl border border-slate-200 dark:border-[#2a3441] p-6 shadow-sm opacity-75">
        <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
                <div class="p-2 bg-purple-500/10 rounded-lg">
                    <span class="material-symbols-outlined text-purple-500">translate</span>
                </div>
                <div>
                    <h2 class="text-lg font-bold text-slate-900 dark:text-white font-display">Language</h2>
                    <p class="text-sm text-slate-500">Select your preferred language</p>
                </div>
            </div>
            <span class="px-2 py-1 bg-amber-500/10 text-amber-600 dark:text-amber-400 text-xs font-bold rounded border border-amber-500/20">Coming Soon</span>
        </div>

        <div class="max-w-md">
            <div class="relative">
                <select 
                    class="w-full appearance-none bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-3 pr-10 focus:outline-none focus:ring-2 focus:ring-[#137fec] disabled:opacity-60 disabled:cursor-not-allowed"
                    bind:value={selectedLanguage}
                    disabled
                >
                    {#each languages as lang}
                        <option value={lang.code}>{lang.name}</option>
                    {/each}
                </select>
                <div class="absolute inset-y-0 right-0 flex items-center px-3 pointer-events-none text-slate-500">
                    <span class="material-symbols-outlined">expand_more</span>
                </div>
            </div>
            <p class="mt-2 text-xs text-slate-400">Language switching functionality is currently under development.</p>
        </div>
    </div>
</div>
