# Apple Liquid Glass Phase A Runtime Notes

## Status

Phase A evidence log. Append command output verbatim enough to make API availability and failures reproducible.

## macOS Runtime Probe

Command:

```bash
xcrun swift examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift
```

Output:

```text
[liquid-glass-probe] class NSGlassEffectView exists=true
[liquid-glass-probe] class NSGlassEffectContainerView exists=true
[liquid-glass-probe] selector initWithFrame: exists=true
[liquid-glass-probe] selector setStyle: exists=true
[liquid-glass-probe] selector setTintColor: exists=true
[liquid-glass-probe] selector setCornerRadius: exists=true
[liquid-glass-probe] selector setContentView: exists=true
[liquid-glass-probe] selector initWithFrame: exists=true
[liquid-glass-probe] selector setSpacing: exists=true
[liquid-glass-probe] style regular raw=0 expected-from-v3
[liquid-glass-probe] style clear raw=1 expected-from-v3
```

## macOS Multi-Panel Prototype

Same command:

```bash
xcrun swift examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift
```

Output:

```text
[liquid-glass-probe] construct NSGlassEffectContainerView ok=true frame=(0.0, 0.0, 640.0, 420.0)
[liquid-glass-probe] send setSpacing: ok=true value=20.0
[liquid-glass-probe] construct NSGlassEffectView[0] ok=true frame=(24.0, 24.0, 180.0, 360.0)
[liquid-glass-probe] send setCornerRadius: ok=true value=90.0
[liquid-glass-probe] send setStyle: ok=true value=0
[liquid-glass-probe] send setTintColor: ok=true value=NSColorSpaceColor
[liquid-glass-probe] construct NSGlassEffectView[1] ok=true frame=(228.0, 24.0, 388.0, 260.0)
[liquid-glass-probe] send setCornerRadius: ok=true value=130.0
[liquid-glass-probe] send setStyle: ok=true value=1
[liquid-glass-probe] send setTintColor: ok=true value=NSColorSpaceColor
[liquid-glass-probe] construct NSGlassEffectView[2] ok=true frame=(228.0, 304.0, 388.0, 92.0)
[liquid-glass-probe] send setCornerRadius: ok=true value=46.0
[liquid-glass-probe] send setStyle: ok=true value=1
[liquid-glass-probe] send setTintColor: ok=true value=NSColorSpaceColor
[liquid-glass-probe] native-container construct ok=true panels=3
```

Visual UI launched: no. This is a command-line runtime construction probe only, not Makepad UI validation.

## iOS SDK Typecheck

SDK version command:

```bash
xcrun --sdk iphoneos --show-sdk-version
```

Output:

```text
18.5
```

Initial command from the plan:

```bash
xcrun --sdk iphoneos swiftc -typecheck examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift
```

Output:

```text
<unknown>:0: warning: using sysroot for 'iPhoneOS' but targeting 'MacOSX'
<unknown>:0: error: unable to load standard library for target 'arm64-apple-macosx16.0'
```

Corrected command with explicit iOS target:

```bash
xcrun --sdk iphoneos swiftc -target arm64-apple-ios18.5 -typecheck examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift
```

Output:

```text
examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift:4:27: error: cannot find 'UIGlassContainerEffect' in scope
 2 | 
 3 | func makeGlassHierarchy(containerRect: CGRect, panelRects: [CGRect]) -> UIView {
 4 |     let containerEffect = UIGlassContainerEffect()
   |                           `- error: cannot find 'UIGlassContainerEffect' in scope
 5 |     let container = UIVisualEffectView(effect: containerEffect)
 6 |     container.frame = containerRect

examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift:10:27: error: cannot find 'UIGlassEffect' in scope
 8 | 
 9 |     for rect in panelRects {
10 |         let panelEffect = UIGlassEffect(style: .regular)
   |                           `- error: cannot find 'UIGlassEffect' in scope
11 |         let panel = UIVisualEffectView(effect: panelEffect)
12 |         panel.frame = rect

examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift:10:49: error: cannot infer contextual base in reference to member 'regular'
 8 | 
 9 |     for rect in panelRects {
10 |         let panelEffect = UIGlassEffect(style: .regular)
   |                                                 `- error: cannot infer contextual base in reference to member 'regular'
11 |         let panel = UIVisualEffectView(effect: panelEffect)
12 |         panel.frame = rect
```

Result: the local iPhoneOS 18.5 SDK does not expose `UIGlassEffect` or `UIGlassContainerEffect`. UIKit backend implementation remains blocked until an iOS 26 SDK is available.

## Phase G SDK Header Scan

SDK path commands:

```bash
xcrun --sdk iphoneos --show-sdk-path
xcrun --sdk iphonesimulator --show-sdk-path
xcrun --sdk macosx --show-sdk-path
```

Output:

```text
/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS18.5.sdk
/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator18.5.sdk
/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.5.sdk
```

Header scan command:

```bash
rg -n "UIGlassEffect|UIGlassContainerEffect|GlassEffect|glassEffect|NSGlassEffectView|NSGlassEffectContainerView" \
  "$(xcrun --sdk iphoneos --show-sdk-path 2>/dev/null)" \
  "$(xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null)" \
  "$(xcrun --sdk macosx --show-sdk-path 2>/dev/null)"
```

Output:

```text
<no matches>
```

Official Apple Developer Documentation does list these APIs for the newer
platform SDKs:

- `UIGlassEffect`: <https://developer.apple.com/documentation/UIKit/UIGlassEffect>
- `UIGlassContainerEffect`: <https://developer.apple.com/documentation/UIKit/UIGlassContainerEffect>
- `NSGlassEffectView`: <https://developer.apple.com/documentation/appkit/nsglasseffectview>

Result: Phase G has enough public documentation to keep the descriptor mapping
accurate, but this checkout cannot compile or type-check a UIKit backend until a
newer local SDK exposes the symbols or we add a runtime Objective-C lookup path
validated on an iOS 26 runtime.
