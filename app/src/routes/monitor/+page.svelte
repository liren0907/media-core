<script lang="ts">
  // Dashboard Overview - HSR Command Style Redesign

  let currentTime = $state(new Date());

  $effect(() => {
    const timer = setInterval(() => {
      currentTime = new Date();
    }, 1000);
    return () => clearInterval(timer);
  });

  // Mock Data
  const activeStreams = [
    { id: 'CAM-01', name: 'Server Room A', status: 'REC', type: 'critical' },
    { id: 'CAM-02', name: 'Main Lobby', status: 'LIVE', type: 'normal' },
    { id: 'CAM-03', name: 'Loading Dock', status: 'IDLE', type: 'warning' },
    { id: 'CAM-04', name: 'Perimeter N', status: 'REC', type: 'critical' },
  ];

  const logs = [
    { time: '14:23:05', category: 'SYSTEM', message: 'System routine check complete.', type: 'info' },
    { time: '14:24:12', category: 'SAFETY', message: 'CPU Load increased to 65%.', type: 'warning' },
    { time: '14:25:45', category: 'SECURITY', message: 'New stream connection established.', type: 'critical' },
    { time: '14:26:01', category: 'SAFETY', message: 'Temp check: 45°C (Normal).', type: 'info' },
  ];
</script>

<svelte:head>
	<title>Media Command Center</title>
</svelte:head>

