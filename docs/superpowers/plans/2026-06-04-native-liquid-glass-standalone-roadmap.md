# Native Liquid Glass Standalone Roadmap

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `makepad-example-native-liquid-glass` into the canonical standalone validation bench for Makepad's Apple native Liquid Glass integration.

**Architecture:** Keep the standalone example focused on Apple native glass behavior: native panel descriptors, container spacing, style/tint/shape mapping, native controls, hit-test policy, and per-panel install state. Keep Makepad-only visual overlays such as the dynamic gold glint explicitly marked as demo overlays so they do not get mistaken for Apple native API behavior.

**Tech Stack:** Rust, Makepad 2.0 `script_mod!`, `makepad-widgets`, `GlassPanel`/`GlassContainer`, macOS `NSGlassEffectView` backend, Studio remote `RunItem`, release `cargo test`.

---

## Current State

- `examples/native_liquid_glass/src/main.rs` is the primary standalone validation surface.
- Existing modes cover `Panels`, `Geometry`, `Morph`, `Container`, `Debug`,
  `Controls`, and `Readability`.
- The example launches as a transparent borderless macOS window and installs three native glass panels.
- Native state logging already reports container and panel install results, including the v1 compatibility line `state=4 substrate=macos-native style=clear`.
- The demo includes tint/tone cycling, pulse-driven spacing/radius/tint changes, native control role probes, and a Makepad-rendered dynamic gold glint overlay.
- The gold glint is intentionally not an Apple native API feature; it is a shader overlay used to visualize glass edges and interaction energy.

## File Map

- `examples/native_liquid_glass/src/main.rs`: standalone app, modes, controls, pure helpers, tests, and visual demo overlay.
- `widgets/src/glass_panel.rs`: inspect when descriptor fields or script-facing native glass properties need extension.
- `widgets/src/glass_panel.rs`: also contains `GlassContainer`; inspect it when
  container spacing, panel collection, or traversal behavior changes.
- `platform/src/os/apple/macos/macos_window.rs`: native AppKit backend, `NSGlassEffectView` installation, container creation, hit-test policy, state logging.
- `docs/superpowers/plans/2026-06-04-native-liquid-glass-standalone-roadmap.md`: this roadmap.

## Non-Goals

- Do not use standalone visual overlays as proof of Apple native Liquid Glass behavior.
- Do not mix ShaderBackdrop refraction work into the Apple native path.
- Do not add per-row or unbounded native panels.
- Do not make aichat the primary validation surface until standalone covers the native API matrix.

## Task 1: Commit The Current Standalone Checkpoint

**Files:**
- Modify: `examples/native_liquid_glass/src/main.rs`
- Modify: `examples/native_liquid_glass/README.md`
- Modify: `docs/superpowers/plans/2026-06-04-native-liquid-glass-standalone-roadmap.md`
- Create: `docs/research/native-liquid-glass-standalone-notes.md`
- Create: `docs/research/native-liquid-glass-studio-checklist.md`

- [x] **Step 1: Verify release tests**

Run:

```bash
cargo test -p makepad-example-native-liquid-glass --release
```

Expected: all standalone tests pass.

- [x] **Step 2: Verify formatting cleanliness**

Run:

```bash
git diff --check
```

Expected: no whitespace errors.

- [x] **Step 3: Commit only relevant files**

Run:

```bash
git add examples/native_liquid_glass/src/main.rs \
  examples/native_liquid_glass/README.md \
  docs/superpowers/plans/2026-06-04-native-liquid-glass-standalone-roadmap.md \
  docs/research/native-liquid-glass-standalone-notes.md \
  docs/research/native-liquid-glass-studio-checklist.md
git commit -m "feat: expand native liquid glass standalone bench"
```

Do not add `.makepad/`.

## Task 2: Add Dedicated Geometry Mode

**Goal:** Make resize, radius, capsule, and panel alignment failures easy to see without relying on the main demo scene.

**Files:**
- Modify: `examples/native_liquid_glass/src/main.rs`

- [x] **Step 1: Add failing tests**

Add tests that require `NativeDemoVisualMode::Geometry`, a stable label, and a distinct scene layout from `Panels`.

- [x] **Step 2: Implement `Geometry` mode**

Add `Geometry` to `NativeDemoVisualMode`, include it in mode switching, and configure scene values that emphasize:

- large main rounded rect
- small capsule
- top-right rounded rect
- resize-safe margins

- [x] **Step 3: Add UI control**

Add a `Geometry` button near `Panels / Morph / Controls / Readability`.

- [x] **Step 4: Verify**

Run:

```bash
cargo test -p makepad-example-native-liquid-glass --release
```

Then launch via Studio `RunItem` and resize the window. Expected: all native panels keep stable alignment and rounded shapes.

## Task 3: Add Dedicated Container Mode

**Goal:** Separate Apple container spacing/morph validation from the more general Morph scene.

**Files:**
- Modify: `examples/native_liquid_glass/src/main.rs`
- Inspect: `widgets/src/glass_panel.rs`
- Inspect: `platform/src/os/apple/macos/macos_window.rs`

- [x] **Step 1: Add failing tests**

Require a `Container` mode with at least three spacing presets: near, threshold, far.

- [x] **Step 2: Implement mode state**

Add a small enum for container spacing probes, or reuse existing morph state if it remains clear.

- [x] **Step 3: Wire controls**

