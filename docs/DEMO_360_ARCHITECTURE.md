# 360° Media Demo — Architecture and Implementation

## 1. Objective

Provide a fully isolated sandbox inside the app for previewing
**equirectangular 360° photos and videos**. The page must:

- Live at its own route (`/demo-360`) so it can be iterated without risk
  to the rest of the app.
- Reuse the existing UI primitives (`Panel`, `TabBar`, `ToggleSwitch`, …)
  for visual consistency.
- Touch **zero existing routes, components, or backend code**.
- Be extensible later (cubemap projection, backend metadata detection,
  embedding into `dedup` / `analysis` previews) without redesign.

## 2. Why the Existing Player Is Not Enough

The in-app media player ([`src/lib/components/media/VideoPlayer.svelte`](../src/lib/components/media/VideoPlayer.svelte))
is a plain `<video>` element. A 360° asset is a 2:1 **equirectangular
projection** — a flattened sphere. Rendered as a flat rectangle it looks
like a stretched world map, not a panoramic scene.

Correct playback requires:

1. A 3D sphere whose inside surface is textured with the source image
   or video.
2. A perspective camera placed at the sphere's centre.
3. Interactive yaw / pitch / FOV controls driving that camera.

This is what the demo page provides, entirely on the client side with
[Three.js](https://threejs.org/).

## 3. File Layout

All new code is confined to two directories:

```
src/routes/demo-360/
  +page.svelte                         Tabbed shell (Photo | Video)
  +page.ts                             export const prerender = true

src/lib/components/features/demo360/
  Panoramic360Viewer.svelte            Three.js renderer + pointer controls
  PhotoDemo.svelte                     File picker + viewer host (image)
  VideoDemo.svelte                     File picker + <video> + playback UI + viewer host
  ControlsOverlay.svelte               FOV / drag-sensitivity / auto-rotate / reset
  index.ts                             Barrel export
```

The only touch point outside this tree is the sidebar nav entry added in
[`src/routes/+layout.svelte`](../src/routes/+layout.svelte) and
`three` / `@types/three` in `package.json`.

## 4. Runtime Architecture

```
┌─────────────────────────── /demo-360 +page.svelte ─────────────────────────┐
│                                                                            │
│   TabBar ──────► activeTab: 'photo' | 'video'                              │
│                                                                            │
│   ┌────────────────────────────── PhotoDemo ─────────────────────────────┐ │
│   │                                                                      │ │
│   │  <input type=file>                                                   │ │
│   │        │                                                             │ │
│   │        ▼                                                             │ │
│   │   URL.createObjectURL(file) ──► new Image()                          │ │
│   │                                     │                                │ │
│   │                                     ▼                                │ │
│   │   ┌─────── ControlsOverlay ───────┐  source ────►  Panoramic360Viewer │ │
│   │   │ fov / autoRotate / dragSens   │  fov (bind) ◄─►                   │ │
│   │   │ reset → resetToken++          │  autoRotate                       │ │
│   │   │ view readout                  │  dragSensitivity                  │ │
│   │   └──────────── bind ─────────────┘  onView(v)                        │ │
│   │                                                                      │ │
│   └──────────────────────────────────────────────────────────────────────┘ │
│                                                                            │
│   ┌────────────────────────────── VideoDemo ─────────────────────────────┐ │
│   │  same shape, source = HTMLVideoElement, + play/seek/volume UI        │ │
│   └──────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────────┘
```

The viewer treats image and video uniformly — the only branch is which
`THREE.Texture` subclass is created in `applySource`:

| Source type          | Texture                                       |
|----------------------|-----------------------------------------------|
| `HTMLImageElement`   | `new THREE.Texture(img)` + `needsUpdate = true` |
| `HTMLVideoElement`   | `new THREE.VideoTexture(video)` (self-updates each frame) |

Everything downstream — sphere geometry, material, camera, render loop —
is identical.

## 5. The Viewer in Detail — `Panoramic360Viewer.svelte`

### 5.1 Props

| Name              | Type                                        | Default | Notes                                                   |
|-------------------|---------------------------------------------|---------|---------------------------------------------------------|
| `source`          | `HTMLImageElement \| HTMLVideoElement \| null` | —     | Texture source. `null` clears the sphere to dark grey.  |
| `fov`             | `number` (bindable)                          | `75`    | Vertical FOV (deg). Bindable so wheel zoom propagates back to parent / slider. |
| `autoRotate`      | `boolean`                                    | `false` | Slow yaw rotation when idle.                            |
| `dragSensitivity` | `number`                                     | `0.6`   | Multiplier on the FOV-per-height drag formula (see 5.4).|
| `resetToken`      | `number`                                     | `0`     | Increment from parent to reset view (lon=0, lat=0, fov=75). |
| `onView`          | `(s: { lon, lat, fov }) => void`             | —       | Callback every animation frame with current camera state. |

### 5.2 Scene Graph

```
Scene
 └── Mesh
      ├── SphereGeometry(500, 60, 40)     // radius, widthSeg, heightSeg
      │   geometry.scale(-1, 1, 1)         // flip inside-out: we look at the back faces
      └── MeshBasicMaterial
           ├── map          = THREE.Texture | THREE.VideoTexture | null
           ├── color        = 0xffffff (with texture) / 0x111111 (empty state)
           ├── colorSpace   = SRGBColorSpace
           └── minFilter/magFilter = LinearFilter, generateMipmaps = false
Camera: PerspectiveCamera(fov, aspect, 0.1, 1100)  positioned at origin
```

The sphere radius (500) is arbitrary — any value well inside the far
clipping plane works. What matters is that the camera sits at the centre
looking outward, with the mesh normals reversed so the texture is
visible from inside.

### 5.3 Camera Aiming (Spherical → Cartesian)

`lon` (longitude, yaw) and `lat` (latitude, pitch) are maintained in
degrees. Each frame converts them to a lookAt target:

```
phi   = (90 - lat) * π / 180
theta =  lon        * π / 180

tx = 500 · sin(phi) · cos(theta)
ty = 500 · cos(phi)
tz = 500 · sin(phi) · sin(theta)

camera.lookAt(tx, ty, tz)
```

`lat` is clamped to ±85° so the camera never passes the poles (which
would unexpectedly flip horizon orientation).

### 5.4 Pointer Drag Sensitivity

The drag formula is **FOV-proportional** and **height-normalised**:

```
degPerPx = (camera.fov / container.clientHeight) * dragSensitivity
lon = lonStart - dx * degPerPx
lat = latStart + dy * degPerPx
```

This means:

- Dragging across the full canvas height at `dragSensitivity = 1` traverses
  exactly `camera.fov` degrees — i.e. one full vertical field of view.
- Zooming in (smaller FOV) automatically makes the drag finer — useful
  when composing a shot or examining detail.

`dragSensitivity` is exposed as a prop (and as a slider in
`ControlsOverlay`) so users can pick a feel without touching code.
A default of `0.6` was tuned by hand: slightly slower than strictly
"natural" for comfortable scene-scanning.

### 5.5 Wheel Zoom

```
fov = clamp(fov + deltaY * 0.05, 30, 90)
```

`fov` is bindable, so this write propagates up to `PhotoDemo` /
`VideoDemo` and the FOV slider in `ControlsOverlay` stays in sync.

### 5.6 Lifecycle — The Reactivity Trap

Svelte 5's `$effect` auto-tracks every `$state` / prop read during
synchronous execution. The initial Three.js setup is wrapped in
`untrack(() => { ... })` for a reason — **without this, the whole scene
tears down and rebuilds every time the user moves the FOV slider or
toggles auto-rotate**. That was a real bug caught during development.

Why: the setup block reads `fov` (when constructing `PerspectiveCamera`)
and its last line calls `animate()` synchronously, whose body reads
`autoRotate`, `lon`, `lat`, and `fov`. Any of those changing would
re-run the effect — disposing the renderer, texture, geometry, and
material, then recreating them with a **fresh, untextured** mesh (since
`source` didn't change, the separate `applySource` effect had no reason
to run). Result: black sphere.

The structure now is:

```ts
$effect(() => {
  if (!canvas || !container) return;
  untrack(() => {
    // build renderer / scene / camera / mesh / resize observer / start rAF
  });
  return () => { /* full dispose on unmount */ };
});

$effect(() => {              // dedicated FOV updater — cheap
  if (!camera) return;
  camera.fov = clamp(fov, 30, 90);
  camera.updateProjectionMatrix();
});

$effect(() => {              // source swap — disposes old texture, binds new
  if (!mesh) return;
  applySource(source);
});

let firstResetRun = true;
$effect(() => {              // parent-triggered view reset
  void resetToken;
  if (firstResetRun) { firstResetRun = false; return; }
  untrack(() => { lon = 0; lat = 0; fov = 75; });
});
```

`animate()` reschedules itself via `requestAnimationFrame`, whose
callback is outside any reactive scope, so reads inside the loop
(`autoRotate`, `lon`, `lat`, `camera.fov`) do not register as
dependencies — which is what we want.

### 5.7 Disposal

Every GPU-side object is cleaned up on unmount:

```ts
cancelAnimationFrame(rafId);
resizeObserver.disconnect();
texture?.dispose();
mesh.geometry.dispose();
mesh.material.dispose();
renderer.dispose();
```

When the `source` prop changes, the old texture is disposed before the
new one is bound (`disposeTexture()` inside `applySource`). This matters
because Tauri sessions are long-lived and users may swap many files.

## 6. Photo Flow — `PhotoDemo.svelte`

1. `<input type="file" accept="image/*">` picks a local file.
2. `URL.createObjectURL(file)` produces a blob URL.
3. A new `HTMLImageElement` loads it. On `img.onload`, the element is
   handed to the viewer via the `source` prop and `resetToken` is
   incremented so the view returns to the default orientation.
4. Clearing or swapping files revokes the previous object URL to avoid
   leaking blob memory.

No backend round-trip — fully client-side.

## 7. Video Flow — `VideoDemo.svelte`

1. File pick → object URL → new `HTMLVideoElement` (hidden, attached to
   `document`-free scope via `document.createElement('video')`).
2. The element is configured: `loop`, `muted = true` (autoplay policy),
   `playsInline`, `preload = 'auto'`.
3. On `loadedmetadata` the video is assigned as `source` and playback is
   attempted with `.play().catch(...)` (may still fail under strict
   autoplay policies; UI falls back to paused state).
4. `THREE.VideoTexture` is self-updating — each rendered frame samples
   the current video texture, no manual `needsUpdate` needed.
5. A playback panel provides play/pause, scrubber (`input type=range`
   bound to `currentTime`), and volume / mute.
6. Cleanup on swap or unmount pauses the video, clears its `src`, calls
   `load()`, and revokes the blob URL.

## 8. Controls Overlay — `ControlsOverlay.svelte`

A small bindable-props component that renders:

- **FOV slider** — 30°–90°, step 1°, bound to `fov`.
- **Drag-sensitivity slider** — 0.10×–2.00×, step 0.05, bound to
  `dragSensitivity` (labelled slow / natural / fast).
- **Auto-rotate toggle** — reuses `ToggleSwitch` from the UI primitives.
- **Reset view button** — invokes the parent-supplied `onReset`, which
  increments `resetToken`.
- **Live debug readout** — `lon / lat / fov` values from the `onView`
  callback.

All three adjustable parameters round-trip through `$bindable()`, so
writes from the viewer (wheel → fov) and writes from the slider are
consistent.

## 9. Isolation Guarantees

This demo does not depend on nor mutate:

- Any other route under `src/routes/`.
- `src/lib/components/media/VideoPlayer.svelte` or any other shared
  player.
- `src/lib/types.ts`, `src/lib/config.svelte.ts`, `src/lib/events.ts`.
- Anything inside `src-tauri/`.

Only outward-facing additions:

- One sidebar entry in `src/routes/+layout.svelte` (and a matching
  header title case).
- `three` and `@types/three` in `package.json`.

Vite's route-level code splitting keeps Three.js out of the other
pages' bundles. The `/demo-360` client chunk carries the ~500 KB
Three.js payload; every other route loads only when visited.

## 10. Performance Notes

- **Mipmaps disabled** (`generateMipmaps = false`) — the source is
  nearly screen-resolution, and mipmap chain generation on large
  textures (8K equirectangular photos are common) is slow and
  memory-hungry. Linear min/mag filters are enough for inside-sphere
  sampling.
- **Pixel ratio** is set once to `window.devicePixelRatio`. Resize is
  tracked via `ResizeObserver` on the container.
- **No OrbitControls dependency** — custom pointer handlers give us
  sensitivity tuning, pitch clamping, and pointer capture without
  pulling in `three/examples/jsm/controls/OrbitControls.js` (which
  would double the JS weight for features we don't need).
- **No anisotropy bump** — added only if moiré becomes visible on
  high-zoom equator shots in practice.

## 11. Limitations and Known Constraints

- **Equirectangular only.** Cubemap, EAC (YouTube's equi-angular
  cubemap), and fisheye projections are not yet supported. The viewer
  architecture can accept them — a `projection` prop would branch in
  `applySource` / geometry construction — but is deliberately out of
  scope for the demo.
- **No automatic 360° detection.** The user picks a file and the viewer
  assumes it is equirectangular 2:1. Spherical metadata in MP4
  (`st3d`/`sv3d` boxes) or in image EXIF (GPano) is not parsed. This
  would need ffprobe on the Rust side, which is deliberately out of
  scope for a sandbox demo.
- **Codec support follows the WebView.** On macOS (WebKit) H.264 MP4 is
  reliable; HEVC depends on system codecs; AV1 coverage is uneven. The
  demo surfaces a decode failure via an inline error message.
- **Autoplay policies.** Videos start muted to satisfy browser autoplay
  rules. Unmuting via the volume slider is a user-initiated action and
  works after that point.
- **No VR / WebXR.** Intentional — desktop Tauri context.

## 12. Extension Paths

If and when this moves out of "demo" status, natural next steps are:

1. **Backend metadata detection** — extend
   `src-tauri/src/commands/metadata.rs` with a `detect_spherical`
   command that invokes ffprobe and checks EXIF. Return
   `{ is_panoramic: bool, projection: '...' }`.
2. **Shared type** — add `projection: 'flat' | 'equirect' | 'cubemap'`
   to whichever media metadata struct is already flowing through
   `src/lib/types.ts`, but **only** once consumers exist.
3. **Integration into existing previews** — dedup / analysis thumbnails
   of 360° assets could open the panoramic viewer in a modal instead of
   the flat player. The viewer already accepts any `HTMLImageElement` /
   `HTMLVideoElement`, so integration is a matter of routing the
   element to the new component based on metadata.
4. **Additional projections** — introduce a `projection` prop on
   `Panoramic360Viewer` and branch on geometry (`BoxGeometry` +
   cubemap, `SphereGeometry` for equirect).
5. **Stereoscopic 3D** — top-bottom or side-by-side 3D 360° videos
   would require two cameras into different halves of the texture;
   feasible but non-trivial.

## 13. Access

- **URL:** `/demo-360`
- **Sidebar entry:** "360° Demo" (icon: `panorama_photosphere`).
- **Interaction:**
  - Drag to look around.
  - Scroll to zoom (FOV).
  - Slider / toggle / reset in the **Controls** panel.
  - Video tab additionally exposes play / seek / volume / mute.

## 14. Verification

- `yarn run check` — 0 errors, 0 warnings in the `demo360/` tree.
- `yarn build` — static build succeeds; Three.js ships only in the
  `/demo-360` client chunk; all other route bundles unchanged.
- Manual: load an equirectangular JPEG and an MP4; confirm drag, zoom,
  FOV slider, drag-sensitivity slider, auto-rotate, and reset all
  behave without scene rebuild or texture loss.
