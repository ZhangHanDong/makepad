spec: task
name: "AI Chat Liquid Glass Step 05 aichat Native Wiring"
tags: [makepad, aichat, liquid-glass, native-panels, runtime-wiring]
---

## Intent

Wire the aichat shell to the new native descriptor export path without changing
the default shader appearance. The app should enable `GlassContainer.native`
only when the resolved substrate is macOS native, and the shell/sidebar/main
area/composer panels should be eligible native panels while retaining their
existing Makepad-rendered content and fallback shader styling.

## Decisions

- Wrap the aichat glass shell in a single `GlassContainer` named
  `glass_container`.
- Keep `glass_container.native: false` in script defaults.
- Toggle `glass_container.native` from Rust in `apply_glass_appearance` when
  `GlassSubstrate::MacosNative` is active.
- Mark only the shell, sidebar, main area, and composer `GlassPanel`s as native
  panels in v4.1.
- Keep Makepad content above native glass by leaving the panel contents in the
  normal widget tree.
- Preserve the existing native Splash opaque-root guard behavior when native
  glass is active.

## Constraints

- Must not force native panels for shader, transparent, or proof substrates.
- Must not add more than twelve native panels to aichat v4.1.
- Must not remove existing shader `GlassPanel` draw properties or opacity
  controls.
- Must keep aichat compiling with the new `GlassPanel` wrapper widget.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-05-aichat-wiring.spec`
- `examples/aichat/src/main.rs`

### Forbidden

- `platform/**`
- `widgets/**`
- `examples/aichat/prototypes/**`

### Out of Scope

- Native installer changes.
- Widget descriptor collector changes.
- Studio visual validation.
- ShaderBackdrop.

## Acceptance Criteria

Scenario: aichat wraps native panels in one GlassContainer
Test: `rg "glass_container := GlassContainer|native: false|spacing: 20\\.0" examples/aichat/src/main.rs`
Given the aichat shell is rendered
When the script layout is searched
Then a single named `GlassContainer` exists with native disabled by default

Scenario: aichat native panel count stays within v4.1 budget
Test: `test "$(rg "native: true" examples/aichat/src/main.rs | wc -l | tr -d ' ')" -le 12`
Given v4.1 allows at most twelve native panels per window
When native panel declarations are counted
Then aichat stays within the native panel budget

Scenario: native container toggles only for macOS native substrate
Test: `rg "GlassSubstrate::MacosNative|glass_container|native: #\\(use_native_panels\\)" examples/aichat/src/main.rs`
Given `apply_glass_appearance` receives a resolved substrate
When the substrate is macOS native
Then it enables `glass_container.native`; otherwise it leaves native panels disabled

Scenario: aichat example compiles with native wiring
Test: `cargo check -p makepad-example-aichat`
Given `GlassPanel` is now a Rust wrapper widget
When the aichat example is checked
Then the script layout and Rust references compile

Scenario: Step 05 commit is limited to aichat wiring files
Test: `git diff --name-only --cached | rg -v "^(examples/aichat/specs/aichat-liquid-glass-step-05-aichat-wiring.spec|examples/aichat/src/main.rs)$" && exit 1 || exit 0`
Given only Step 05 files are staged
When the staged file list is checked
Then platform and widget source files are absent
