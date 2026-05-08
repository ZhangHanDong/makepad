spec: task
name: "AI Chat Liquid Glass Step 82 macOS Metal Layer Factory"
tags: [makepad, liquid-glass, apple-native, interleave, macos]
---

## Intent

Prepare the macOS renderer for future lower/upper native interleave surfaces by
centralizing the duplicated `CAMetalLayer` creation and configuration used by
normal windows and popup windows.

## Decisions

- Introduce a private helper that creates and configures a `CAMetalLayer`.
- Keep `MetalWindow::new` and `MetalWindow::new_popup` behavior unchanged.
- Continue assigning the returned layer to the existing single view layer.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not create additional Metal layers in this step.
- Do not change draw-pass routing, resize behavior, native glass installation,
  or popup behavior.
- Keep the helper private to `macos.rs`.

## Acceptance Criteria

### Scenario: private layer factory exists
Given normal and popup windows need identical primary layer configuration
When the macOS platform source is inspected
Then it defines a private `new_macos_ca_metal_layer` helper
Test: `rg "fn new_macos_ca_metal_layer" platform/src/os/apple/macos/macos.rs`

### Scenario: layer factory owns core CAMetalLayer configuration
Given the helper must preserve the current layer behavior
When the helper is inspected
Then it configures `CAMetalLayer`, device, pixel format, drawable count, display sync, delegate, opacity, and transparent background
Test: `rg "CAMetalLayer|setDevice|setPixelFormat|setMaximumDrawableCount|setDisplaySyncEnabled|setDelegate|setOpaque|setBackgroundColor" platform/src/os/apple/macos/macos.rs`

### Scenario: normal window uses the layer factory
Given normal windows still use one primary layer
When `MetalWindow::new` is inspected
Then it calls `new_macos_ca_metal_layer`
Test: `rg "let ca_layer = new_macos_ca_metal_layer\\(metal_cx, cocoa_window\\.view\\)" platform/src/os/apple/macos/macos.rs`

### Scenario: popup window uses the layer factory
Given popup windows still use one primary layer
When `MetalWindow::new_popup` is inspected
Then it calls `new_macos_ca_metal_layer`
Test: `test "$(rg -c "new_macos_ca_metal_layer\\(metal_cx, cocoa_window\\.view\\)" platform/src/os/apple/macos/macos.rs)" = "2"`

### Scenario: platform crate still compiles
Given the extraction must preserve behavior
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only extracts the macOS layer factory
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-82-macos-layer-factory.spec platform/src/os/apple/macos/macos.rs "`
