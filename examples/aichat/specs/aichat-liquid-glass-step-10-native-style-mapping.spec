spec: task
name: "AI Chat Liquid Glass Step 10 Native Style Mapping"
tags: [makepad, aichat, liquid-glass, macos-native-clear, studio-validation]
---

## Intent

Fix the Studio validation mismatch where the `macos-native-clear` runnable
preflights with clear style but the native panel batch installs with regular
style. aichat must propagate the resolved macOS native style into each native
`GlassPanel` descriptor.

## Decisions

- Derive a `GlassNativeStyle` from `GlassAppearance.substrate` inside
  `apply_glass_appearance`.
- Map `GlassSubstrate::MacosNative { style: Clear }` to
  `GlassNativeStyle::Clear`.
- Map all other substrates to `GlassNativeStyle::Regular`; native panels remain
  disabled for non-native substrates.
- Apply the native style to shell, sidebar, main area, and composer panels.

## Constraints

- Must not change platform raw style mapping.
- Must not enable native panels for non-native substrates.
- Must keep aichat compiling.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-10-native-style-mapping.spec`
- `examples/aichat/src/main.rs`

### Forbidden

- `platform/**`
- `widgets/**`

### Out of Scope

- Visual tuning.
- Native installer changes.
- ShaderBackdrop.

## Acceptance Criteria

Scenario: aichat applies native style to native panels
Test: `rg "GlassNativeStyle::Clear|GlassNativeStyle::Regular|native_style: #\\(native_style\\)" examples/aichat/src/main.rs`
Given the resolved appearance is macOS native clear
When `apply_glass_appearance` updates the native panels
Then each exported panel descriptor uses clear style

Scenario: aichat still gates native panels by substrate
Test: `rg "use_native_panels|GlassSubstrate::MacosNative|native: #\\(use_native_panels\\)" examples/aichat/src/main.rs`
Given the substrate is not macOS native
When appearance is applied
Then native container export remains disabled

Scenario: aichat compiles after style propagation
Test: `cargo check -p makepad-example-aichat`
Given `GlassNativeStyle` is passed through script application
When the aichat crate is checked
Then the runtime update code compiles
