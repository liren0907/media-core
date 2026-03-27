<script lang="ts">
  import { FormField } from '$lib/components/ui';
  import { selectDirectory } from '$lib/utils/file-dialog';
  import { getFileName } from '$lib/utils/format';
  import { inputClass, browseClass } from '$lib/utils/styles';

  interface Props {
    value: string;
    label: string;
    placeholder?: string;
  }

  let { value = $bindable(), label, placeholder = 'Select...' }: Props = $props();

  async function browse() {
    const result = await selectDirectory();
    if (result) value = result;
  }
</script>

<FormField {label}>
  <div class="flex gap-1.5">
    <input type="text" readonly value={value ? getFileName(value) : ''} class="{inputClass} flex-1" {placeholder} />
    <button onclick={browse} class={browseClass}>Browse</button>
  </div>
</FormField>
