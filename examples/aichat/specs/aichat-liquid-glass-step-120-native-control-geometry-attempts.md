# AI Chat Liquid Glass Step 120 Native Control Geometry Attempts

Date: 2026-05-09

## Context

Step 119 hierarchy logs showed that the macOS native control mirrors are topmost
siblings, but their frames do not match the Studio `WidgetQuery` button rects.

Build `[117]` / `[119]` hierarchy evidence:

```text
clear_button WidgetQuery: Button 828 564 78 36
native clear frame:       (695.5,474.0,78.0,36.0)

send_button WidgetQuery:  Button 914 560 44 44
native send frame:        (781.5,466.0,44.0,44.0)
```

The native controls are installed above the Metal view:

```text
hierarchy index=1 role=metal-view class=RenderViewClass frame=(0.0,0.0,900.0,700.0) hidden=false
hierarchy index=2 role=native-control class=NativeGlassButton frame=(695.5,474.0,78.0,36.0) hidden=false
hierarchy index=3 role=native-control class=NativeGlassButton frame=(781.5,466.0,44.0,44.0) hidden=false
```

## Attempt 1: clipped rect

Changing Button native-control geometry from raw `area().rect(cx)` to
`area().clipped_rect(cx)` produced an empty control rect at runtime.

Build `[118]` evidence:

```text
[liquid-glass] backend=apple-native-controls state=Rejected reason=empty-visible-control-rect controls_total=2 controls_visible=2
```

The change was not retained.

## Attempt 2: view-origin transform

Adding draw-list `view_clip.xy` and `view_shift` to the raw rect produced invalid
offscreen frames.

Build `[120]` evidence:

```text
hierarchy index=2 role=native-control class=NativeGlassButton frame=(-99304.5,100474.0,78.0,36.0) hidden=false
hierarchy index=3 role=native-control class=NativeGlassButton frame=(-99218.5,100466.0,44.0,44.0) hidden=false
```

The change was not retained.

## Result

Native control geometry remains unresolved. The retained implementation is the
pre-existing raw `area().rect(cx)` descriptor plus Step 119 hierarchy logging.

Build `[121]` confirms the retained implementation is back to installable state
and still shows the same mismatch:

```text
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
hierarchy index=2 role=native-control class=NativeGlassButton frame=(695.5,474.0,78.0,36.0) hidden=false
hierarchy index=3 role=native-control class=NativeGlassButton frame=(781.5,466.0,44.0,44.0) hidden=false
```

Next investigation should trace how Studio `WidgetQuery` derives its rects and
compare that path to `Area::rect(cx)` for the same `Button` area, rather than
guessing additional transforms.
