<script lang="ts">
    import { theme, effectiveTheme } from '$lib/theme.svelte';
    import { appConfig, type StreamConfig, type PathConfig } from '$lib/config.svelte';
    import { open } from '@tauri-apps/plugin-dialog';
    import { PageContent, Panel, StatusBadge } from '$lib/components/ui';

    const languages = [
        { code: 'en', name: 'English' },
        { code: 'zh-TW', name: '繁體中文' },
        { code: 'ja', name: '日本語' }
    ];

    let selectedLanguage = $state('en');

    // Stream settings
    let hlsServerUrl = $state('');
    let hlsServerPort = $state(1521);
    let defaultRtspUrl = $state('');
    let hlsOutputDir = $state('');

    // Path settings
    let defaultVideoDir = $state('');
    let defaultImageDir = $state('');
    let defaultOutputDir = $state('');
    let annotationDir = $state('');

    let saveStatus = $state('');

    $effect(() => {
        const config = appConfig.current;
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
            const config = appConfig.current;
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
                    case 'video': defaultVideoDir = selected; break;
                    case 'image': defaultImageDir = selected; break;
                    case 'output': defaultOutputDir = selected; break;
                    case 'annotation': annotationDir = selected; break;
                    case 'hls': hlsOutputDir = selected; break;
                }
            }
        } catch (err) {
            console.error('Failed to open directory picker:', err);
        }
    }

    const comingSoonColors: Record<string, string> = {
        'coming soon': 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20',
    };

    const inputClass = 'bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#137fec]';
    const browseClass = 'px-3 py-2 bg-slate-100 dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-slate-300 rounded-lg hover:bg-slate-200 dark:hover:bg-[#242d38] transition-colors';
</script>

<svelte:head>
    <title>Settings | Media Core</title>
</svelte:head>

