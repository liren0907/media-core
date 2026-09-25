# Typography Tokens

This project uses **semantic typography classes** defined in
[`src/app.css`](../src/app.css) rather than sprinkling raw Tailwind
size/weight/colour classes across every component. The goal is
single-source-of-truth styling: change a token once and every
semantically-related element updates consistently.

## How It Works

1. **Font family** is wired through Tailwind via CSS variables.
   `tailwind.config.js` declares `fontFamily.{sans,display,mono}` so
   that `font-sans`, `font-display`, and `font-mono` all read from
   `--font-body`, `--font-display`, `--font-mono` in `:root`.

2. **Semantic tokens** live in `@layer components` inside
   `src/app.css`. Because Tailwind's `utilities` layer ships after
   `components` in the cascade, **any Tailwind utility written on the
   same element overrides the token's defaults** — this is intentional
   and how we compose.

3. **Two kinds of tokens**:
   - **Role tokens** bundle size + weight + family + case + tracking +
     default colour for a specific UI role.
   - **Color-only tokens** apply just a colour pairing (light + dark).
     They can be layered on top of a role token.

## Token Catalog

### Role tokens

| Class | Purpose | `@apply` |
|---|---|---|
| `text-page-title` | Page-level heading (h1/h2 in layout) | `text-xl font-bold font-display tracking-tight text-slate-900 dark:text-white` |
| `text-section-title` | Panel / section header (small uppercase caps) | `text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300` |
| `text-card-title` | Card or row title (bold, not uppercase) | `text-xs font-bold text-slate-900 dark:text-white` |
| `text-stat-label` | StatCard label, small uppercase | `text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400` |
| `text-stat-value` | StatCard big number | `text-xl font-bold font-display text-slate-900 dark:text-white` |
| `text-body` | Default body copy | `text-xs text-slate-700 dark:text-slate-300` |
| `text-caption` | Small helper / field hint | `text-xs text-slate-500 dark:text-slate-400` |
| `text-meta` | Monospace metadata (paths, IDs, timestamps) | `text-[10px] font-mono text-slate-500 dark:text-slate-400` |
| `text-code` | Inline code — size + family only | `text-xs font-mono` |
| `text-button` | Button label — size + weight only | `text-[10px] font-bold` |
| `text-nav` | Sidebar nav labels | `text-sm font-medium font-display` |
| `text-badge` | Badge / tag inner text — size + weight only | `text-[10px] font-bold` |

### Color-only tokens

Declared **after** the role tokens in `app.css` so that when combined
with a role token on the same element, the colour token wins.

| Class | Colour pair |
|---|---|
| `text-muted` | `text-slate-500 dark:text-slate-400` |
| `text-status-error` | `text-red-600 dark:text-red-400` |
| `text-status-success` | `text-green-600 dark:text-green-400` |
| `text-status-warning` | `text-amber-600 dark:text-amber-400` |
| `text-status-info` | brand `#137fec` in both modes |

## Usage

### Basic

```svelte
<h2 class="text-page-title">Dashboard</h2>
<h3 class="text-section-title">Recent Activity</h3>
<p class="text-body">Lorem ipsum dolor sit amet.</p>
<span class="text-meta">2m ago</span>
```

### Composing with layout utilities

Tailwind spacing / flex / layout utilities compose cleanly:

```svelte
<div class="flex items-center gap-2 px-3 py-1.5">
  <span class="text-stat-label">Memory</span>
  <span class="text-stat-value">42%</span>
</div>
```

### Overriding a role's default colour

Because Tailwind's `utilities` layer is output after `components`, any
`text-<color>` utility on the same element wins over the token's
default colour. This is the preferred way to introduce status hints:

```svelte
<!-- Default: slate colours from the token -->
<span class="text-stat-label">Idle</span>

<!-- Error variant: token provides size/weight/case; utility provides colour -->
<span class="text-stat-label text-status-error">Failed</span>

<!-- Brand accent -->
<a class="text-stat-label text-status-info hover:text-blue-400">View All</a>
```

Because `text-status-*` and `text-muted` are declared *after* the
role tokens within the `@layer components` block, they override the
role's default colour in either order in your `class` attribute.

### Linking a role with hover / variant behaviour

```svelte
<button class="text-button text-status-error hover:text-red-700">
  Delete
</button>
```

## Updating a Token (“the whole point”)

Need every panel header to be a touch bigger? Edit **one line** in
`src/app.css`:

```css
.text-section-title {
  @apply text-sm font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300;
  /*     ↑↑↑↑↑↑↑  was text-xs */
}
```

Every `<h3 class="text-section-title">` across the app picks up the
change. No hunting.

## Do / Don't

### Do

- Use the role tokens wherever they apply semantically.
- Layer `text-status-*` or `text-muted` on top of a role to tint it.
- Compose with Tailwind spacing / flex / display utilities freely.
- Reach for raw Tailwind text utilities for **one-off** sizes (e.g. a
  unique hero number at `text-4xl`). Don't invent a token for a single
  use.

### Don't

- **Don't apply text tokens to `material-symbols-outlined` spans.**
  Those are icons, not text — they use `text-[14px]` etc. as an icon
  size, and should keep their raw size class.
- **Don't stack two role tokens** on the same element
  (`class="text-body text-caption"`). Results are fragile and
  depend on declaration order. Pick the role that matches, then
  tweak with a single utility.
- **Don't re-declare a role's defaults inline**
  (`class="text-section-title text-xs font-bold uppercase ..."`).
  That defeats the purpose. If a role doesn't match, use raw
  utilities or propose a new token.
- **Don't add tokens speculatively.** The catalog is lean on purpose.
  If a new role appears 3+ times, then it earns a token.

## Extending

To add a new token:

1. Pick a **long, descriptive name** that does not collide with any
   Tailwind built-in utility (`text-xs/sm/...`, `text-left/center/...`,
   `text-{color}-{shade}`).
2. Add the definition in `@layer components` in `app.css`:
   - Role tokens in the upper block
   - Color-only tokens in the lower block (after roles)
3. Update this document with a row in the token table.
4. If migrating existing usages, add a regex to
   `scripts/migrate-typography.sh` so the consolidation is repeatable.

## Related Files

- [`src/app.css`](../src/app.css) — token definitions
- [`tailwind.config.js`](../tailwind.config.js) — font-family wiring
- [`scripts/migrate-typography.sh`](../scripts/migrate-typography.sh)
  — migration helper (rounds 1–3)