<div class="flex h-[calc(100vh-64px)] overflow-hidden bg-slate-50 dark:bg-[#0d1117] text-slate-800 dark:text-slate-200 font-sans">
    
    <!-- Main Content Area (Left/Center) -->
    <div class="flex-1 flex flex-col min-w-0">
        
        <!-- Command Header -->
        <div class="h-16 px-6 border-b border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] flex items-center justify-between shrink-0">
            <div class="flex items-center gap-4">
                <div class="flex flex-col">
                    <h1 class="text-lg font-bold tracking-tight uppercase font-display text-slate-900 dark:text-white">Media Command</h1>
                    <span class="text-[10px] tracking-widest text-slate-500 uppercase">Surveillance v2.4</span>
                </div>
            </div>
            
            <div class="font-mono text-xl font-medium tracking-wider text-slate-700 dark:text-slate-300">
                {currentTime.toLocaleTimeString('en-GB', { hour12: false })}
            </div>

            <div class="flex items-center gap-4">
                <span class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-green-500/10 border border-green-500/20 text-green-600 dark:text-green-400 text-xs font-bold uppercase tracking-wider">
                    <span class="size-2 rounded-full bg-green-500 animate-pulse"></span>
                    System Normal
                </span>
                <div class="size-8 rounded bg-slate-200 dark:bg-slate-700 flex items-center justify-center text-xs font-bold text-slate-600 dark:text-slate-300">
                    AD
                </div>
            </div>
        </div>

        <!-- Scrollable Content -->
        <div class="flex-1 overflow-auto p-6 flex flex-col gap-6">
            
            <!-- Filter Bar -->
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-2 bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] rounded-lg px-3 py-2 w-64 shadow-sm">
                    <span class="material-symbols-outlined text-slate-400 text-[18px]">search</span>
                    <input type="text" placeholder="Find stream, station..." class="bg-transparent border-none outline-none text-sm w-full placeholder:text-slate-400 text-slate-700 dark:text-slate-200">
                </div>
                <div class="flex items-center gap-2">
                    <button class="px-3 py-1.5 text-xs font-medium bg-[#137fec] text-white rounded shadow-sm hover:bg-blue-600 transition-colors">2x2</button>
                    <button class="px-3 py-1.5 text-xs font-medium bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] text-slate-600 dark:text-slate-300 rounded hover:bg-slate-50 dark:hover:bg-[#283039] transition-colors">Full</button>
                </div>
            </div>

            <!-- Monitor Grid -->
            <div class="grid grid-cols-2 gap-4 flex-1 min-h-[400px]">
                {#each activeStreams as stream}
                    <div class="relative bg-black rounded-lg overflow-hidden border border-slate-800 shadow-sm group">
                        <!-- Placeholder Video Content -->
                        <div class="absolute inset-0 flex items-center justify-center">
                            <span class="material-symbols-outlined text-slate-700 text-[64px] opacity-50 group-hover:opacity-80 transition-opacity">videocam_off</span>
                        </div>
                        
                        <!-- Overlay UI -->
                        <div class="absolute inset-0 p-4 flex flex-col justify-between">
                            <div class="flex justify-between items-start">
                                <div>
                                    <div class="flex items-center gap-2">
                                        <span class="text-white font-bold text-sm tracking-wide">{stream.name}</span>
                                        {#if stream.status === 'REC'}
                                            <span class="px-1.5 py-0.5 bg-red-600 text-white text-[10px] font-bold rounded">REC</span>
                                        {/if}
                                    </div>
                                    <div class="text-slate-400 text-xs font-mono mt-0.5">{stream.id}</div>
                                </div>
                                <div class="size-2 rounded-full {stream.status === 'IDLE' ? 'bg-slate-500' : 'bg-green-500'} shadow-[0_0_8px_rgba(34,197,94,0.6)]"></div>
                            </div>
                            
                            <!-- Bounding Box Simulation -->
                            {#if stream.id === 'CAM-04'}
                                <div class="absolute top-1/2 left-1/3 w-32 h-48 border-2 border-yellow-500/80 rounded-sm">
                                    <div class="absolute -top-5 left-0 bg-yellow-500/80 text-black text-[10px] font-bold px-1">Person (98%)</div>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>

            <!-- Live Detection Log -->
            <div class="bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] rounded-lg overflow-hidden shadow-sm shrink-0">
                <div class="px-4 py-2 border-b border-slate-200 dark:border-[#2a3441] flex items-center justify-between bg-slate-50 dark:bg-[#1f2937]/50">
                    <div class="flex items-center gap-2">
                        <span class="material-symbols-outlined text-slate-500 text-[18px]">list_alt</span>
                        <h3 class="text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300">Live Detection Log</h3>
                    </div>
                    <div class="flex items-center gap-2 text-[10px]">
                        <span class="flex items-center gap-1 text-red-500 font-medium"><span class="size-1.5 rounded-full bg-red-500"></span> Critical</span>
                        <span class="flex items-center gap-1 text-orange-500 font-medium"><span class="size-1.5 rounded-full bg-orange-500"></span> Warning</span>
                    </div>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-left text-xs font-mono">
                        <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                            <tr>
                                <th class="px-4 py-2 font-medium w-24">TIME</th>
                                <th class="px-4 py-2 font-medium w-32">CATEGORY</th>
                                <th class="px-4 py-2 font-medium">EVENT MESSAGE</th>
                                <th class="px-4 py-2 font-medium text-right w-24">CAM ID</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                            {#each logs as log}
                                <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                                    <td class="px-4 py-2 text-slate-600 dark:text-slate-400">{log.time}</td>
                                    <td class="px-4 py-2">
                                        <span class="px-1.5 py-0.5 rounded text-[10px] font-bold border 
                                            {log.category === 'SECURITY' ? 'bg-orange-500/10 text-orange-600 border-orange-500/20' : 
                                             log.category === 'SYSTEM' ? 'bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300 border-slate-300' :
                                             'bg-blue-500/10 text-blue-600 border-blue-500/20'}">
                                            {log.category}
                                        </span>
                                    </td>
                                    <td class="px-4 py-2 text-slate-700 dark:text-slate-300">{log.message}</td>
                                    <td class="px-4 py-2 text-right text-slate-400">cam-805</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            </div>

        </div>
    </div>

    <!-- Telemetry Panel (Right) -->
    <div class="w-[320px] bg-white dark:bg-[#161e27] border-l border-slate-200 dark:border-[#2a3441] flex flex-col shrink-0">
        
        <!-- Tab Switcher -->
        <div class="flex border-b border-slate-200 dark:border-[#2a3441]">
            <button class="flex-1 py-3 text-xs font-bold uppercase tracking-wider text-[#137fec] border-b-2 border-[#137fec]">Telemetry</button>
            <button class="flex-1 py-3 text-xs font-bold uppercase tracking-wider text-slate-400 hover:text-slate-600 dark:hover:text-slate-300">Raw Data</button>
        </div>

        <div class="p-6 flex flex-col gap-6 overflow-auto">
            
            <!-- CPU/Crowd Occupancy Style Card -->
            <div class="bg-white dark:bg-[#1f2937] rounded-xl border border-slate-100 dark:border-[#2a3441] p-6 shadow-sm flex flex-col items-center relative">
                <h3 class="text-[10px] font-bold uppercase tracking-widest text-slate-400 mb-4 self-start">System Load</h3>
                
                <div class="relative size-32">
                    <svg class="size-full rotate-[-90deg]" viewBox="0 0 36 36">
                        <path class="text-slate-100 dark:text-slate-700" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="currentColor" stroke-width="3"></path>
                        <path class="text-[#137fec]" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="currentColor" stroke-dasharray="65, 100" stroke-width="3"></path>
                    </svg>
                    <div class="absolute inset-0 flex flex-col items-center justify-center text-slate-900 dark:text-white">
                        <span class="text-3xl font-bold font-display">65%</span>
                    </div>
                </div>

                <div class="absolute bottom-6 right-6 px-2 py-0.5 bg-green-500/10 text-green-600 dark:text-green-400 text-[10px] font-bold rounded">Level: Normal</div>
            </div>

            <!-- Stats Row -->
            <div class="grid grid-cols-2 gap-3">
                <div class="bg-white dark:bg-[#1f2937] rounded-xl border border-slate-100 dark:border-[#2a3441] p-4 shadow-sm">
                    <h4 class="text-[10px] font-bold uppercase tracking-wider text-slate-400 mb-1">Temperature</h4>
                    <p class="text-xl font-bold text-slate-900 dark:text-white font-display">42.5 °C</p>
                </div>
                <div class="bg-white dark:bg-[#1f2937] rounded-xl border border-slate-100 dark:border-[#2a3441] p-4 shadow-sm">
                    <h4 class="text-[10px] font-bold uppercase tracking-wider text-slate-400 mb-1">Noise Level</h4>
                    <p class="text-xl font-bold text-slate-900 dark:text-white font-display">35 dB</p>
                </div>
            </div>

            <!-- Avg Dwell Time / Memory -->
            <div class="bg-white dark:bg-[#1f2937] rounded-xl border border-slate-100 dark:border-[#2a3441] p-4 shadow-sm">
                <h4 class="text-[10px] font-bold uppercase tracking-wider text-slate-400 mb-2">Avg Processing Time</h4>
                <div class="flex items-baseline justify-between">
                    <p class="text-2xl font-bold text-slate-900 dark:text-white font-display">12ms</p>
                    <span class="text-[10px] font-bold text-red-500 flex items-center gap-1">
                        <span class="material-symbols-outlined text-[12px]">arrow_upward</span> 12% vs avg
                    </span>
                </div>
            </div>

            <!-- Anomaly Trend / Network Chart -->
            <div class="bg-white dark:bg-[#1f2937] rounded-xl border border-slate-100 dark:border-[#2a3441] p-4 shadow-sm flex-1">
                <div class="flex justify-between items-center mb-4">
                    <h4 class="text-[10px] font-bold uppercase tracking-wider text-slate-400">Anomaly Trend</h4>
                    <div class="size-2 rounded-full bg-green-500"></div>
                </div>
                
                <div class="flex items-end gap-1 h-24 w-full">
                    <div class="bg-[#137fec]/20 w-full h-[30%] rounded-t-sm"></div>
                    <div class="bg-[#137fec]/20 w-full h-[50%] rounded-t-sm"></div>
                    <div class="bg-[#137fec]/20 w-full h-[20%] rounded-t-sm"></div>
                    <div class="bg-[#137fec]/40 w-full h-[70%] rounded-t-sm"></div>
                    <div class="bg-[#137fec]/20 w-full h-[40%] rounded-t-sm"></div>
                    <div class="bg-[#137fec]/20 w-full h-[30%] rounded-t-sm"></div>
                    <div class="bg-[#137fec]/60 w-full h-[80%] rounded-t-sm"></div>
                    <div class="bg-[#137fec]/20 w-full h-[45%] rounded-t-sm"></div>
                    <div class="bg-[#137fec]/20 w-full h-[25%] rounded-t-sm"></div>
                </div>
            </div>

        </div>
    </div>

</div>