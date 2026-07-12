spec: task
name: "P0 macOS NativeTextInput Host Widget PoC"
tags: [makepad, macos, native-host-widget, text-input, splash, poc]
---

## Intent

Implement the smallest macOS proof of concept for a Makepad Host Widget that
wraps an AppKit text field as a native leaf view.

This validates the architecture in
`docs/rust-makepad-native-wrapping-architecture.md`: Splash calls widget
methods, the widget enqueues typed platform ops, and macOS consumes those ops on
the platform UI thread using the same native-view overlay pattern already used
by `MacosSystemBrowser`.

## Non-Goals

- No generic proc macro.
- No complete RN/Fabric-style mounting transaction.
- No Texture Layer or render-to-texture path.
- No custom `NSTextInputClient` implementation.
- No rich text, candidate window customization, or multi-line text field
  behavior.
- P0 focuses on macOS runtime closure. iOS, Android, and OHOS are tracked in
  later roadmap phases and must pass their own device/runtime gates before
  being considered closed.

## User-Facing Behavior

Splash can render a named native text input and call basic methods:

```splash
View{
    width: Fill height: Fit flow: Down spacing: 8

    input := NativeTextInput{
        width: Fill height: 32
        placeholder: "Type text"
        on_change: |text| ui.preview.set_text(text)
    }

    Button{text: "Focus" on_click: || ui.input.focus()}
    Button{text: "Clear" on_click: || ui.input.set_text("")}

    preview := Label{text: ""}
}
```

Expected behavior on macOS:

- The native `NSTextField` appears at the Makepad widget location.
- `ui.input.focus()` makes the AppKit text field first responder.
- `ui.input.blur()` resigns focus.
- `ui.input.set_text("...")` updates the native field without firing
  `on_change`.
- User edits emit `NativeTextInputAction::Changed(text)` for Rust callers and
  invoke `on_change(text)` for Splash callers.
- Focus changes emit `NativeTextInputAction::KeyFocus` /
  `NativeTextInputAction::KeyFocusLost`.
- Selection changes emit
  `NativeTextInputAction::SelectionChanged { start, end }`.
- `ui.input.select_all()`, `ui.input.copy()`, `ui.input.cut()`, and
  `ui.input.paste()` forward to the platform native text field.

## Architecture

Minimum loop:

```text
Splash
  -> ui.input.focus() / ui.input.set_text(...) / ui.input.copy()
  -> Widget::script_call(...)
  -> CxNativeTextInput queues typed NativeMountMutation
  -> CxOsOp::CreateNativeView / UpdateNativeViewProps / CommandNativeView
  -> macOS handle_platform_ops main-thread drain
  -> MacosNativeTextInput updates NSTextField
  -> AppKit delegate/action posts NativeTextInputChanged / FocusChanged /
     SelectionChanged
  -> NativeTextInput::handle_event updates widget state and emits widget action
```

## Threading Rules

- Widget code may enqueue `CxOsOp`.
- Widget code must not directly call AppKit APIs.
- macOS platform op consumption must run on the existing main-loop
  `handle_platform_ops` path.
- Native text input ops must be drained at a frame boundary, not immediately
  from each widget.
- Native callbacks must enter Rust through `Cx::try_post_action(...)`, so Drop
  and delegate paths are best-effort and non-panicking.

## Props / Commands / Events

Props:

- `text: String`
- `placeholder: String`
- `editable: bool`

Commands:

- `focus()`
- `blur()`
- `set_text(text)`
- `select_all()`
- `copy()`
- `cut()`
- `paste()`

Events:

- `on_change(text)`
- `NativeTextInputAction::Changed(text)`
- `NativeTextInputAction::KeyFocus`
- `NativeTextInputAction::KeyFocusLost`
- `NativeTextInputAction::SelectionChanged { start, end }`

P0A originally allowed concrete `CxOsOp` variants. The current implementation
uses the roadmap's typed native host form:
`CreateNativeView { kind, props }`, `UpdateNativeViewProps`, and
`CommandNativeView`.

## Implementation Targets

Expected new or changed files:

- `widgets/src/native_text_input.rs`
- `widgets/src/lib.rs`
- `platform/src/cx_api.rs`
- `platform/src/os/apple/apple_native_text_input.rs`
- `platform/src/os/apple/apple_native_host.rs`
- `platform/src/os/apple/apple_native_label.rs`
- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/mod.rs` if a new module export is required
- `examples/native_text_input/`
- `makepad.splash`

## Acceptance Criteria

- `NativeTextInput` is registered in `mod.widgets`.
- Splash can name it with `input := NativeTextInput{...}` and call
  `ui.input.text()`, `ui.input.set_text(...)`, `ui.input.focus()`,
  `ui.input.blur()`, `ui.input.select_all()`, `ui.input.copy()`,
  `ui.input.cut()`, and `ui.input.paste()`.
- macOS creates an `NSTextField` lazily and attaches it to the Makepad window's
  parent Cocoa view.
- Makepad `Area` is converted to Cocoa coordinates using the same y-axis flip
  strategy as `MacosSystemBrowser`.
- Programmatic `set_text` is guarded with an internal update flag so it does
  not recursively emit `on_change`.
- AppKit changed/focus/selection callbacks are delivered to Rust through
  `Cx::try_post_action(...)`, then converted to widget actions.
- Dropping/hiding the widget detaches the native field from the view hierarchy.
- A non-UI compile check for the affected crates passes.
- Studio runtime verification through `makepad-example-native-text-input`
  passes the smoke checklist in the roadmap before P0B is closed.
