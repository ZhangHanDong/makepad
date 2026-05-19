# P0A macOS NativeTextInput PoC Plan

Status note: this document records the original P0A compile-oriented landing
plan. The implementation has since advanced to the typed native host shape used
by the spec and roadmap. For current runtime closure work, use this document as
historical context and follow:

- `docs/native-textinput-macos-poc.spec.md` for the current contract.
- `docs/native-textinput-macos-poc.roadmap.md` for P0B and later sequencing.
- `examples/native_text_input/` as the primary smoke example.
- `examples/text_input/` and `widgets/src/text_input.rs` as the existing
  Makepad event/ref action reference.
- `makepad.splash` RunItems `makepad-example-native-text-input` and
  `makepad-example-native-text-input-diag` for Studio release validation.

## Goal

Create a minimal, compile-oriented macOS Host Widget proof of concept that wraps
`NSTextField` and validates the `NativeTextInput` lifecycle before generalizing
the abstraction. This goal describes P0A. P0B requires the runtime event,
selection, command, and cleanup behavior described by the spec and roadmap.

## Step 1: Copy Architecture Document

Target:

- `docs/rust-makepad-native-wrapping-architecture.md`

Source:

- `shared-docs/rust-makepad-native-wrapping-architecture.md`

## Step 2: Add Spec

Target:

- `docs/native-textinput-macos-poc.spec.md`

Contents:

- Intent, non-goals, architecture, props/commands/events, threading rules, and
  acceptance criteria for P0.

## Step 3: Add Widget Skeleton

Target:

- `widgets/src/native_text_input.rs`
- `widgets/src/lib.rs`

Implementation outline:

- Register `NativeTextInputBase` and `NativeTextInput` in `script_mod!`.
- Store `text`, `placeholder`, `editable`, `visible`, and `on_change`.
- Implement `Widget::script_call` for `text`, `set_text`, `focus`, and `blur`.
- In `draw_walk`, draw a lightweight placeholder/background area and enqueue:
  - spawn if not created,
  - props update if changed,
  - layout update every draw with `Area` and `visible`.

Current implementation note: the widget now also exposes `select_all`, `copy`,
`cut`, and `paste`, and receives native changed/focus/selection actions before
emitting widget-level `NativeTextInputAction` values.

## Step 4: Add Platform Ops

Target:

- `platform/src/cx_api.rs`

Original P0A allowed concrete text input ops:

```rust
SpawnNativeTextInput { text_input_id, text, placeholder, editable }
UpdateNativeTextInputLayout { text_input_id, area, visible }
SetNativeTextInputText { text_input_id, text, programmatic }
SetNativeTextInputPlaceholder { text_input_id, placeholder }
SetNativeTextInputEditable { text_input_id, editable }
CommandNativeTextInput { text_input_id, command }
DetachNativeTextInput { text_input_id }
CloseNativeTextInput { text_input_id }
```

This avoids over-generalizing the final `NativeHostWidget` op shape before P0
proves the lifecycle.

Current implementation note: these concrete ops have been superseded by the
typed native host API:

```rust
CreateNativeView { id, kind, props }
UpdateNativeViewLayout { id, area, visible }
UpdateNativeViewProps { id, update }
CommandNativeView { id, command }
DetachNativeView { id }
CloseNativeView { id }
```

`NativeHostKind`, `NativeHostProps`, `NativeHostPropUpdate`, and
`NativeHostCommand` are the source of truth for new work. Do not add new
`NativeTextInput`-specific `CxOsOp` variants unless the spec is deliberately
changed.

## Step 5: Add macOS Native View Wrapper

Target:

- `platform/src/os/apple/apple_native_text_input.rs`
- `platform/src/os/apple/macos/macos.rs`

Implementation outline:

- Follow `MacosSystemBrowser` structure:
  - `attached_window: Option<WindowId>`
  - `field: ObjcId`
  - `ensure_field`
  - `ensure_attached`
  - `update(window_id, parent_view, rect, visible)`
  - `detach`
  - `focus`
  - `blur`
- Use `NSTextField`.
- Set frame with the same y-axis flip already computed in macOS op handling.
- Use an `is_programmatic_update` guard around `set_text`.

## Step 6: Wire macOS Op Handling

Target:

- `platform/src/os/apple/macos/macos.rs`

Implementation outline:

- P0A used `native_text_inputs: HashMap<LiveId, MacosNativeTextInput>` in
  `CxOs`.
- Current code uses a shared native host registry so `NativeTextInput` and
  `NativeLabel` go through one platform host path.
- Handle create/update/command/detach/close in `handle_platform_ops`.
- Resolve `Area -> window_id -> parent_view -> flipped rect` exactly like
  `UpdateSystemBrowser`.

## Step 7: Verification

Compile gate:

```bash
cargo check -p makepad-widgets
```

Runtime gate through Studio:

- Use the checked-in smoke example at `examples/native_text_input/`.
- Keep `examples/text_input/` as the reference for existing `TextInput` action
  and `WidgetRef` patterns.
- Launch with Studio `RunItem` `makepad-example-native-text-input`, not raw
  `cargo run`.
- For leak diagnostics, launch `makepad-example-native-text-input-diag`.
- Verify field visibility, focus, blur, typing, changed action, selection
  action, select all, copy, cut, paste, detach, and close behavior.

## Implementation Carrier

Historical P0A carrier only: if this plan is prepared outside the writable
`fork-makepad` worktree, the original skeleton was applied by the companion
script from the source workspace:

```bash
python3 shared-docs/fork-makepad-native-textinput-poc/apply-native-textinput-poc.py \
  /Users/zhangalex/Work/Projects/fw/fork-makepad
```

The script copies the architecture document, this plan, and the spec into
`docs/`, writes the two new skeleton source files, and patches the existing
platform/widget registration points. Do not use it for P0B or later work unless
it is first updated to the current typed native host API and example layout.

Validation note: this script was applied to a temporary copy of
`fork-makepad`, and `cargo check -p makepad-widgets` completed successfully for
the generated P0 skeleton.

## Risks

- AppKit delegate/action callback bridging is now part of the current P0B
  implementation surface and must stay aligned with widget actions.
- `NSTextField` default editing behavior is enough for the macOS smoke example
  but not enough to close future IME or marked-text work.
- Typed native host payloads are now the baseline; the remaining risk is schema
  drift across Rust, AppKit, UIKit, Android, and ArkTS glue.
