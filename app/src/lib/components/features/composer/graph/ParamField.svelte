<script lang="ts">
  /**
   * One parameter, rendered from its manifest declaration.
   *
   * Nothing here knows which method it belongs to — `kind` picks the control,
   * `options` fills the select, `min`/`max` bound the number. A new backend
   * parameter appears with no change to this file.
   */
  import { FormField, Icon } from '$lib/components/ui';
  import { selectDirectory, selectFile } from '$lib/utils/file-dialog';
  import { getFileName } from '$lib/utils/format';
  import { inputClass, browseClass } from '$lib/utils/styles';
  import type { GraphParamSpec } from '$lib/types';

  interface Props {
    spec: GraphParamSpec;
    value: unknown;
    /** An upstream node can supply this parameter if it is left blank. */
    canInherit?: boolean;
    onchange: (value: unknown) => void;
  }

  let { spec, value, canInherit = false, onchange }: Props = $props();

  const text = $derived(value === undefined || value === null ? '' : String(value));
  const placeholder = $derived(
    spec.default !== undefined
      ? `Default: ${spec.default}`
      : canInherit
        ? 'From upstream'
        : spec.required
          ? 'Required'
          : 'Not set'
  );

  async function browse() {
    const picked =
      spec.kind === 'dir' ? await selectDirectory() : await selectFile([{ name: 'All', extensions: ['*'] }]);
    if (picked) onchange(picked);
  }

  function onNumber(raw: string) {
    if (raw.trim() === '') return onchange(undefined);
    const n = Number(raw);
    if (Number.isNaN(n)) return;
    // No rounding: a threshold of 0.9 rounded to an integer becomes 0.
    onchange(spec.kind === 'int' ? Math.trunc(n) : n);
  }
</script>

<FormField label={spec.required ? `${spec.label} *` : spec.label}>
  {#if spec.kind === 'bool'}
    <label class="flex items-center gap-2 cursor-pointer">
      <input
        type="checkbox"
        checked={value === true || (value === undefined && spec.default === true)}
        onchange={(e) => onchange(e.currentTarget.checked)}
        class="accent-[#137fec]"
      />
      <span class="text-xs text-slate-600 dark:text-slate-300">
        {value === true || (value === undefined && spec.default === true) ? 'On' : 'Off'}
      </span>
    </label>
  {:else if spec.kind === 'enum'}
    <select
      value={text || (spec.default ?? '')}
      onchange={(e) => onchange(e.currentTarget.value)}
      class="{inputClass} w-full"
    >
      {#each spec.options ?? [] as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  {:else if spec.kind === 'path' || spec.kind === 'dir'}
    <div class="flex gap-1.5">
      <input
        type="text"
        readonly
        value={text ? getFileName(text) : ''}
        title={text}
        {placeholder}
        class="{inputClass} flex-1 min-w-0"
      />
      {#if text}
        <button
          onclick={() => onchange(undefined)}
          class={browseClass}
          title="Clear"
          aria-label="Clear {spec.label}"
        >
          <Icon name="close" class="text-[14px]" />
        </button>
      {/if}
      <button onclick={browse} class={browseClass}>Browse</button>
    </div>
  {:else if spec.kind === 'int' || spec.kind === 'float'}
    <input
      type="number"
      value={text}
      min={spec.min}
      max={spec.max}
      step={spec.kind === 'int' ? 1 : 'any'}
      {placeholder}
      oninput={(e) => onNumber(e.currentTarget.value)}
      class="{inputClass} w-full"
    />
  {:else}
    <input
      type="text"
      value={text}
      {placeholder}
      oninput={(e) => onchange(e.currentTarget.value)}
      class="{inputClass} w-full"
    />
  {/if}

  {#if spec.hint}
    <p class="text-[10px] text-slate-400 dark:text-slate-500 leading-snug">{spec.hint}</p>
  {/if}
</FormField>
