# Apple Liquid Glass API Matrix

## Status

Phase A evidence document. Update whenever SDK/runtime validation changes.

## Phase G SDK Evidence Update

Date: 2026-05-08

Official Apple documentation now lists the UIKit/AppKit Liquid Glass APIs that
the v4 spec targets:

- `UIGlassEffect`: <https://developer.apple.com/documentation/UIKit/UIGlassEffect>
- `UIGlassContainerEffect`: <https://developer.apple.com/documentation/UIKit/UIGlassContainerEffect>
- `NSGlassEffectView`: <https://developer.apple.com/documentation/appkit/nsglasseffectview>
- UIKit updates: <https://developer.apple.com/documentation/Updates/UIKit>
- AppKit updates: <https://developer.apple.com/documentation/updates/appkit>

Current local implementation gate remains blocked by SDK availability:

- `xcrun --sdk iphoneos --show-sdk-path`:
  `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS18.5.sdk`
- `xcrun --sdk iphonesimulator --show-sdk-path`:
  `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator18.5.sdk`
- `xcrun --sdk macosx --show-sdk-path`:
  `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.5.sdk`

Header scan command:

```bash
rg -n "UIGlassEffect|UIGlassContainerEffect|GlassEffect|glassEffect|NSGlassEffectView|NSGlassEffectContainerView" \
  "$(xcrun --sdk iphoneos --show-sdk-path)" \
  "$(xcrun --sdk iphonesimulator --show-sdk-path)" \
  "$(xcrun --sdk macosx --show-sdk-path)"
```

Result: the local SDK headers do not expose `UIGlassEffect`,
`UIGlassContainerEffect`, `NSGlassEffectView`, or
`NSGlassEffectContainerView`. The macOS runtime still exposes the AppKit
classes on the validation OS, which is why the current macOS backend uses
runtime Objective-C lookup instead of typed SDK bindings.

Phase G consequence: keep iOS runtime-gated. Step 68 adds Objective-C runtime
class preflight without using typed iOS 26 SDK symbols. Step 71 adds a dynamic
UIKit installer skeleton that only creates native glass views after class and
selector preflight passes.

## Phase A Result

Status: Partial

macOS:

- Passed on the current validation runtime. `NSGlassEffectContainerView` and `NSGlassEffectView` exist; required selectors exist; one container with three child glass panels constructs successfully.

iOS/iPadOS:

- Blocked on the local SDK. `xcrun --sdk iphoneos --show-sdk-version` reports `18.5`, and that SDK does not expose `UIGlassEffect` or `UIGlassContainerEffect`.
- Step 68 adds runtime class preflight for `UIVisualEffectView`,
  `UIGlassContainerEffect`, and `UIGlassEffect`.
- Step 69 prepares the UIKit underlay host view by making the root controller
  view a plain `UIView` containing the existing `MTKView`.
- Step 70 adds Objective-C selector preflight for the planned UIKit backend:
  `initWithEffect:`, `contentView`, `initWithStyle:`, `setTintColor:`,
  `setInteractive:`, and `setSpacing:`.
- Step 71 adds the first UIKit installer skeleton: when preflight passes it
  creates a `UIGlassContainerEffect`, inserts a `UIVisualEffectView` container
  below `MTKView`, and adds passthrough panel `UIVisualEffectView`s backed by
  `UIGlassEffect`.
- Step 73 extends selector preflight to the UIView/CALayer selectors used by
  the installer before it attaches native glass views.

Decision for Phase B:

- Treat iOS as implemented only to the installer-skeleton level until an iOS 26
  SDK/runtime validates class availability, selector names, raw style values,
  visual output, rotation, safe area, keyboard, split view, and Stage Manager.

## macOS AppKit

