spec: task
name: "AI Chat Liquid Glass Step 86 Interleave Probe Role Struct"
tags: [makepad, liquid-glass, apple-native, interleave, macos]
---

## Intent

Refactor the hidden macOS interleave probe from loose fields into a typed probe
layer structure that records its future renderer role. This prepares lower and
upper draw routing without changing the current hidden probe behavior.

## Decisions

- Keep the primary surface role as `MacosMetalSurfaceRole::Primary`.
- Add reserved `LowerScene` and `UpperUi` roles to the same enum.
- Store the hidden probe as `MacosInterleaveProbeLayer` with role
  `LowerScene`.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not draw Makepad content into the probe layer.
- Do not construct `UpperUi` yet.
- Do not change the Studio runnable or default runtime behavior.

## Acceptance Criteria

### Scenario: surface role enum includes future split roles
Given draw routing will eventually need lower and upper surfaces
When the macOS source is inspected
Then `MacosMetalSurfaceRole` includes `Primary`, `LowerScene`, and `UpperUi`
Test: `rg "Primary|LowerScene|UpperUi" platform/src/os/apple/macos/macos.rs`

### Scenario: hidden probe has a typed layer structure
Given raw probe fields are too ambiguous for future draw routing
When the macOS source is inspected
Then it defines `MacosInterleaveProbeLayer` with role, layer, and drawable checked fields
Test: `rg "struct MacosInterleaveProbeLayer|role: MacosMetalSurfaceRole|ca_layer: ObjcId|drawable_checked: bool" platform/src/os/apple/macos/macos.rs`

### Scenario: probe installs as LowerScene
Given the hidden probe is the future lower sampling surface
When the installer is inspected
Then it returns `MacosInterleaveProbeLayer` with `role: MacosMetalSurfaceRole::LowerScene`
Test: `rg "MacosInterleaveProbeLayer|role: MacosMetalSurfaceRole::LowerScene" platform/src/os/apple/macos/macos.rs`

### Scenario: drawable check uses the typed probe
Given drawable probing should operate on the typed probe state
When resize/DPI synchronization is inspected
Then it uses `probe.ca_layer` and `probe.drawable_checked`
Test: `rg "probe\\.ca_layer|probe\\.drawable_checked" platform/src/os/apple/macos/macos.rs`

### Scenario: platform crate still compiles
Given this is a refactor of the hidden probe state
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only refactors macOS probe state
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-86-interleave-probe-role-struct.spec platform/src/os/apple/macos/macos.rs "`
