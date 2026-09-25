#!/usr/bin/env bash
# Round 2: variant colors and reordered class patterns that round 1 missed.
# Applies additional consolidations. Idempotent.
#
# Note: macOS sed does not support PCRE lookaround, so ordering is
# crucial — longest / most-specific patterns first so shorter ones
# don't eat substrings that belong to a bigger token.

set -uo pipefail
cd "$(dirname "$0")/.."

FILES=$(find src -type f -name '*.svelte' \
  ! -path 'src/routes/demo-360/*' \
  ! -path 'src/lib/components/features/demo360/*')

SEDI=(sed -i '')

replace() {
  local pattern="$1"
  local token="$2"
  for f in $FILES; do
    "${SEDI[@]}" -E "s|${pattern}|${token}|g" "$f"
  done
}

# ---- text-stat-label variant colors -----------------------
replace 'text-\[10px\] font-bold uppercase tracking-wider text-slate-400' 'text-stat-label'

# ---- text-caption (run canonical WITH dark: first, then bare) ----
replace 'text-xs text-slate-500 dark:text-slate-400' 'text-caption'
replace 'text-xs text-slate-500' 'text-caption'

# ---- text-body (canonical with dark) ----------------------
replace 'text-xs text-slate-700 dark:text-slate-300' 'text-body'

# ---- text-meta variant colors (slate-400 consolidates to token default slate-500) ----
replace 'text-\[10px\] text-slate-400 font-mono truncate' 'text-meta truncate'
replace 'text-\[10px\] text-slate-400 truncate font-mono' 'text-meta truncate'
replace 'text-\[10px\] font-mono text-slate-400' 'text-meta'
replace 'text-\[10px\] text-slate-400 font-mono' 'text-meta'

# ---- text-muted (color pair) ------------------------------
replace 'text-slate-600 dark:text-slate-400' 'text-muted'

echo "--- Round 2 residual check (should be 0 or near 0):"
for pattern in \
  'text-xs text-slate-500' \
  'text-xs text-slate-700 dark:text-slate-300' \
  'text-\[10px\] text-slate-400 font-mono' \
  'text-slate-600 dark:text-slate-400'; do
  count=$(grep -rEn "$pattern" src --include='*.svelte' 2>/dev/null | grep -v 'demo-?360' | wc -l | tr -d ' ')
  printf '  %-50s %s\n' "$pattern" "$count"
done
echo "done."
