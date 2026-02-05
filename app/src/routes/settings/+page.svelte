<script lang="ts">
    import { theme, effectiveTheme } from '$lib/theme';
    import { appConfig, type StreamConfig, type PathConfig } from '$lib/config';
    import { open } from '@tauri-apps/plugin-dialog';
    import { onMount } from 'svelte';

    // Options for the dropdown
    const languages = [
        { code: 'en', name: 'English' },
        { code: 'zh-TW', name: '繁體中文' },
        { code: 'ja', name: '日本語' }
    ];
    
    let selectedLanguage = 'en'; // Placeholder
    
    // Stream settings
    let hlsServerUrl = '';
    let hlsServerPort = 1521;
    let defaultRtspUrl = '';
    let hlsOutputDir = '';
    
    // Path settings
    let defaultVideoDir = '';
    let defaultImageDir = '';
    let defaultOutputDir = '';
    let annotationDir = '';
    
    let saveStatus = '';
    
    onMount(() => {
        // Load current config
        const config = $appConfig;
        hlsServerUrl = config.streams.hlsServerUrl;
        hlsServerPort = config.streams.hlsServerPort;
        defaultRtspUrl = config.streams.defaultRtspUrl;
        hlsOutputDir = config.streams.hlsOutputDir;
        defaultVideoDir = config.paths.defaultVideoDir;
        defaultImageDir = config.paths.defaultImageDir;
        defaultOutputDir = config.paths.defaultOutputDir;
        annotationDir = config.paths.annotationDir;
    });
    
    function saveStreamSettings() {
        appConfig.updateStreams({
            hlsServerUrl,
            hlsServerPort,
            defaultRtspUrl,
            hlsOutputDir,
        });
        showSaveStatus();
    }
    
    function savePathSettings() {
        appConfig.updatePaths({
            defaultVideoDir,
            defaultImageDir,
            defaultOutputDir,
            annotationDir,
        });
        showSaveStatus();
    }
    
    function showSaveStatus() {
        saveStatus = 'Settings saved!';
        setTimeout(() => { saveStatus = ''; }, 2000);
    }
    
    function resetToDefaults() {
        if (confirm('Reset all settings to defaults?')) {
            appConfig.reset();
            // Reload values
            const config = $appConfig;
            hlsServerUrl = config.streams.hlsServerUrl;
            hlsServerPort = config.streams.hlsServerPort;
            defaultRtspUrl = config.streams.defaultRtspUrl;
            hlsOutputDir = config.streams.hlsOutputDir;
            defaultVideoDir = config.paths.defaultVideoDir;
            defaultImageDir = config.paths.defaultImageDir;
            defaultOutputDir = config.paths.defaultOutputDir;
            annotationDir = config.paths.annotationDir;
            showSaveStatus();
        }
    }
    
    async function browseDirectory(field: 'video' | 'image' | 'output' | 'annotation' | 'hls') {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Select Directory'
            });
            
            if (selected && typeof selected === 'string') {
                switch (field) {
                    case 'video':
                        defaultVideoDir = selected;
                        break;
                    case 'image':
                        defaultImageDir = selected;
                        break;
                    case 'output':
                        defaultOutputDir = selected;
                        break;
                    case 'annotation':
                        annotationDir = selected;
                        break;
                    case 'hls':
                        hlsOutputDir = selected;
                        break;
                }
            }
        } catch (err) {
            console.error('Failed to open directory picker:', err);
        }
    }
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

    <!-- Stream Settings -->
    <div class="bg-white dark:bg-[#161e27] rounded-xl border border-slate-200 dark:border-[#2a3441] p-6 shadow-sm">
        <div class="flex items-center gap-3 mb-6">
            <div class="p-2 bg-green-500/10 rounded-lg">
                <span class="material-symbols-outlined text-green-500">stream</span>
            </div>
            <div>
                <h2 class="text-lg font-bold text-slate-900 dark:text-white font-display">Stream Settings</h2>
                <p class="text-sm text-slate-500">Configure HLS and RTSP stream defaults</p>
            </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- HLS Server URL -->
            <div class="flex flex-col gap-1">
                <label for="hlsServerUrl" class="text-sm font-medium text-slate-700 dark:text-slate-300">HLS Server URL</label>
                <input 
                    id="hlsServerUrl"
                    type="text" 
                    bind:value={hlsServerUrl}
                    placeholder="http://127.0.0.1"
                    class="bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-[#137fec]"
                />
            </div>
            
            <!-- HLS Server Port -->
            <div class="flex flex-col gap-1">
                <label for="hlsServerPort" class="text-sm font-medium text-slate-700 dark:text-slate-300">HLS Server Port</label>
                <input 
                    id="hlsServerPort"
                    type="number" 
                    bind:value={hlsServerPort}
                    placeholder="1521"
                    class="bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-[#137fec]"
                />
            </div>
            
            <!-- Default RTSP URL -->
            <div class="flex flex-col gap-1 md:col-span-2">
                <label for="defaultRtspUrl" class="text-sm font-medium text-slate-700 dark:text-slate-300">Default RTSP URL</label>
                <input 
                    id="defaultRtspUrl"
                    type="text" 
                    bind:value={defaultRtspUrl}
                    placeholder="rtsp://localhost:8554/mystream"
                    class="bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-[#137fec]"
                />
            </div>
            
            <!-- HLS Output Directory -->
            <div class="flex flex-col gap-1 md:col-span-2">
                <label for="hlsOutputDir" class="text-sm font-medium text-slate-700 dark:text-slate-300">HLS Output Directory</label>
                <div class="flex gap-2">
                    <input 
                        id="hlsOutputDir"
                        type="text" 
                        bind:value={hlsOutputDir}
                        placeholder="hls_output"
                        class="flex-1 bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-[#137fec]"
                    />
                    <button 
                        on:click={() => browseDirectory('hls')}
                        class="px-4 py-2.5 bg-slate-100 dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-slate-300 rounded-lg hover:bg-slate-200 dark:hover:bg-[#242d38] transition-colors"
                    >
                        <span class="material-symbols-outlined text-[20px]">folder_open</span>
                    </button>
                </div>
            </div>
        </div>
        
        <button 
            on:click={saveStreamSettings}
            class="mt-4 px-4 py-2 bg-[#137fec] text-white rounded-lg hover:bg-[#0f6dd3] transition-colors"
        >
            Save Stream Settings
        </button>
    </div>

    <!-- Path Settings -->
    <div class="bg-white dark:bg-[#161e27] rounded-xl border border-slate-200 dark:border-[#2a3441] p-6 shadow-sm">
        <div class="flex items-center gap-3 mb-6">
            <div class="p-2 bg-orange-500/10 rounded-lg">
                <span class="material-symbols-outlined text-orange-500">folder</span>
            </div>
            <div>
                <h2 class="text-lg font-bold text-slate-900 dark:text-white font-display">Default Paths</h2>
                <p class="text-sm text-slate-500">Configure default directories for file operations</p>
            </div>
        </div>

        <div class="grid grid-cols-1 gap-4">
            <!-- Default Video Directory -->
            <div class="flex flex-col gap-1">
                <label for="defaultVideoDir" class="text-sm font-medium text-slate-700 dark:text-slate-300">Default Video Directory</label>
                <div class="flex gap-2">
                    <input 
                        id="defaultVideoDir"
                        type="text" 
                        bind:value={defaultVideoDir}
                        placeholder="Select default video directory..."
                        class="flex-1 bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-[#137fec]"
                    />
                    <button 
                        on:click={() => browseDirectory('video')}
                        class="px-4 py-2.5 bg-slate-100 dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-slate-300 rounded-lg hover:bg-slate-200 dark:hover:bg-[#242d38] transition-colors"
                    >
                        <span class="material-symbols-outlined text-[20px]">folder_open</span>
                    </button>
                </div>
            </div>
            
            <!-- Default Image Directory -->
            <div class="flex flex-col gap-1">
                <label for="defaultImageDir" class="text-sm font-medium text-slate-700 dark:text-slate-300">Default Image Directory</label>
                <div class="flex gap-2">
                    <input 
                        id="defaultImageDir"
                        type="text" 
                        bind:value={defaultImageDir}
                        placeholder="Select default image directory..."
                        class="flex-1 bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-[#137fec]"
                    />
                    <button 
                        on:click={() => browseDirectory('image')}
                        class="px-4 py-2.5 bg-slate-100 dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-slate-300 rounded-lg hover:bg-slate-200 dark:hover:bg-[#242d38] transition-colors"
                    >
                        <span class="material-symbols-outlined text-[20px]">folder_open</span>
                    </button>
                </div>
            </div>
            
            <!-- Default Output Directory -->
            <div class="flex flex-col gap-1">
                <label for="defaultOutputDir" class="text-sm font-medium text-slate-700 dark:text-slate-300">Default Output Directory</label>
                <div class="flex gap-2">
                    <input 
                        id="defaultOutputDir"
                        type="text" 
                        bind:value={defaultOutputDir}
                        placeholder="Select default output directory..."
                        class="flex-1 bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-[#137fec]"
                    />
                    <button 
                        on:click={() => browseDirectory('output')}
                        class="px-4 py-2.5 bg-slate-100 dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-slate-300 rounded-lg hover:bg-slate-200 dark:hover:bg-[#242d38] transition-colors"
                    >
                        <span class="material-symbols-outlined text-[20px]">folder_open</span>
                    </button>
                </div>
            </div>
            
            <!-- Annotation Directory -->
            <div class="flex flex-col gap-1">
                <label for="annotationDir" class="text-sm font-medium text-slate-700 dark:text-slate-300">Annotation Directory</label>
                <div class="flex gap-2">
                    <input 
                        id="annotationDir"
                        type="text" 
                        bind:value={annotationDir}
                        placeholder="Select annotation directory..."
                        class="flex-1 bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-[#137fec]"
                    />
                    <button 
                        on:click={() => browseDirectory('annotation')}
                        class="px-4 py-2.5 bg-slate-100 dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-slate-300 rounded-lg hover:bg-slate-200 dark:hover:bg-[#242d38] transition-colors"
                    >
                        <span class="material-symbols-outlined text-[20px]">folder_open</span>
                    </button>
                </div>
            </div>
        </div>
        
        <button 
            on:click={savePathSettings}
            class="mt-4 px-4 py-2 bg-[#137fec] text-white rounded-lg hover:bg-[#0f6dd3] transition-colors"
        >
            Save Path Settings
        </button>
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
    
    <!-- Reset & Status -->
    <div class="flex items-center justify-between">
        <button 
            on:click={resetToDefaults}
            class="px-4 py-2 bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20 rounded-lg hover:bg-red-500/20 transition-colors"
        >
            Reset All to Defaults
        </button>
        
        {#if saveStatus}
            <div class="flex items-center gap-2 text-green-600 dark:text-green-400">
                <span class="material-symbols-outlined">check_circle</span>
                <span>{saveStatus}</span>
            </div>
        {/if}
    </div>
</div>
