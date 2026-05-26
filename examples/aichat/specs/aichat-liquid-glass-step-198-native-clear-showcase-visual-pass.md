# Step 198 - Native Clear Showcase Visual Pass

Date: 2026-05-26

## Context

Step 197 left the current full objective incomplete because the macOS visual
gate still needed fresh system-composited evidence after the native container
`contentView` parenting change. Step 198 records the dedicated clear showcase
target, whose purpose is to inspect Apple native glass without the normal
aichat layout covering the material.

This is a showcase validation, not a full aichat production verdict.

## Runtime Run

Studio remote release run:

```text
RunItem makepad-example-aichat-native-glass-showcase
build_id=[15]
```

The build was launched through the Studio remote bridge after clearing the
previous showcase build. Relevant logs:

```text
[liquid-glass] native-glass-showcase=enabled mode=macOS native clear
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-underlay state=Installed reason=installed-on-proofed-hierarchy
[liquid-glass] compositing-proof=transparent-overlay
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
[liquid-glass] native-container-panel-parent container=0000000000000010 requested=contentView role=container-created-contentView class=NSView
```

The Studio framebuffer screenshot was captured at:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-15-kind-0-req-737-1779776393683.png
```

That screenshot verifies Makepad foreground layout and the resize grip position.
It does not prove AppKit native material by itself.

## Manual Evidence

User-provided system-composited screenshot:

```text
/Users/zhangalex/Desktop/截屏2026-05-26 14.29.40.png
```

Observed result:

- the glass panel is transparent across the broad interior, not only at the
  rounded edge;
- background terminal text is visibly softened through the panel;
- rounded corners and edge highlights are visible;
- the right-bottom resize grip is present and correctly positioned;
- Makepad foreground no longer paints an opaque root over the native material.

## Verdict

The clear showcase target passes the
[Apple Native Liquid Glass Standard](APPLE-NATIVE-LIQUID-GLASS-STANDARD.md)
claim level:

```text
Native Underlay Visible
```

It does not pass the full `Apple Native Liquid Glass` claim level because:

- the target intentionally removes most of the real aichat layout;
- it uses the `AppleNativeUnderlay` backend, not the real
  `AppleNativeInterleave` production route;
- it does not validate native controls, iOS, fullscreen, Stage Manager,
  multi-display, popup/modal, or accessibility behavior;
- it does not prove that the real aichat UI can preserve broad interior native
  material while keeping foreground content readable.

## Next Gate

Run and manually validate the real macOS `AppleNativeInterleave` target against
the same standard. The expected promotion criteria are:

- broad interior native material visible in the real aichat route;
- no diagnostic grid or stripe field;
- foreground text and controls remain readable;
- resize keeps native panel geometry aligned;
- logs show `app-substrate=apple-native-interleave` and State 4.