| API | Kind | Required for v4.1 | Availability evidence | Notes |
|---|---|---:|---|---|
| `NSGlassEffectView` | class | yes | Phase A macOS probe: exists=true | Single native glass panel surface. |
| `NSGlassEffectContainerView` | class | yes | Phase A macOS probe: exists=true | Native multi-panel grouping/spacing container. |
| `NSGlassEffectView.Style.regular` | style | yes | v3 runtime validation: raw `0` | Keep env override until SDK source is confirmed. |
| `NSGlassEffectView.Style.clear` | style | yes | v3 runtime validation: raw `1` | Keep env override until SDK source is confirmed. |
| `NSColor` sRGB tint | color conversion | yes | Phase A macOS probe: `setTintColor:` accepts `NSColor(srgbRed:green:blue:alpha:)` | v4.1 tint conversion uses sRGB to match Makepad color assumptions. |
| `NSView` hit-test passthrough | behavior | yes | Policy confirmed, implementation not prototyped | v4.1 platform backend must keep native panels passthrough, likely by view layering and/or hit-test override. |
| `CALayer.cornerRadius` / `setCornerRadius:` | shape | yes | Phase A macOS probe: both layer corner radius and `setCornerRadius:` used successfully | `Capsule` maps to `min(width, height) / 2`. |

## iOS/iPadOS UIKit

| API | Kind | Required for v4.1 | Availability evidence | Notes |
|---|---|---:|---|---|
| `UIVisualEffectView` | class | yes | Local iPhoneOS 18.5 SDK typecheck reaches UIKit; Step 68 runtime preflight checks class presence | UIKit host for visual effects. |
| `UIGlassEffect` | type | yes | Official Apple docs list `class UIGlassEffect`; local iPhoneOS 18.5 SDK typed symbols unavailable; Step 68 runtime preflight checks class presence | Step 71 uses dynamic runtime lookup; requires iOS 26 runtime validation. |
| `UIGlassContainerEffect` | type | yes | Official Apple docs list `class UIGlassContainerEffect`; local iPhoneOS 18.5 SDK typed symbols unavailable; Step 68 runtime preflight checks class presence | Step 71 uses dynamic runtime lookup; requires iOS 26 runtime validation. |
| `UIGlassEffect(style:)` | initializer | yes | Official Apple docs list `init(style: UIGlassEffect.Style)`; local SDK unavailable | Step 71 calls `initWithStyle:` dynamically after selector preflight. |
| `UIGlassEffect.isInteractive` | property | future | Official Apple docs list `isInteractive` | v4.1 keeps native panels passthrough; interactive remains future work. |
| `UIGlassEffect.tintColor` | property | yes | Official Apple docs list `tintColor` | v4.1 tint conversion remains sRGB. |
| `UIGlassContainerEffect.spacing` | property | yes | Official Apple docs list `spacing` | Maps to `GlassContainer.spacing` morph distance. |
| `UIView.isUserInteractionEnabled = false` | hit-test policy | yes | UIKit skeleton typecheck blocked by missing glass types; property itself is existing UIKit API | v4.1 panels are passthrough. |
| `CALayer.cornerRadius` | shape | yes | UIKit skeleton typecheck blocked by missing glass types; property itself is existing UIKit API | `Capsule` maps to `min(width, height) / 2`. |
| `UIButton` | native control | future | Step 147 runtime preflight checks class and ordinary button selectors dynamically; Step 148 compiles a UIKit installer skeleton | Native controls remain an opt-in future path, separate from visual panels. |
| `UIButtonConfiguration.glass()` / `clearGlass()` | native control style | future | Official Apple docs list Liquid Glass button configuration; Step 147 checks ObjC selectors `glassButtonConfiguration` and `clearGlassButtonConfiguration` dynamically; Step 148 applies those configurations dynamically | iOS 26 runtime validation remains open. |

## SwiftUI Reference Model

| API | Makepad design implication |
|---|---|
| `Glass` | Makepad needs a semantic glass style, not only a shader fill. |
| `glassEffect(_:in:)` | Makepad `GlassPanel` should export a native descriptor when explicitly opted in. |
| `GlassEffectContainer` | Makepad `GlassContainer` maps to native grouping/spacing semantics. |
| `GlassButtonStyle` / prominent glass button styles | Makepad `GlassButton` research belongs to Phase F, separate from panel descriptors. |
| Material/vibrancy environment | Makepad needs glass-aware foreground, separator, and contrast tokens. |

