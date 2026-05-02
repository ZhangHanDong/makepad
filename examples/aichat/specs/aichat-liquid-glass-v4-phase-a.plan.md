# Apple Native Liquid Glass Phase A Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the Apple Liquid Glass API matrix and native prototype evidence needed before Makepad implements shared native glass descriptors.

**Architecture:** Phase A is research/prototype only. It validates Apple macOS/iOS Liquid Glass classes, selectors, style values, spacing/morph support, tint behavior, hit-test behavior, and corner/radius behavior without changing aichat, `GlassPanel`, or the Makepad descriptor model.

**Tech Stack:** Apple AppKit/UIKit runtime and SDK probes, Swift prototype snippets, Makepad docs/specs, existing macOS Objective-C runtime lookup patterns in `platform/src/os/apple/macos/macos_window.rs`.

---

## Scope

Phase A implements the first v4 step from [aichat-liquid-glass-v4-apple-native-full.spec.md](aichat-liquid-glass-v4-apple-native-full.spec.md).

Deliverables:

- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- macOS probe/prototype notes for `NSGlassEffectContainerView` and multiple `NSGlassEffectView` children
- iOS/iPadOS probe/prototype notes for `UIGlassContainerEffect`, `UIGlassEffect`, and `UIVisualEffectView`
- runtime logs proving class/selector/property availability or absence
- documented style/tint/hit-test/corner/spacing behavior
- explicit sRGB tint conversion rule for v4.1

Non-goals:

- no `NativeGlassBatch` implementation
- no Makepad widget traversal
- no `GlassContainer` / `GlassPanel` DSL changes
- no aichat native panel integration
- no ShaderBackdrop work
- no runtime substrate switching

## Files

- Create: `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- Create: `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`
- Create: `examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift`
- Create: `examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift`
- Modify: `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md` only if Phase A finds a spec mismatch

Prototype files are evidence artifacts. They are not production Makepad runtime code.

## Reference Inputs

- Apple AppKit `NSGlassEffectView`: <https://developer.apple.com/documentation/appkit/nsglasseffectview>
- Apple AppKit `NSGlassEffectContainerView`: <https://developer.apple.com/documentation/appkit/nsglasseffectcontainerview>
- Apple UIKit `UIGlassEffect`: <https://developer.apple.com/documentation/uikit/uiglasseffect>
- Apple UIKit `UIGlassContainerEffect`: <https://developer.apple.com/documentation/uikit/uiglasscontainereffect>
- Apple SwiftUI `GlassEffectContainer`: <https://developer.apple.com/documentation/swiftui/glasseffectcontainer>
- Local v4 spec: `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- Existing macOS runtime lookup: `platform/src/os/apple/macos/macos_window.rs`
- Existing v3 style mapping evidence: `examples/aichat/specs/aichat-liquid-glass-v1-release-notes.md`

## Validation Policy

- Use direct shell commands for SDK/header/probe compilation.
- Use Makepad Studio remote release runs only if validating a Makepad UI target.
- Do not use raw `cargo run` for Makepad UI validation.
- For Swift prototypes that open UI windows, record the command and result as prototype evidence; do not treat that as Makepad runtime validation.

## Task 1: Create Apple API Matrix Document

**Files:**

- Create: `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`

- [ ] **Step 1: Draft the matrix skeleton**

Create sections:

```markdown
# Apple Liquid Glass API Matrix

## Status

Phase A evidence document. Update whenever SDK/runtime validation changes.

## macOS AppKit

| API | Kind | Required for v4.1 | Availability evidence | Notes |
|---|---|---:|---|---|

## iOS/iPadOS UIKit

| API | Kind | Required for v4.1 | Availability evidence | Notes |
|---|---|---:|---|---|

## SwiftUI Reference Model

| API | Makepad design implication |
|---|---|

## Style Values

| Platform | Style | Raw value | Evidence |
|---|---|---:|---|

## Selectors and Properties

| Platform | Class | Selector/property | Required | Evidence |
|---|---|---|---:|---|

## Color and Units

## Hit-Test Policy

## Open Risks
```

- [ ] **Step 2: Fill known v3 evidence**

Add the current macOS 26 evidence:

```text
NSGlassEffectView.Style.regular.rawValue == 0
NSGlassEffectView.Style.clear.rawValue   == 1
```

Reference `aichat-liquid-glass-v1-release-notes.md`.

- [ ] **Step 3: Add v4.1 required AppKit rows**

Rows must include:

