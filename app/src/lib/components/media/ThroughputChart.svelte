<script lang="ts">
  import type { ThroughputHistory } from '$lib/types';

  interface Props {
    throughputHistory: ThroughputHistory | null;
    height?: number;
    gradientId?: string;
  }

  let { throughputHistory, height = 180, gradientId = 'chartGradient' }: Props = $props();

  let currentTime = $state(new Date());

  $effect(() => {
    const interval = setInterval(() => { currentTime = new Date(); }, 60000);
    return () => clearInterval(interval);
  });
</script>

<div class="p-4 relative w-full" style="height: {height}px">
  <svg class="w-full h-full" preserveAspectRatio="none" viewBox="0 0 800 {height}">
    <defs>
      <linearGradient id={gradientId} x1="0" x2="0" y1="0" y2="1">
        <stop offset="0%" stop-color="#137fec" stop-opacity="0.2"></stop>
        <stop offset="100%" stop-color="#137fec" stop-opacity="0"></stop>
      </linearGradient>
    </defs>
    <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="145" y2="145" opacity="0.5"></line>
    <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="100" y2="100" opacity="0.5"></line>
    <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="55" y2="55" opacity="0.5"></line>
    {#if throughputHistory && throughputHistory.points.length > 0}
      {@const points = throughputHistory.points}
      {@const pathData = points.map((p, i) => {
        const x = (i / (points.length - 1)) * 800;
        const y = 160 - (p.networkMbps / 30 * 130);
        return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
      }).join(' ')}
      <path d="{pathData} V {height} H 0 Z" fill="url(#{gradientId})"></path>
      <path d="{pathData}" fill="none" stroke="#137fec" stroke-width="2"></path>
    {:else}
      <path d="M0 140 C 100 125, 200 155, 300 105 S 500 55, 600 85 S 700 40, 800 55 V {height} H 0 Z" fill="url(#{gradientId})"></path>
      <path d="M0 140 C 100 125, 200 155, 300 105 S 500 55, 600 85 S 700 40, 800 55" fill="none" stroke="#137fec" stroke-width="2"></path>
    {/if}
    <path d="M0 130 C 120 135, 250 115, 350 120 S 550 105, 650 100 S 750 115, 800 108" fill="none" opacity="0.5" stroke="#94a3b8" stroke-dasharray="5 5" stroke-width="2"></path>
  </svg>
  <div class="flex justify-between mt-1 text-[10px] text-slate-500 font-mono">
    <span>{new Date(Date.now() - 3600000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
    <span>{new Date(Date.now() - 2700000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
    <span>{new Date(Date.now() - 1800000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
    <span>{new Date(Date.now() - 900000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
    <span>{currentTime.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
  </div>
</div>
