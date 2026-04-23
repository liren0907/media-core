#!/usr/bin/env bash
# Round 3: apply text-nav and text-card-title tokens.
set -uo pipefail
cd "$(dirname "$0")/.."

while IFS= read -r f; do
  sed -i '' -E 's|text-sm font-medium leading-normal font-display|text-nav leading-normal|g' "$f"
  sed -i '' -E 's|text-xs font-bold text-slate-900 dark:text-white|text-card-title|g' "$f"
done < <(find src -type f -name '*.svelte' \
  ! -path 'src/routes/demo-360/*' \
  ! -path 'src/lib/components/features/demo360/*')

echo "done. nav count: $(grep -rn text-nav src --include='*.svelte' | grep -v 'demo-?360' | wc -l | tr -d ' ')"
echo "done. card-title count: $(grep -rn text-card-title src --include='*.svelte' | grep -v 'demo-?360' | wc -l | tr -d ' ')"