- `NSGlassEffectContainerView`
- `NSGlassEffectView`
- `initWithFrame:`
- `setStyle:`
- `setTintColor:`
- `setCornerRadius:`
- `setContentView:`
- `setSpacing:` if available

- [ ] **Step 4: Add v4.1 required UIKit rows**

Rows must include:

- `UIVisualEffectView`
- `UIGlassEffect`
- `UIGlassContainerEffect`
- style initialization
- tint support
- corner/radius support through layer
- `isUserInteractionEnabled = false` passthrough policy

- [ ] **Step 5: Verify document contains required headings**

Run:

```bash
rg -n "macOS AppKit|iOS/iPadOS UIKit|Style Values|Selectors and Properties|Hit-Test Policy" examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md
```

Expected: each heading is found.

## Task 2: Build macOS Runtime Probe

**Files:**

- Create: `examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift`
- Modify: `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- Modify: `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`

- [ ] **Step 1: Write a command-line AppKit runtime probe**

Create `GlassContainerProbe.swift` with a command-line probe that prints:

```swift
import AppKit
import ObjectiveC.runtime

func probeClass(_ name: String) -> AnyClass? {
    let cls = NSClassFromString(name)
    print("[liquid-glass-probe] class \(name) exists=\(cls != nil)")
    return cls
}

func probeSelector(_ cls: AnyClass?, _ selectorName: String) {
    guard let cls else {
        print("[liquid-glass-probe] selector \(selectorName) exists=false reason=class-missing")
        return
    }
    let sel = NSSelectorFromString(selectorName)
    let instances = class_getInstanceMethod(cls, sel) != nil
    print("[liquid-glass-probe] selector \(selectorName) exists=\(instances)")
}

let glassClass = probeClass("NSGlassEffectView")
let containerClass = probeClass("NSGlassEffectContainerView")

for selector in ["initWithFrame:", "setStyle:", "setTintColor:", "setCornerRadius:", "setContentView:"] {
    probeSelector(glassClass, selector)
}

for selector in ["initWithFrame:", "setSpacing:"] {
    probeSelector(containerClass, selector)
}

if #available(macOS 26.0, *) {
    print("[liquid-glass-probe] style regular raw=0 expected-from-v3")
    print("[liquid-glass-probe] style clear raw=1 expected-from-v3")
}
```

If the target SDK exposes typed `NSGlassEffectView.Style`, add typed raw value prints. If typed APIs are unavailable, keep runtime selector evidence and record that typed SDK binding is unavailable.

- [ ] **Step 2: Run the macOS probe**

Run:

```bash
xcrun swift examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift
```

Expected:

- probe compiles or the compile error is captured
- class existence for `NSGlassEffectView`
- class existence for `NSGlassEffectContainerView`
- selector existence lines

- [ ] **Step 3: Record results**

Append the output to `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md` under:

```markdown
## macOS Runtime Probe

Command:

```bash
xcrun swift examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift
```

Output:

```text
...
```
```

- [ ] **Step 4: Update API matrix**

For every class/selector result, update the AppKit matrix with:

- available / unavailable
- command used
- notes about typed SDK availability

## Task 3: macOS Multi-Panel Prototype Evidence

**Files:**

- Modify: `examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift`
- Modify: `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`
- Modify: `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`

- [ ] **Step 1: Extend prototype to create a container and multiple panel views**

Add a guarded AppKit construction path:

```swift
let rootFrame = NSRect(x: 0, y: 0, width: 640, height: 420)
let panelFrames = [
    NSRect(x: 24, y: 24, width: 180, height: 360),
    NSRect(x: 228, y: 24, width: 388, height: 260),
    NSRect(x: 228, y: 304, width: 388, height: 92),
]
```

The prototype should attempt:

- instantiate `NSGlassEffectContainerView(frame:)` if available
- instantiate three `NSGlassEffectView(frame:)` children
- set style regular/clear where possible
- set `cornerRadius`
- set `tintColor` using sRGB `NSColor`
- set `spacing` if the container responds to `setSpacing:`
- set all native views to input passthrough if AppKit supports a direct override; otherwise document that v4.1 must implement passthrough in the platform manager

- [ ] **Step 2: Run compile/runtime probe**

Run:

```bash
xcrun swift examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift
```

Expected:

- no crash if classes are missing
- if classes exist, construction result logs include container and three panels
- if a selector is missing, the prototype logs a structured missing-selector line

- [ ] **Step 3: Record whether visual UI was launched**

If the prototype opens a window, record:

- command used
- whether a window appeared
- whether multiple glass panels were visible
- whether spacing/morph appeared

Do not treat this as Makepad UI validation.

- [ ] **Step 4: Update matrix**

Update:

- `setSpacing:` behavior
- `setTintColor:` sRGB behavior
- capsule mapping via `cornerRadius = min(width, height) / 2`
- hit-test/passthrough notes

## Task 4: iOS/UIKit Probe Skeleton

**Files:**

- Create: `examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift`
- Modify: `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- Modify: `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`