## Style Values

| Platform | Style | Raw value | Evidence |
|---|---|---:|---|
| macOS | regular | `0` | Verified in v3 runtime logs; see `aichat-liquid-glass-v1-release-notes.md`. |
| macOS | clear | `1` | Verified in v3 runtime logs; see `aichat-liquid-glass-v1-release-notes.md`. |
| iOS/iPadOS | regular | Assumed `0` | Step 71 skeleton uses the same declaration order as macOS; Step 72 adds `AICHAT_IOS_GLASS_STYLE_REGULAR_RAW` for runtime override; must be confirmed on iOS 26 runtime. |
| iOS/iPadOS | clear | Assumed `1` | Step 71 skeleton uses the same declaration order as macOS; Step 72 adds `AICHAT_IOS_GLASS_STYLE_CLEAR_RAW` for runtime override; must be confirmed on iOS 26 runtime. |

## Selectors and Properties

| Platform | Class | Selector/property | Required | Evidence |
|---|---|---|---:|---|
| macOS | `NSGlassEffectView` | `initWithFrame:` | yes | Phase A macOS probe: exists=true |
| macOS | `NSGlassEffectView` | `setStyle:` | yes | Phase A macOS probe: exists=true; send succeeds for raw `0` and `1` |
| macOS | `NSGlassEffectView` | `setTintColor:` | yes | Phase A macOS probe: exists=true; send succeeds with sRGB `NSColor` |
| macOS | `NSGlassEffectView` | `setCornerRadius:` | yes | Phase A macOS probe: exists=true; send succeeds |
| macOS | `NSGlassEffectView` | `setContentView:` | preferred | Phase A macOS probe: exists=true |
| macOS | `NSGlassEffectContainerView` | `initWithFrame:` | yes | Phase A macOS probe: exists=true |
| macOS | `NSGlassEffectContainerView` | `setSpacing:` | yes if present | Phase A macOS probe: exists=true; send succeeds with `20.0` |
| iOS/iPadOS | `UIGlassContainerEffect` | initializer | yes | Step 71 calls `alloc/init` dynamically after class preflight. |
| iOS/iPadOS | `UIGlassEffect` | `init(style:)` | yes | Step 71 calls `initWithStyle:` dynamically after selector preflight. |
| iOS/iPadOS | `UIVisualEffectView` | `init(effect:)` | yes | Step 71 creates container and panel visual effect views dynamically. |
| iOS/iPadOS | `UIView` / `UIVisualEffectView` / `CALayer` | frame, input, hierarchy, and corner selectors | yes | Step 73 preflights installer selectors including `setFrame:`, `setUserInteractionEnabled:`, `insertSubview:belowSubview:`, `addSubview:`, `layer`, `setMasksToBounds:`, and `setCornerRadius:`. |

## Color and Units

- Descriptor geometry uses Makepad logical units. Platform backends convert to AppKit/UIKit coordinates and backing scale at the boundary.
- v4.1 tint conversion uses sRGB.
- `Capsule` is represented with `cornerRadius = min(width, height) / 2`.

## Hit-Test Policy

- v4.1 native glass panels are passthrough only.
- `NativeGlassHitTest::Interactive` exists for future descriptor shape but must be rejected or downgraded during v4.1.
- Makepad remains the input owner.

## Open Risks

- Apple API names may differ from current assumptions in the local SDK.
- `NSGlassEffectContainerView` or `UIGlassContainerEffect` may be runtime-present but not typed in the local SDK.
- `setSpacing:` may be absent or named differently.
- Native passthrough behavior may require subclassing or hit-test override rather than simple property configuration.
- sRGB tint may visually diverge from Display P3 system colors; v4.1 deliberately chooses sRGB for Makepad consistency.