Add controls for `Near`, `Threshold`, and `Far`. The scene should place panels so the Apple container spacing behavior is visually obvious.

- [x] **Step 4: Verify Studio logs**

Run via Studio and confirm logs include:

```text
[liquid-glass] native-container-spacing container=... spacing=...
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=3 panels_failed=0
```

## Task 4: Add Debug Mode

**Goal:** Make native descriptor and install state visible inside the standalone app.

**Files:**
- Modify: `examples/native_liquid_glass/src/main.rs`
- Inspect: `platform/src/os/apple/macos/macos_window.rs`

- [x] **Step 1: Add pure debug summary helper**

Add a helper that formats:

- mode
- style
- tone
- tint alpha
- spacing
- radius
- panel count
- current expected `z_order`

- [x] **Step 2: Add failing tests**

Test the summary helper includes mode, style, spacing, radius, and panel count.

- [x] **Step 3: Add Debug mode UI**

Show a compact text block that mirrors the expected descriptor batch. Keep it Makepad-rendered and readable over native glass.

- [x] **Step 4: Verify**

Run release tests and Studio screenshot. Expected: debug text is readable and does not cover the primary panel edges.

## Task 5: Add Accessibility / Contrast Mode

**Goal:** Verify that native glass remains readable under bright and dark backgrounds, and prepare for Reduce Transparency behavior.

**Files:**
- Modify: `examples/native_liquid_glass/src/main.rs`
- Inspect: `platform/src/os/apple/macos/macos_window.rs`

- [x] **Step 1: Add contrast token helper**

Add pure helper values for bright-background foreground text, dark-background foreground text, and separator colors.

- [x] **Step 2: Add failing tests**

Test that bright and dark contrast tokens differ and keep alpha high enough for text.

- [x] **Step 3: Extend Readability mode**

Add a simple toggle between bright, dark, and mixed visual backgrounds. Keep panel tint alpha within the native translucency range.

- [x] **Step 4: Verify**

Run release tests and Studio screenshots for bright and dark states.

## Task 6: Make Demo Overlays Explicit

**Goal:** Prevent gold glint and other Makepad-rendered overlays from being confused with Apple native glass features.

**Files:**
- Modify: `examples/native_liquid_glass/src/main.rs`
- Modify or create: `docs/research/native-liquid-glass-standalone-notes.md`

- [x] **Step 1: Rename or label overlay code**

Keep `GoldGlintEdge`, but ensure in-code names and docs call it a Makepad demo overlay.

- [x] **Step 2: Add a visibility toggle**

Add `Glint On/Off` so native glass can be inspected without shader decoration.

- [x] **Step 3: Add tests**

Test the glint toggle state and status text.

- [x] **Step 4: Verify**

Run Studio screenshots with glint on and off. Expected: native glass remains visible without the overlay.

## Task 7: Add Studio Verification Script Notes

**Goal:** Standardize manual validation through Studio remote.

**Files:**
- Modify: `docs/superpowers/plans/2026-06-04-native-liquid-glass-standalone-roadmap.md`
- Create: `docs/research/native-liquid-glass-studio-checklist.md`

Checklist artifact:

- `docs/research/native-liquid-glass-studio-checklist.md` records the required
  Studio remote launch flow, state=4 log evidence, visual acceptance, and
  non-acceptance criteria for demo overlays and Studio screenshots.

- [x] **Step 1: Document Studio flow**

Record the exact flow:

```text
ListBuilds
ClearBuild old standalone build_id
RunItem makepad-example-native-liquid-glass
wait for state=4
WidgetTreeDump
Screenshot
Click Tone / Morph / Controls / Debug
Screenshot again
```

- [x] **Step 2: Define visual acceptance**

Expected:

- transparent borderless window
- native glass visible under Makepad foreground
- no opaque root background
- panel geometry tracks resize
- clear/regular styles visibly differ
- glint overlay can be disabled

## Task 8: Revisit Native Backend Gaps From Standalone

**Goal:** Use standalone as the first place to expose missing platform API support.

**Files:**
- Modify as needed: `widgets/src/glass_panel.rs`
- Modify as needed: `widgets/src/glass_panel.rs`
- Modify as needed: `platform/src/os/apple/macos/macos_window.rs`
- Modify: `examples/native_liquid_glass/src/main.rs`

- [x] **Step 1: Audit descriptor fields**

Confirm standalone can express all v4.1 fields:

- style
- tint
- shape
- radius
- z_order
- hit_test passthrough
- container spacing

- [x] **Step 2: Add missing script-facing knobs**

Only add fields that map to real descriptor/backend behavior.

- [x] **Step 3: Add tests**

Use pure helper tests for descriptor defaults and scene mapping; use Studio for runtime native install verification.

- [x] **Step 4: Verify**

Run:

```bash
cargo test -p makepad-example-native-liquid-glass --release
cargo test -p makepad-widgets --release glass
```

Then run standalone through Studio and confirm `state=4`.

## Acceptance Checklist

- [x] Current standalone checkpoint is committed without `.makepad/`.
- [x] Standalone has modes dedicated to geometry, container spacing, debug state, controls, readability, and demo overlays.
- [x] Every mode has release unit tests for pure state/scene behavior.
- [x] Studio validation reaches `state=4 substrate=macos-native`.
- [x] Native glass behavior can be inspected with demo overlays disabled.
- [x] Documentation clearly separates Apple native API behavior from Makepad shader overlays.
