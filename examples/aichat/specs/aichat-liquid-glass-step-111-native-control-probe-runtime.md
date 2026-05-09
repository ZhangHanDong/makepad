# AI Chat Liquid Glass Step 111 Native Control Probe Runtime

Date: 2026-05-09

## Run

Studio remote RunItem:

```text
makepad-example-aichat-macos-native-clear-control-probe
```

Build id:

```text
[111]
```

This runnable comes from `makepad.splash` and passes:

```text
AICHAT_GLASS_BACKEND=macos-native-clear
AICHAT_NATIVE_CONTROL_PROBE=buttons
```

## Evidence

Relevant Studio/child-app log lines:

```text
[liquid-glass] native-control-probe=buttons-enabled ids=clear_button,send_button
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-underlay state=Installed reason=installed-on-proofed-hierarchy
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

Widget query evidence:

```text
clear_button Button 828 564 78 36
send_button Button 914 560 44 44
```

Studio framebuffer screenshot path:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-111-kind-0-req-82-1778326023007.png
```

## Result

The macOS native-control pipeline reached the platform installer:

- aichat opt-in probe enabled descriptors for `clear_button` and `send_button`
- the macOS backend installed two AppKit `NSButton` mirrors
- the native underlay stayed installed with four panels

## Limits

This does not complete native interactive controls:

- Studio click injection goes through Makepad app events and does not prove
  real AppKit `NSButton` target/action delivery.
- The Studio framebuffer screenshot does not reliably capture AppKit overlay
  views; system/manual visual inspection is still needed.
- macOS 26 glass-specific `NSButton` styling remains unproven.
- UIKit native controls are not implemented.
- Accessibility ownership is not implemented.