<PageContent>
    <!-- Theme Settings -->
    <Panel title="Theme" icon="palette">
        <div class="p-4">
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                <button
                    onclick={() => theme.set('light')}
                    class="relative flex flex-col items-center gap-2 p-3 rounded-lg border-2 transition-all {theme.value === 'light' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
                >
                    <div class="w-full aspect-video bg-[#f6f7f8] rounded-lg border border-slate-200 shadow-sm flex items-center justify-center overflow-hidden">
                        <div class="w-3/4 h-3/4 bg-white rounded shadow-sm flex flex-col gap-2 p-2">
                            <div class="w-1/3 h-2 bg-slate-200 rounded"></div>
                            <div class="w-full h-16 bg-slate-100 rounded"></div>
                        </div>
                    </div>
                    <div class="flex items-center gap-1.5 text-xs">
                        <span class="material-symbols-outlined text-[16px] text-slate-500">light_mode</span>
                        <span class="font-medium text-slate-900 dark:text-white">Light</span>
                    </div>
                    {#if theme.value === 'light'}
                        <div class="absolute top-2 right-2 text-[#137fec]"><span class="material-symbols-outlined text-[18px] filled">check_circle</span></div>
                    {/if}
                </button>

                <button
                    onclick={() => theme.set('dark')}
                    class="relative flex flex-col items-center gap-2 p-3 rounded-lg border-2 transition-all {theme.value === 'dark' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
                >
                    <div class="w-full aspect-video bg-[#101922] rounded-lg border border-slate-700 shadow-sm flex items-center justify-center overflow-hidden">
                        <div class="w-3/4 h-3/4 bg-[#1a222c] rounded shadow-sm flex flex-col gap-2 p-2">
                            <div class="w-1/3 h-2 bg-slate-700 rounded"></div>
                            <div class="w-full h-16 bg-slate-800 rounded"></div>
                        </div>
                    </div>
                    <div class="flex items-center gap-1.5 text-xs">
                        <span class="material-symbols-outlined text-[16px] text-slate-500">dark_mode</span>
                        <span class="font-medium text-slate-900 dark:text-white">Dark</span>
                    </div>
                    {#if theme.value === 'dark'}
                        <div class="absolute top-2 right-2 text-[#137fec]"><span class="material-symbols-outlined text-[18px] filled">check_circle</span></div>
                    {/if}
                </button>

                <button
                    onclick={() => theme.set('auto')}
                    class="relative flex flex-col items-center gap-2 p-3 rounded-lg border-2 transition-all {theme.value === 'auto' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
                >
                    <div class="w-full aspect-video bg-gradient-to-br from-[#f6f7f8] to-[#101922] rounded-lg border border-slate-200 dark:border-[#2a3441] shadow-sm flex items-center justify-center">
                        <span class="material-symbols-outlined text-3xl text-slate-400">brightness_auto</span>
                    </div>
                    <div class="flex items-center gap-1.5 text-xs">
                        <span class="material-symbols-outlined text-[16px] text-slate-500">settings_brightness</span>
                        <span class="font-medium text-slate-900 dark:text-white">Auto</span>
                    </div>
                    {#if theme.value === 'auto'}
                        <div class="absolute top-2 right-2 text-[#137fec]"><span class="material-symbols-outlined text-[18px] filled">check_circle</span></div>
                    {/if}
                </button>
            </div>

            <div class="mt-3 flex items-center gap-2 text-xs text-slate-500 bg-slate-50 dark:bg-[#1a222c] p-2 rounded-lg border border-slate-100 dark:border-[#2a3441]">
                <span class="material-symbols-outlined text-[16px]">info</span>
                <p>
                    Current: <span class="font-bold text-slate-900 dark:text-white uppercase">{effectiveTheme.value}</span>
                    {#if theme.value === 'auto'} (system){/if}
                </p>
            </div>
        </div>
    </Panel>

    <!-- Stream Settings -->
    <Panel title="Stream Settings" icon="stream">
        {#snippet actions()}
            <button onclick={saveStreamSettings} class="text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400">Save</button>
        {/snippet}
        <div class="p-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div class="flex flex-col gap-1">
                    <label for="hlsServerUrl" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">HLS Server URL</label>
                    <input id="hlsServerUrl" type="text" bind:value={hlsServerUrl} placeholder="http://127.0.0.1" class={inputClass} />
                </div>
                <div class="flex flex-col gap-1">
                    <label for="hlsServerPort" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">HLS Server Port</label>
                    <input id="hlsServerPort" type="number" bind:value={hlsServerPort} placeholder="1521" class={inputClass} />
                </div>
                <div class="flex flex-col gap-1 md:col-span-2">
                    <label for="defaultRtspUrl" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Default RTSP URL</label>
                    <input id="defaultRtspUrl" type="text" bind:value={defaultRtspUrl} placeholder="rtsp://localhost:8554/mystream" class={inputClass} />
                </div>
                <div class="flex flex-col gap-1 md:col-span-2">
                    <label for="hlsOutputDir" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">HLS Output Directory</label>
                    <div class="flex gap-2">
                        <input id="hlsOutputDir" type="text" bind:value={hlsOutputDir} placeholder="hls_output" class="{inputClass} flex-1" />
                        <button onclick={() => browseDirectory('hls')} class={browseClass}>
                            <span class="material-symbols-outlined text-[18px]">folder_open</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </Panel>

    <!-- Path Settings -->
    <Panel title="Default Paths" icon="folder">
        {#snippet actions()}
            <button onclick={savePathSettings} class="text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400">Save</button>
        {/snippet}
        <div class="p-4 flex flex-col gap-3">
            <div class="flex flex-col gap-1">
                <label for="defaultVideoDir" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Video Directory</label>
                <div class="flex gap-2">
                    <input id="defaultVideoDir" type="text" bind:value={defaultVideoDir} placeholder="Select default video directory..." class="{inputClass} flex-1" />
                    <button onclick={() => browseDirectory('video')} class={browseClass}><span class="material-symbols-outlined text-[18px]">folder_open</span></button>
                </div>
            </div>
            <div class="flex flex-col gap-1">
                <label for="defaultImageDir" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Image Directory</label>
                <div class="flex gap-2">
                    <input id="defaultImageDir" type="text" bind:value={defaultImageDir} placeholder="Select default image directory..." class="{inputClass} flex-1" />
                    <button onclick={() => browseDirectory('image')} class={browseClass}><span class="material-symbols-outlined text-[18px]">folder_open</span></button>
                </div>
            </div>
            <div class="flex flex-col gap-1">
                <label for="defaultOutputDir" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Output Directory</label>
                <div class="flex gap-2">
                    <input id="defaultOutputDir" type="text" bind:value={defaultOutputDir} placeholder="Select default output directory..." class="{inputClass} flex-1" />
                    <button onclick={() => browseDirectory('output')} class={browseClass}><span class="material-symbols-outlined text-[18px]">folder_open</span></button>
                </div>
            </div>
            <div class="flex flex-col gap-1">
                <label for="annotationDir" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Annotation Directory</label>
                <div class="flex gap-2">
                    <input id="annotationDir" type="text" bind:value={annotationDir} placeholder="Select annotation directory..." class="{inputClass} flex-1" />
                    <button onclick={() => browseDirectory('annotation')} class={browseClass}><span class="material-symbols-outlined text-[18px]">folder_open</span></button>
                </div>
            </div>
        </div>
    </Panel>

    <!-- Language Settings -->
    <Panel title="Language" icon="translate">
        {#snippet actions()}
            <StatusBadge status="coming soon" colorMap={comingSoonColors} />
        {/snippet}
        <div class="p-4 opacity-60">
            <div class="max-w-md">
                <div class="relative">
                    <select class="{inputClass} w-full appearance-none pr-10 disabled:opacity-60 disabled:cursor-not-allowed" bind:value={selectedLanguage} disabled>
                        {#each languages as lang}
                            <option value={lang.code}>{lang.name}</option>
                        {/each}
                    </select>
                    <div class="absolute inset-y-0 right-0 flex items-center px-3 pointer-events-none text-slate-500">
                        <span class="material-symbols-outlined text-[18px]">expand_more</span>
                    </div>
                </div>
                <p class="mt-2 text-[10px] text-slate-400">Language switching is under development.</p>
            </div>
        </div>
    </Panel>

    <!-- Reset & Status -->
    <div class="flex items-center justify-between">
        <button onclick={resetToDefaults} class="px-3 py-1.5 text-xs font-medium bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20 rounded hover:bg-red-500/20 transition-colors">
            Reset All to Defaults
        </button>
        {#if saveStatus}
            <div class="flex items-center gap-1.5 text-xs text-green-600 dark:text-green-400">
                <span class="material-symbols-outlined text-[16px]">check_circle</span>
                <span>{saveStatus}</span>
            </div>
        {/if}
    </div>
</PageContent>
