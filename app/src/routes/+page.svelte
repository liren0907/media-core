<script lang="ts">
  // Dashboard Overview
</script>

<svelte:head>
	<title>Media Core Dashboard</title>
</svelte:head>

<div class="max-w-[1400px] mx-auto p-8 flex flex-col gap-6">
    <!-- Quick Actions & Top Row -->
    <div class="flex justify-end gap-3">
        <a href="/stream" class="flex items-center gap-2 px-4 py-2 bg-[#137fec] hover:bg-blue-600 text-white text-sm font-medium rounded-lg transition-colors">
            <span class="material-symbols-outlined text-[20px]">add_circle</span>
            Add Stream
        </a>
        <button class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#283039] text-sm font-medium rounded-lg transition-colors">
            <span class="material-symbols-outlined text-[20px]">download</span>
            Export Report
        </button>
    </div>

    <!-- Stats Grid -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- CPU Card -->
        <div class="flex flex-col gap-4 rounded-xl p-6 bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] shadow-sm">
            <div class="flex justify-between items-start">
                <div class="flex items-center gap-2 text-slate-500 dark:text-slate-400">
                    <span class="material-symbols-outlined">memory</span>
                    <h3 class="text-sm font-medium uppercase tracking-wider font-display">CPU Load</h3>
                </div>
                <span class="text-green-500 bg-green-500/10 px-2 py-0.5 rounded text-xs font-bold">+2.4%</span>
            </div>
            <div class="flex items-baseline gap-2">
                <p class="text-4xl font-bold text-slate-900 dark:text-white font-display">45%</p>
                <span class="text-xs text-slate-500">core avg</span>
            </div>
            <!-- Micro Chart -->
            <div class="h-12 w-full flex items-end gap-1 mt-2">
                <div class="bg-[#137fec]/20 hover:bg-[#137fec]/40 w-full h-[40%] rounded-sm transition-all"></div>
                <div class="bg-[#137fec]/20 hover:bg-[#137fec]/40 w-full h-[60%] rounded-sm transition-all"></div>
                <div class="bg-[#137fec]/20 hover:bg-[#137fec]/40 w-full h-[45%] rounded-sm transition-all"></div>
                <div class="bg-[#137fec]/20 hover:bg-[#137fec]/40 w-full h-[30%] rounded-sm transition-all"></div>
                <div class="bg-[#137fec]/20 hover:bg-[#137fec]/40 w-full h-[70%] rounded-sm transition-all"></div>
                <div class="bg-[#137fec] hover:bg-[#137fec]/80 w-full h-[45%] rounded-sm transition-all"></div>
                <div class="bg-[#137fec]/20 hover:bg-[#137fec]/40 w-full h-[55%] rounded-sm transition-all"></div>
                <div class="bg-[#137fec]/20 hover:bg-[#137fec]/40 w-full h-[35%] rounded-sm transition-all"></div>
            </div>
        </div>

        <!-- GPU Card -->
        <div class="flex flex-col gap-4 rounded-xl p-6 bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] shadow-sm relative overflow-hidden">
            <div class="absolute top-0 right-0 w-16 h-16 bg-orange-500/10 rounded-bl-full -mr-4 -mt-4"></div>
            <div class="flex justify-between items-start relative z-10">
                <div class="flex items-center gap-2 text-slate-500 dark:text-slate-400">
                    <span class="material-symbols-outlined">developer_board</span>
                    <h3 class="text-sm font-medium uppercase tracking-wider font-display">GPU Usage</h3>
                </div>
                <span class="text-orange-500 bg-orange-500/10 px-2 py-0.5 rounded text-xs font-bold">High Load</span>
            </div>
            <div class="flex items-baseline gap-2 relative z-10">
                <p class="text-4xl font-bold text-slate-900 dark:text-white font-display">82%</p>
                <span class="text-xs text-slate-500">compute</span>
            </div>
            <div class="w-full bg-slate-200 dark:bg-slate-700 rounded-full h-1.5 mt-auto">
                <div class="bg-orange-500 h-1.5 rounded-full" style="width: 82%"></div>
            </div>
        </div>

        <!-- Memory Card -->
        <div class="flex flex-col gap-4 rounded-xl p-6 bg-white dark:bg-[#161e27] border border-slate-200 dark:border-[#2a3441] shadow-sm">
            <div class="flex justify-between items-start">
                <div class="flex items-center gap-2 text-slate-500 dark:text-slate-400">
                    <span class="material-symbols-outlined">storage</span>
                    <h3 class="text-sm font-medium uppercase tracking-wider font-display">Memory</h3>
                </div>
                <span class="text-slate-400 bg-slate-500/10 px-2 py-0.5 rounded text-xs font-bold">Stable</span>
            </div>
            <div class="flex items-baseline gap-2">
                <p class="text-4xl font-bold text-slate-900 dark:text-white font-display">12.4<span class="text-xl ml-1 font-normal text-slate-500">GB</span></p>
                <span class="text-xs text-slate-500">/ 32 GB</span>
            </div>
            <div class="flex gap-2 mt-auto">
                <div class="h-1.5 flex-1 bg-[#137fec] rounded-full"></div>
                <div class="h-1.5 flex-1 bg-[#137fec]/40 rounded-full"></div>
                <div class="h-1.5 flex-1 bg-slate-200 dark:bg-slate-700 rounded-full"></div>
            </div>
        </div>
    </div>

    <!-- Main Chart Section -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- Large Chart -->
        <div class="lg:col-span-2 rounded-xl border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] p-6 shadow-sm">
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h3 class="text-base font-bold text-slate-900 dark:text-white font-display">Network & Processing Throughput</h3>
                    <p class="text-slate-500 text-sm">Real-time bandwidth and FPS metrics</p>
                </div>
                <div class="flex items-center gap-2">
                    <span class="flex items-center gap-1.5 text-xs text-slate-500">
                        <span class="size-2 rounded-full bg-[#137fec]"></span> Network
                    </span>
                    <span class="flex items-center gap-1.5 text-xs text-slate-500">
                        <span class="size-2 rounded-full bg-slate-400"></span> FPS
                    </span>
                    <select class="ml-2 bg-transparent text-xs text-slate-500 border-none outline-none cursor-pointer hover:text-slate-700 dark:hover:text-white focus:ring-0">
                        <option>Last Hour</option>
                        <option>Last 24h</option>
                    </select>
                </div>
            </div>
            <div class="relative w-full h-[250px]">
                <!-- Chart SVG -->
                <svg class="w-full h-full" preserveAspectRatio="none" viewBox="0 0 800 250">
                    <defs>
                        <linearGradient id="chartGradient" x1="0" x2="0" y1="0" y2="1">
                            <stop offset="0%" stop-color="#137fec" stop-opacity="0.2"></stop>
                            <stop offset="100%" stop-color="#137fec" stop-opacity="0"></stop>
                        </linearGradient>
                    </defs>
                    <!-- Grid Lines -->
                    <line stroke="#2a3441" stroke-dasharray="4 4" stroke-width="1" x1="0" x2="800" y1="200" y2="200"></line>
                    <line stroke="#2a3441" stroke-dasharray="4 4" stroke-width="1" x1="0" x2="800" y1="150" y2="150"></line>
                    <line stroke="#2a3441" stroke-dasharray="4 4" stroke-width="1" x1="0" x2="800" y1="100" y2="100"></line>
                    <line stroke="#2a3441" stroke-dasharray="4 4" stroke-width="1" x1="0" x2="800" y1="50" y2="50"></line>
                    <!-- Primary Line (Area) -->
                    <path d="M0 200 C 100 180, 200 220, 300 150 S 500 80, 600 120 S 700 60, 800 80 V 250 H 0 Z" fill="url(#chartGradient)"></path>
                    <path d="M0 200 C 100 180, 200 220, 300 150 S 500 80, 600 120 S 700 60, 800 80" fill="none" stroke="#137fec" stroke-width="2"></path>
                    <!-- Secondary Line -->
                    <path d="M0 180 C 120 190, 250 160, 350 170 S 550 150, 650 140 S 750 160, 800 150" fill="none" opacity="0.5" stroke="#94a3b8" stroke-dasharray="5 5" stroke-width="2"></path>
                </svg>
                <!-- X Axis Labels -->
                <div class="flex justify-between mt-2 text-xs text-slate-500 font-mono">
                    <span>10:00</span>
                    <span>10:15</span>
                    <span>10:30</span>
                    <span>10:45</span>
                    <span>11:00</span>
                </div>
            </div>
        </div>

        <!-- System Health & Quick Stats -->
        <div class="lg:col-span-1 flex flex-col gap-4">
            <div class="flex-1 rounded-xl border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] p-6 shadow-sm">
                <h3 class="text-base font-bold text-slate-900 dark:text-white font-display mb-4">Storage Status</h3>
                <div class="flex items-center justify-center py-4">
                    <div class="relative size-32">
                        <svg class="size-full rotate-[-90deg]" viewBox="0 0 36 36">
                            <!-- Background Circle -->
                            <path class="text-slate-200 dark:text-slate-700" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="currentColor" stroke-width="3"></path>
                            <!-- Value Circle -->
                            <path class="text-[#137fec]" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="currentColor" stroke-dasharray="75, 100" stroke-width="3"></path>
                        </svg>
                        <div class="absolute inset-0 flex flex-col items-center justify-center text-slate-900 dark:text-white">
                            <span class="text-2xl font-bold font-display">75%</span>
                            <span class="text-[10px] text-slate-500">USED</span>
                        </div>
                    </div>
                </div>
                <div class="flex flex-col gap-2 mt-2">
                    <div class="flex justify-between text-sm">
                        <span class="text-slate-500">SSD (NVMe)</span>
                        <span class="text-slate-900 dark:text-white font-mono">1.2 TB / 2.0 TB</span>
                    </div>
                    <div class="flex justify-between text-sm">
                        <span class="text-slate-500">Throughput</span>
                        <span class="text-slate-900 dark:text-white font-mono">450 MB/s</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Active Tasks Table -->
    <div class="rounded-xl border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] shadow-sm overflow-hidden">
        <div class="px-6 py-4 border-b border-slate-200 dark:border-[#2a3441] flex justify-between items-center">
            <h3 class="text-base font-bold text-slate-900 dark:text-white font-display">Active Processes</h3>
            <button class="text-xs font-medium text-[#137fec] hover:text-blue-400">View All</button>
        </div>
        <div class="overflow-x-auto">
            <table class="w-full text-left text-sm">
                <thead class="bg-slate-50 dark:bg-[#1f2937] text-slate-500 font-medium">
                    <tr>
                        <th class="px-6 py-3">Stream ID</th>
                        <th class="px-6 py-3">Type</th>
                        <th class="px-6 py-3">Status</th>
                        <th class="px-6 py-3">Codec</th>
                        <th class="px-6 py-3">Duration</th>
                        <th class="px-6 py-3 text-right">Actions</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-200 dark:divide-[#2a3441]">
                    <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                        <td class="px-6 py-4 font-mono text-slate-900 dark:text-white">stream_01_alpha</td>
                        <td class="px-6 py-4 text-slate-600 dark:text-slate-300">Live Encoding</td>
                        <td class="px-6 py-4">
                            <span class="inline-flex items-center gap-1.5 rounded px-2 py-1 text-xs font-medium bg-green-500/10 text-green-600 dark:text-green-400 border border-green-500/20">
                                <span class="size-1.5 rounded-full bg-green-500 animate-pulse"></span>
                                Processing
                            </span>
                        </td>
                        <td class="px-6 py-4 text-slate-500">H.264 / AAC</td>
                        <td class="px-6 py-4 text-slate-500 font-mono">02:14:55</td>
                        <td class="px-6 py-4 text-right">
                            <button class="text-slate-400 hover:text-red-500 transition-colors"><span class="material-symbols-outlined text-lg">stop_circle</span></button>
                            <button class="text-slate-400 hover:text-blue-500 transition-colors ml-2"><span class="material-symbols-outlined text-lg">restart_alt</span></button>
                        </td>
                    </tr>
                    <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                        <td class="px-6 py-4 font-mono text-slate-900 dark:text-white">cam_feed_north</td>
                        <td class="px-6 py-4 text-slate-600 dark:text-slate-300">Object Detection</td>
                        <td class="px-6 py-4">
                            <span class="inline-flex items-center gap-1.5 rounded px-2 py-1 text-xs font-medium bg-green-500/10 text-green-600 dark:text-green-400 border border-green-500/20">
                                <span class="size-1.5 rounded-full bg-green-500 animate-pulse"></span>
                                Processing
                            </span>
                        </td>
                        <td class="px-6 py-4 text-slate-500">YOLOv8</td>
                        <td class="px-6 py-4 text-slate-500 font-mono">14:22:01</td>
                        <td class="px-6 py-4 text-right">
                             <button class="text-slate-400 hover:text-red-500 transition-colors"><span class="material-symbols-outlined text-lg">stop_circle</span></button>
                             <button class="text-slate-400 hover:text-blue-500 transition-colors ml-2"><span class="material-symbols-outlined text-lg">restart_alt</span></button>
                        </td>
                    </tr>
                    <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                        <td class="px-6 py-4 font-mono text-slate-900 dark:text-white">archive_export_04</td>
                        <td class="px-6 py-4 text-slate-600 dark:text-slate-300">Transcoding</td>
                        <td class="px-6 py-4">
                            <span class="inline-flex items-center gap-1.5 rounded px-2 py-1 text-xs font-medium bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20">
                                <span class="size-1.5 rounded-full bg-red-500"></span>
                                Error
                            </span>
                        </td>
                        <td class="px-6 py-4 text-slate-500">VP9</td>
                        <td class="px-6 py-4 text-slate-500 font-mono">--:--:--</td>
                        <td class="px-6 py-4 text-right">
                             <button class="text-slate-400 hover:text-red-500 transition-colors"><span class="material-symbols-outlined text-lg">stop_circle</span></button>
                             <button class="text-slate-400 hover:text-blue-500 transition-colors ml-2"><span class="material-symbols-outlined text-lg">restart_alt</span></button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</div>
