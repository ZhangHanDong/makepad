# Apple Native Button Glass Research

## Status

Phase F research deliverable for `aichat-liquid-glass-v4-apple-native-full.spec.md`.

This document records the native-control boundary for `GlassButton` and
`GlassToolbar`. It does not implement native controls.

Local SDK evidence used for this pass:

- macOS SDK: `MacOSX15.5.sdk`
- iOS SDK: `iPhoneOS18.5.sdk`
- Swift: Apple Swift `6.1.2`

The local iOS SDK does not expose the iOS 26 glass types used by the v4 native
plan. `UIGlassEffect` and `UIGlassContainerEffect` remain unavailable in the
local `iPhoneOS18.5.sdk`, matching the Phase A runtime notes.

## Decision

Native button glass is not a `NativeGlassPanelDescriptor`.

`NativeGlassPanelDescriptor` describes passive glass surfaces rendered below the
Makepad Metal view. v4.1 native panels are passthrough and must not consume
input. A native button is an interactive platform control with its own hit-test,
focus, accessibility, action dispatch, hover/pressed state, and control styling.
Those responsibilities conflict with the v4.1 panel compositing model.

Therefore:

- v4.1 and v4.2 `GlassButton` remains Makepad-rendered above native glass panels.
- `GlassToolbar` remains Makepad-rendered grouping above native glass panels.
- Native button glass configuration stays separate from panel descriptors.
- A future native-control backend may add a different descriptor family for
  controls, but it must not reuse passive panel descriptors.

## macOS AppKit Findings

The local `MacOSX15.5.sdk` exposes standard AppKit button configuration through
`NSButton`, `NSButtonCell`, `NSButtonType`, and `NSBezelStyle`.

Relevant AppKit surface:

- `NSButton`
- `NSButtonCell`
- `NSButton.bezelStyle`
- `NSButtonCell.bezelStyle`
- `NSBezelStyle`
- `NSButtonType`

Observed `NSBezelStyle` options in the local SDK include:

- `NSBezelStyleAutomatic`
- `NSBezelStylePush`
- `NSBezelStyleFlexiblePush`
- `NSBezelStyleDisclosure`
- `NSBezelStyleCircular`
- `NSBezelStyleHelpButton`
- `NSBezelStyleSmallSquare`
- `NSBezelStyleToolbar`
- `NSBezelStyleAccessoryBarAction`
- `NSBezelStyleAccessoryBar`
- `NSBezelStylePushDisclosure`
- `NSBezelStyleBadge`

No local `MacOSX15.5.sdk` header evidence was found for a dedicated public
`NSButton` glass bezel style. AppKit still exposes the general `bezelStyle`
model. If a macOS 26 SDK adds a glass-specific button style or configuration,
it needs a new SDK probe before implementation.

Implication for Makepad:

- Do not infer button glass from `NSGlassEffectView`.
- Do not create an `NSGlassEffectView` panel per button.
- Keep Makepad-rendered button visuals for v4.1/v4.2.
- Later native-button work should probe AppKit for a button-specific glass API,
  not panel APIs.

## UIKit Findings

The local `iPhoneOS18.5.sdk` exposes configuration-based buttons through
`UIButton` and `UIButtonConfiguration`.

Relevant UIKit surface:

- `UIButton`
- `UIButtonConfiguration`
- `UIButton.buttonWithConfiguration(_:primaryAction:)`
- `UIButton.configuration`
- `UIButton.configurationUpdateHandler`
- `UIButtonConfiguration.cornerStyle`
- `UIButtonConfiguration.buttonSize`
- `UIButtonConfiguration.borderedButtonConfiguration`
- `UIButtonConfiguration.borderedProminentButtonConfiguration`

Observed `UIButtonConfigurationCornerStyle` options include:

- `Fixed`
- `Dynamic`
- `Small`
- `Medium`
- `Large`
- `Capsule`

The local SDK does not expose `UIGlassEffect` or `UIGlassContainerEffect`, so it
also cannot validate any iOS 26 glass-specific `UIButton.Configuration` option.
This blocks typed native iOS button-glass implementation until an iOS 26 SDK is
available locally.

Implication for Makepad:

- `UIButton.Configuration` is the likely native-control configuration surface
  on iOS/iPadOS.
- `UIButton.Configuration` work must stay separate from `UIGlassEffect` panel
  descriptors.
- In v4.1/v4.2, Makepad should keep `GlassButton` and toolbar controls rendered
  by Makepad above native glass panels.
- A later iOS backend should re-run the SDK probe against an iOS 26 SDK before
  naming concrete glass button configuration fields.

## Makepad Phase F Direction

Phase F should split into two tracks:

1. Makepad-rendered semantic controls.
   - `GlassButton`
   - `GlassToolbar`
   - `GlassSeparator`
   - glass foreground/theme tokens
   - hover/down/focus/disabled styling

2. Future Apple native controls.
   - macOS: probe for button-specific glass appearance beyond `NSBezelStyle`
   - iOS/iPadOS: probe `UIButton.Configuration` on an iOS 26 SDK
   - define a control descriptor if native controls become viable

The first track can proceed now. The second track is blocked on SDK validation
and a separate input-routing design.

## Runtime and Hit-Test Policy

Native panels currently sit below the Metal view and are passthrough. Native
buttons would need to sit above or inside a different native hierarchy to receive
events. That would affect:

- event routing
- focus traversal
- accessibility
- z-order relative to Makepad content
- state synchronization between Makepad widgets and platform controls

For this reason, native button glass must be treated as a later native-control
phase, not as an extension of passive panel descriptors.

## Next Implementation Step

Add Makepad-rendered semantic controls first:

- `GlassButton`: semantic variant of the existing button stack with glass-aware
  tokens and states.
- `GlassToolbar`: grouping wrapper that remains Makepad-rendered in v4.1.
- `GlassSeparator`: material-aware separator colors.

These controls should continue to work on every backend. Apple-native control
integration can be revisited after SDK/API validation.
