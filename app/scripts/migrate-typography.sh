#!/usr/bin/env bash
# Typography token migration — bulk sed pass for canonical patterns.
# Run once from the repo root. Idempotent: running again is a no-op
# because canonical patterns no longer match after migration.
#
# Scope: every src/**/*.svelte except the demo-360 sandbox.

set -euo pipefail

cd "$(dirname "$0")/.."

# Collect in-scope files
FILES=$(find src -type f -name '*.svelte' \
  ! -path 'src/routes/demo-360/*' \
  ! -path 'src/lib/components/features/demo360/*')

# macOS sed compatibility: use -i ''
SEDI=(sed -i '')

replace() {
  local pattern="$1"
  local token="$2"
  for f in $FILES; do
    "${SEDI[@]}" -E "s|${pattern}|${token}|g" "$f"
  done
}

# ============================================================
# Order: most specific patterns first (longer class strings),
# then shorter or re-ordered variants.
# ============================================================

# ---- text-page-title (text-xl + tracking-tight) -----------
replace 'text-xl font-bold font-display tracking-tight text-slate-900 dark:text-white' 'text-page-title'

# ---- text-stat-value / page-title duplicate (no tracking-tight) ----
# Route to text-stat-value; hand-fix genuine page titles in phase 4b.
replace 'text-xl font-bold font-display text-slate-900 dark:text-white' 'text-stat-value'

# ---- text-section-title ------------------------------------
replace 'text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300' 'text-section-title'
replace 'text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400' 'text-section-title'
replace 'text-xs font-bold text-slate-900 dark:text-white uppercase tracking-wider' 'text-section-title text-slate-900 dark:text-white'
replace 'text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-white' 'text-section-title text-slate-900 dark:text-white'

# ---- text-stat-label ---------------------------------------
# Canonical: text-[10px] font-bold uppercase tracking-widest text-slate-500
replace 'text-\[10px\] font-bold uppercase tracking-widest text-slate-500' 'text-stat-label'
# Variant A: medium + wider (consolidate to canonical stat-label)
replace 'text-\[10px\] font-medium uppercase tracking-wider text-slate-500' 'text-stat-label'
# Variant B: bold + wider
replace 'text-\[10px\] font-bold uppercase tracking-wider text-slate-500' 'text-stat-label'
# Variant C: brand-blue section title (same size as stat-label but colored) — use section-title + brand override
replace 'text-\[10px\] font-bold uppercase tracking-wider text-\[#137fec\]' 'text-stat-label text-status-info'

# ---- text-meta (monospace metadata, 10px, slate-500) ----
replace 'text-\[10px\] font-mono text-slate-500' 'text-meta'
replace 'text-\[10px\] text-slate-500 font-mono' 'text-meta'

# ---- text-code (inline mono, no color baked) ---------------
# Intentionally NOT migrating `text-xs font-mono` because some uses rely on
# inheriting parent color and color-free text-code would change nothing.
# Leaving those as-is is fine. Skipped by design.

# ============================================================
# Report
# ============================================================
echo "--- Post-migration residual canonical patterns (should be 0 or near 0):"
grep -rEn 'text-xs font-bold uppercase tracking-wider text-slate-(500|600|900)' src --include='*.svelte' | grep -v 'demo-?360' | wc -l | awk '{print "  section-title residuals: " $1}'
grep -rEn 'text-\[10px\] font-(bold|medium) uppercase tracking-wid(er|est) text-slate-500' src --include='*.svelte' | grep -v 'demo-?360' | wc -l | awk '{print "  stat-label residuals:    " $1}'
grep -rEn 'text-xl font-bold font-display' src --include='*.svelte' | grep -v 'demo-?360' | wc -l | awk '{print "  page-title/stat-value residuals: " $1}'
grep -rEn 'text-\[10px\] font-mono text-slate-500|text-\[10px\] text-slate-500 font-mono' src --include='*.svelte' | grep -v 'demo-?360' | wc -l | awk '{print "  meta residuals:          " $1}'
echo "done."