- [ ] **Step 1: Write UIKit skeleton**

Create a source file containing the intended hierarchy:

```swift
import UIKit

func makeGlassHierarchy(containerRect: CGRect, panelRects: [CGRect]) -> UIView {
    let containerEffect = UIGlassContainerEffect()
    let container = UIVisualEffectView(effect: containerEffect)
    container.frame = containerRect
    container.isUserInteractionEnabled = false

    for rect in panelRects {
        let panelEffect = UIGlassEffect(style: .regular)
        let panel = UIVisualEffectView(effect: panelEffect)
        panel.frame = rect
        panel.layer.cornerRadius = min(rect.width, rect.height) / 2
        panel.layer.masksToBounds = true
        panel.isUserInteractionEnabled = false
        container.contentView.addSubview(panel)
    }

    return container
}
```

If the SDK does not expose these APIs under these names, document the compile error and correct API name in the matrix.

- [ ] **Step 2: Typecheck against iOS SDK**

Run:

```bash
xcrun --sdk iphoneos swiftc -typecheck examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift
```

Expected:

- success if the target SDK exposes `UIGlassEffect` / `UIGlassContainerEffect`
- otherwise compile errors that identify missing names or changed API shape

- [ ] **Step 3: Record results**

Append command output to `apple-liquid-glass-phase-a-runtime-notes.md` under:

```markdown
## iOS SDK Typecheck
```

- [ ] **Step 4: Update matrix**

Update UIKit rows:

- class/type availability
- initializer/property names
- hit-test policy
- corner/radius behavior through `CALayer`
- whether typed API is available in the local SDK

## Task 5: Phase A Acceptance Review

**Files:**

- Modify: `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- Modify: `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`
- Modify: `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md` only if evidence contradicts the frozen spec

- [ ] **Step 1: Verify required evidence exists**

Run:

```bash
rg -n "NSGlassEffectContainerView|NSGlassEffectView|UIGlassContainerEffect|UIGlassEffect|setSpacing|setTintColor|setCornerRadius|Hit-Test|sRGB" examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md
rg -n "macOS Runtime Probe|iOS SDK Typecheck|xcrun swift|xcrun --sdk iphoneos swiftc" examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md
```

Expected:

- every required API appears in the matrix
- runtime/typecheck commands and outputs are recorded

- [ ] **Step 2: Check v4.1 non-goals were preserved**

Run:

```bash
rg -n "NativeGlassBatch|GlassContainer \\{|native: true|aichat native panel|ShaderBackdrop" examples/aichat/prototypes/apple_liquid_glass examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md
```

Expected:

- prototypes may mention `GlassContainer` only as Apple/SwiftUI concept, not Makepad DSL implementation
- no production Makepad descriptor API is added
- no aichat runtime code is changed

- [ ] **Step 3: Summarize Phase A result**

Add a top-level status block to `APPLE-LIQUID-GLASS-API-MATRIX.md`:

```markdown
## Phase A Result

Status: Passed | Blocked | Partial

macOS:
- ...

iOS/iPadOS:
- ...

Decision for Phase B:
- proceed | do not proceed
- blockers:
```

- [ ] **Step 4: Commit**

Run:

```bash
git add examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md \
        examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md \
        examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift \
        examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift
git commit -m "docs: add apple liquid glass phase a evidence"
```

Expected:

- commit includes only Phase A evidence artifacts unless the frozen v4 spec needed an evidence-driven correction

## Completion Criteria

Phase A is complete when:

- `APPLE-LIQUID-GLASS-API-MATRIX.md` exists and covers AppKit, UIKit, SwiftUI reference model, style values, selectors/properties, hit-test policy, color/units, and risks.
- macOS runtime probe output is recorded.
- iOS SDK typecheck output is recorded.
- `NSGlassEffectContainerView` / `NSGlassEffectView` availability is known on the validation machine.
- `UIGlassContainerEffect` / `UIGlassEffect` typed API availability is known for the local iOS SDK.
- `setSpacing:` support is confirmed or explicitly unavailable.
- sRGB tint conversion remains the v4.1 rule.
- No Makepad production runtime, widget DSL, or aichat UI code was changed as part of Phase A.
