<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { theme, effectiveTheme } from '$lib/theme.svelte';
    import { appConfig, type StreamConfig, type PathConfig } from '$lib/config.svelte';
    import { PageContent, Panel, StatusBadge, ToggleSwitch, FormField } from '$lib/components/ui';
    import { DirPicker } from '$lib/components/form';
    import type { HardwareAccelConfig, HardwareCapabilities } from '$lib/types';

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

    // Hardware acceleration state
    let hwConfig = $state<HardwareAccelConfig>({ enabled: false, mode: 'auto', fallbackToCpu: true, preferBackends: [] });
    let hwCapabilities = $state<HardwareCapabilities | null>(null);

    async function loadHardwareInfo() {
        try {
            const [config, caps] = await Promise.all([
                invoke('get_hardware_accel_config') as Promise<HardwareAccelConfig>,
                invoke('detect_hardware_capabilities') as Promise<HardwareCapabilities>,
            ]);
            hwConfig = config;
            hwCapabilities = caps;
        } catch (e) {
            console.error('Failed to load hardware info:', e);
        }
    }

    // Load hardware info on mount (add to existing effect)
    $effect(() => { loadHardwareInfo(); });

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
                <FormField label="HLS Server URL" id="hlsServerUrl">
                    <input id="hlsServerUrl" type="text" bind:value={hlsServerUrl} placeholder="http://127.0.0.1" class={inputClass} />
                </FormField>
                <FormField label="HLS Server Port" id="hlsServerPort">
                    <input id="hlsServerPort" type="number" bind:value={hlsServerPort} placeholder="1521" class={inputClass} />
                </FormField>
                <div class="md:col-span-2">
                    <FormField label="Default RTSP URL" id="defaultRtspUrl">
                        <input id="defaultRtspUrl" type="text" bind:value={defaultRtspUrl} placeholder="rtsp://localhost:8554/mystream" class={inputClass} />
                    </FormField>
                </div>
                <div class="md:col-span-2">
                    <DirPicker bind:value={hlsOutputDir} label="HLS Output Directory" placeholder="hls_output" />
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
            <DirPicker bind:value={defaultVideoDir} label="Video Directory" placeholder="Select default video directory..." />
            <DirPicker bind:value={defaultImageDir} label="Image Directory" placeholder="Select default image directory..." />
            <DirPicker bind:value={defaultOutputDir} label="Output Directory" placeholder="Select default output directory..." />
            <DirPicker bind:value={annotationDir} label="Annotation Directory" placeholder="Select default annotation directory..." />
        </div>
    </Panel>

    <!-- Language Settings -->
    <!-- Hardware Acceleration -->
    <Panel title="Hardware Acceleration" icon="bolt">
        <div class="p-4 flex flex-col gap-3">
            {#if hwCapabilities}
                <div class="flex items-center gap-2 text-xs text-slate-500 bg-slate-50 dark:bg-[#1a222c] p-2 rounded border border-slate-100 dark:border-[#2a3441]">
                    <span class="material-symbols-outlined text-[16px]">info</span>
                    <span>Platform: <span class="font-bold text-slate-900 dark:text-white">{hwCapabilities.platform}</span></span>
                    {#if hwCapabilities.isAppleSilicon}
                        <StatusBadge status="Apple Silicon" colorMap={{ 'Apple Silicon': 'bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-500/20' }} />
                    {/if}
                </div>
            {/if}

            <ToggleSwitch bind:checked={hwConfig.enabled} label="Enable Hardware Acceleration" />

            {#if hwConfig.enabled}
                <FormField label="Mode" id="hwMode">
                    <select id="hwMode" bind:value={hwConfig.mode} class="{inputClass} w-full">
                        {#if hwCapabilities}
                            {#each hwCapabilities.availableModes as mode}
                                <option value={mode}>{mode === 'auto' ? 'Auto Detect' : mode === 'apple_silicon' ? 'Apple Silicon (VideoToolbox)' : mode === 'cuda' ? 'NVIDIA CUDA' : mode === 'disabled' ? 'Disabled (CPU)' : mode}</option>
                            {/each}
                        {:else}
                            <option value="auto">Auto Detect</option>
                            <option value="disabled">Disabled</option>
                        {/if}
                    </select>
                </FormField>

                <ToggleSwitch bind:checked={hwConfig.fallbackToCpu} label="Fallback to CPU if acceleration fails" />
            {/if}
        </div>
    </Panel>

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
