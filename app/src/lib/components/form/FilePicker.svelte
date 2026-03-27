<script lang="ts">
  import { FormField } from '$lib/components/ui';
  import { selectFile } from '$lib/utils/file-dialog';
  import { getFileName } from '$lib/utils/format';
  import { inputClass, browseClass } from '$lib/utils/styles';

  interface Props {
    value: string;
    label: string;
    filters: { name: string; extensions: string[] }[];
    placeholder?: string;
  }

  let { value = $bindable(), label, filters, placeholder = 'Select...' }: Props = $props();

  async function browse() {
    const result = await selectFile(filters);
    if (result) value = result;
  }
</script>

<FormField {label}>
  <div class="flex gap-1.5">
    <input type="text" readonly value={value ? getFileName(value) : ''} class="{inputClass} flex-1" {placeholder} />
    <button onclick={browse} class={browseClass}>Browse</button>
  </div>
</FormField>
