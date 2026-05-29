# Native Liquid Glass Example

This example is the standalone macOS native Liquid Glass integration target.
It is intentionally separate from `examples/aichat` so AppKit glass behavior can
be validated without app layout, generated Splash UI, or chat surface opacity
interfering with the visual result.

Run it through Makepad Studio as:

```text
makepad-example-native-liquid-glass
```

The native button action probe can be run separately as:

```text
makepad-example-native-liquid-glass-control-action-probe
```

The native button focus probe can be run separately as:

```text
makepad-example-native-liquid-glass-control-focus-probe
```

On macOS 26, the expected visual result is:

- One transparent borderless window.
- One large `GlassContainer { native: true }` with three native panels.
- A full-window `Clear` native panel.
- A smaller `Regular` native panel in the top-right.
- A capsule-shaped `Clear` native panel near the bottom.
- A five-button AppKit native glass control matrix for `Default`, `Primary`,
  `Utility`, `Icon`, and `Nav` button roles.
- Foreground text rendered by Makepad above the native panels.

The in-window controls update the native descriptors at runtime:

- `Clear` / `Regular` switches the large panel and bottom capsule style.
- `Panels` / `Morph` / `Controls` / `Readability` switch between focused
  validation scenes.
- In `Morph`, `Near` / `Far` / `Overlap` move the native panels through
  container-spacing merge, separated, and mixed-style overlap states.
- `Tint -` / `Tint +` changes the native tint alpha.
- `Spacing -` / `Spacing +` changes `GlassContainer.spacing`.
- `Radius -` / `Radius +` changes the large rounded-rect radius.
- `Reset` restores the default visual tuning.
- `Hide` collapses the controls for visual inspection; `Tune` expands them again.
- In `Controls`, `Disable Primary` / `Enable Primary` toggles the native
  `Primary` button's enabled descriptor while keeping it installed for AppKit
  state validation.
- In `Controls`, pressing any native role button triggers a slow multi-second
  native descriptor pulse by temporarily increasing container spacing, panel
  radius, and tint alpha. This validates the Apple native morph/update path; it
  is not a shader water-ripple effect.

The expected native panel install logs include:

```text
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=3 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

After switching to `Controls`, the native control install log should include:

```text
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=5 controls_visible=5
```

After clicking `Disable Primary`, the `Primary` native control frame log should
remain visible but report `enabled=false`:

```text
[liquid-glass] native-control-frame ... label="Primary" ... enabled=false visible=true
```

For resize validation:

- In `Controls`, resizing the window should keep reinstalling native controls
  with `controls_total=5 controls_visible=5`.
- In `Panels`, `Morph`, or `Readability`, resizing should keep native controls
  intentionally hidden with `controls_total=0 controls_visible=0`.
- Mode switches log `control-resize-validation ...` so those two cases can be
  distinguished from an actual disappearance bug.

The action probe should additionally emit a native control activation path:

```text
[liquid-glass] standalone-native-example configured=true panels=3 mode=Controls
[liquid-glass] backend=apple-native-controls event=perform-click-probe control=... label="Primary"
[liquid-glass] backend=apple-native-controls event=button-action control_id=...
[liquid-glass] standalone-native-example native-control activated label=Primary count=1
[liquid-glass] standalone-native-example native-control pulse=start count=1
```

The focus probe should additionally emit:

```text
[liquid-glass] standalone-native-example configured=true panels=3 mode=Controls
[liquid-glass] backend=apple-native-controls event=focus-probe-result control=... label="Primary" accepts_first_responder=true make_first_responder=true is_first_responder=true
```

Studio `Screenshot` captures only the Metal framebuffer, so it will not show
the AppKit underlay. Use a normal macOS screenshot for visual validation of the
native glass material.

This example validates the Apple native path only. It does not attempt the
cross-platform ShaderBackdrop refraction/warp path.
