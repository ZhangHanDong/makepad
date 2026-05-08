spec: task
name: "AI Chat Liquid Glass Step 45 Phase G SDK Evidence"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, docs]
---

## Intent

Record the current Phase G SDK evidence so the UIKit backend does not proceed
from guessed API names. The evidence must distinguish official Apple
documentation for Liquid Glass APIs from the local Xcode SDK that is currently
available to this branch.

## Decisions

- Treat Apple Developer Documentation as evidence that the UIKit/AppKit 26
  symbols exist.
- Treat local Xcode SDK header scans as the implementation gate for compiling
  native UIKit backend code in this checkout.
- Keep the iOS backend in explicit unsupported fallback until an iOS 26 SDK is
  locally available.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`
- this step spec

### Constraints

- Do not edit platform code.
- Do not claim the UIKit backend is implemented.
- Do not replace local SDK evidence with web documentation; both are required.

## Acceptance Criteria

### Scenario: API matrix records official UIKit docs
Given the Apple Liquid Glass API matrix
When it is inspected
Then it links to official Apple docs for `UIGlassEffect` and `UIGlassContainerEffect`
And it records `isInteractive`, `tintColor`, and `spacing`
Test: `rg "developer\\.apple\\.com/documentation/UIKit/UIGlassEffect|developer\\.apple\\.com/documentation/UIKit/UIGlassContainerEffect|isInteractive|tintColor|spacing" examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`

### Scenario: API matrix records local SDK gate
Given the local SDK scan
When the API matrix is inspected
Then it records iPhoneOS 18.5 and MacOSX 15.5 as lacking the new typed symbols
Test: `rg "iPhoneOS18\\.5|MacOSX15\\.5|local SDK headers do not expose" examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`

### Scenario: runtime notes include reproducible commands
Given Phase G remains blocked by local SDK availability
When the runtime notes are inspected
Then they include the xcrun SDK path commands and the header scan command
Test: `rg "xcrun --sdk iphoneos --show-sdk-path|rg -n \\\"UIGlassEffect\\|UIGlassContainerEffect\\|NSGlassEffectView" examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`

### Scenario: no code changes
Given this is an evidence update
When the diff is inspected
Then no source file outside `examples/aichat/specs` changed
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/|^\\.makepad/"` 
