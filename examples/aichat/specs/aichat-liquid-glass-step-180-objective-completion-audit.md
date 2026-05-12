# Step 180 - Objective Completion Audit

Date: 2026-05-12

## Objective

Complete support for Apple's native Liquid Glass API.

For this branch, that objective means all of the following must be true before
completion can be claimed:

1. macOS AppKit native glass panels can be installed from Makepad descriptors.
2. macOS native container spacing/morph semantics are represented and observable.
3. macOS text and controls remain readable over light and dark backgrounds.
4. aichat can show full-window native glass material, not only edge halo.
5. native interactive controls have a defined and validated policy.
6. fullscreen, Stage Manager, multi-display, popup/modal, and runtime switching
   are either implemented or explicitly scoped as known limitations.
7. iOS/iPadOS UIKit Liquid Glass APIs are implemented or runtime-gated with
   validated behavior on an iOS 26 runtime.
8. docs, release notes, known limitations, and validation evidence match the
   actual implementation state.

## Prompt-to-Artifact Checklist

| Requirement | Current Evidence | Status |
| --- | --- | --- |
| macOS reaches native State 4 | Step 179 build `[32]` logs `state=4 substrate=macos-native style=clear style_raw=1`; four panels install with `panels_installed=4 panels_failed=0`. | Verified for guarded macOS targets |
| native AppKit hierarchy exists | Step 179 logs lower Metal sibling, `NSGlassEffectContainerView`, then primary `RenderViewClass` `CAMetalLayer`. | Verified |
| foreground does not cover native glass with opaque Metal | Step 179 Studio screenshot alpha histogram: `zero_alpha=589869`, `opaque_alpha=1706` for 900x700. | Verified for preview foreground |
| full-window native glass visibility | Step 178 user screenshot passes diagnostic interleave: lower diagnostic scene visible across interior, not only edges. | Verified only for diagnostic scene |
| production full-window native glass | Step 179 adds production preview with `grid_strength=0.000`. It still requires a manual system-composited verdict because Studio cannot capture AppKit material. | Incomplete |
| macOS native container spacing | Prior steps log `native-container-spacing ... spacing=20.000` and animated spacing rerun keeps native batch installed. | Partially verified; final visual morph quality still open |
| native interactive controls | macOS Clear button system-click gate passed in Step 165; iOS native-control path is skeleton/preflight only. | Partially verified |
| iOS/iPadOS native glass | Local environment rechecked on 2026-05-12: `xcodebuild -showsdks` reports iOS/iOS Simulator 18.5; `simctl` has iOS 18.6 runtime only. | Blocked locally; not complete |
| fullscreen / Stage Manager / multi-display | Fullscreen fallback/restore probe exists; Stage Manager and multi-display remain known limitations. | Not complete |
| popup/modal native glass | macOS transient probe and dismiss routes exist; UIKit transient and separate native modal glass remain incomplete. | Not complete |
| release notes / limitations | v4.1 release notes and completion audit identify non-goals and gaps. | Current, but must be updated if route changes |

## Local Environment Recheck

Commands:

```text
xcodebuild -showsdks
xcrun simctl list runtimes
xcrun --sdk iphoneos --show-sdk-version
xcrun --sdk iphonesimulator --show-sdk-version
```

Results:

```text
iOS SDKs: iOS 18.5
iOS Simulator SDKs: Simulator - iOS 18.5
simctl runtimes: iOS 18.6
iphoneos SDK version: 18.5
iphonesimulator SDK version: 18.5
```

This machine still cannot validate UIKit `UIGlassEffect`,
`UIGlassContainerEffect`, iOS style raw values, iOS native controls, rotation,
keyboard, split view, or Stage Manager behavior on iOS 26.

## Verdict

The objective is not complete.

The nearest macOS task is to manually validate the Step 179 production preview
in the real system-composited window. If the production preview is too subtle,
the next implementation slice should tune the lower production scene content so
native glass has visible detail to refract without reintroducing diagnostic
grid/stripe artifacts.

The iOS/iPadOS portion remains environment-blocked until an iOS 26 SDK/runtime
or device is available.
