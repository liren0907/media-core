<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
  }

  let { checked = $bindable(), label }: Props = $props();
</script>

<label class="flex items-center gap-2 cursor-pointer">
  <div class="relative inline-flex items-center">
    <input type="checkbox" bind:checked class="sr-only peer">
    <div class="w-9 h-5 bg-slate-200 rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[#137fec]"></div>
  </div>
  <span class="text-xs text-slate-700 dark:text-slate-300">{label}</span>
</label>
