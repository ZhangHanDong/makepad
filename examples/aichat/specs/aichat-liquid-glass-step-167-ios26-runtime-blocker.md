# Step 167 - iOS 26 Runtime Blocker

Date: 2026-05-10

## Context

The next full Apple-native Liquid Glass gate is UIKit runtime validation for:

- `UIGlassEffect`
- `UIGlassContainerEffect`
- `UIVisualEffectView` hierarchy behavior
- native UIKit glass button configuration
- rotation, safe area, keyboard, split view, and Stage Manager behavior

The current local machine cannot run that gate because it does not have an iOS
26 SDK or simulator runtime installed.

## Local Environment Evidence

Command:

```text
xcrun --sdk iphoneos --show-sdk-version
```

Output:

```text
18.5
```

Command:

```text
xcrun simctl list runtimes | rg -n "iOS|unavailable|26|18|19"
```

Output:

```text
2:iOS 18.6 (18.6 - 22G86) - com.apple.CoreSimulator.SimRuntime.iOS-18-6
```

## Verdict

The iOS native Liquid Glass backend remains compile/preflight-only on this
machine. Runtime validation is blocked until an Xcode installation with iOS 26
SDK and an iOS 26 simulator runtime is available.

Current validated scope:

- iOS target compile checks
- Objective-C runtime preflight code paths
- selector/style override plumbing
- UIKit installer skeleton compilation
- stable unsupported logs for transient popup probes

Unvalidated scope:

- actual `UIGlassEffect` / `UIGlassContainerEffect` class availability
- real style raw values
- visual output
- rotation/safe-area/keyboard behavior
- split view / Stage Manager behavior
- UIKit native-control hit testing
- UIKit accessibility/focus behavior

Installing an iOS 26 simulator is not an in-place upgrade of the existing iOS
18.x runtime. It requires an Xcode version that ships or downloads the iOS 26
SDK/runtime, then a new simulator device created against that runtime.
