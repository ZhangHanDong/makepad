# Step 121: Native Control Studio Coordinate Evidence

Date: 2026-05-09

## Result

The macOS native-control probe still must not be marked complete. Step 121 adds
runtime evidence that separates three coordinate spaces which were previously
compared as if they were the same:

- Makepad/AppKit child-window local control rects
- Studio `WidgetQuery` / `Click` coordinates
- Studio framebuffer screenshots

## Evidence

Studio release build `[125]` for
`makepad-example-aichat-macos-native-clear-control-probe` logged:

```text
[liquid-glass] native-control-frame control=0000000000000043 kind=Button { role: Default } label="Clear" makepad=(695.5,190.0,78.0,36.0) appkit=(695.5,474.0,78.0,36.0) z_order=0 visible=true
[liquid-glass] native-control-frame control=0000000000000044 kind=Button { role: Default } label="↑" makepad=(781.5,190.0,44.0,44.0) appkit=(781.5,466.0,44.0,44.0) z_order=0 visible=true
```

The label field confirms the installed AppKit controls are the intended
`clear_button` and `send_button`, not unrelated toolbar buttons.

Studio `WidgetQuery` for the same runnable reports coordinates in the Studio
remote input space, for example:

```text
clear_button Button 828 564 78 36
send_button Button 914 560 44 44
```

These numbers should not be directly compared to AppKit subview frames. They
include Studio run-view/input-space placement, while `NSView.frame` is local to
the child window's AppKit container.

Studio `Screenshot` responses capture the Metal framebuffer. They do not include
native AppKit sibling controls inserted into the child window hierarchy, so a
Studio screenshot alone cannot prove native-control visual alignment.

## Attempted Non-Fixes

- `Area::clipped_rect(cx)` remains unsuitable for native controls here; earlier
  testing rejected the batch as `empty-visible-control-rect`.
- Adding only draw-list `view_shift` did not change the native control rect in
  this path; logged `view_shift` was `(0.0, 0.0)`.
- A macOS `screencapture` attempt in this session returned an all-black image,
  so it did not provide usable visual evidence.

## Next Requirement

Native-control completion still needs either:

- a reliable system-level screenshot/capture path that includes AppKit sibling
  views, or
- an AppKit-side verification path that maps the installed native controls to
  the rendered Makepad button geometry without relying on Studio framebuffer
  screenshots.

