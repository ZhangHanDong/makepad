spec: task
name: "AI Chat Liquid Glass Step 119 Native Control Hierarchy Dump"
tags: [makepad, macos, liquid-glass, native-controls, diagnostics]
---

## Intent

Log the AppKit container subview hierarchy after macOS native control
installation so native-control debugging can verify sibling order, class names,
frames, and hidden state without relying on click automation.

## Decisions

- The hierarchy dump runs after native control batch installation.
- Each subview log includes index, role, Objective-C class name, frame, and
  hidden state.
- Roles distinguish the Makepad Metal view, native controls, and other sibling
  views.
- The dump is diagnostic only and does not change view ordering.

## Boundaries

- Do not alter native glass panel hierarchy.
- Do not change native control descriptors.
- Do not synthesize click events.
- Do not mark native controls complete from hierarchy evidence alone.

## Out of Scope

- UIKit hierarchy dumps.
- Accessibility tree dumps.
- macOS 26 glass button styling.
- Runtime view hierarchy UI tooling.

## Acceptance Criteria

### Scenario: hierarchy dump logs subview count

Test: `rg "hierarchy subviews" platform/src/os/apple/macos/macos_window.rs`

Given a native control batch is installed
When the macOS backend finishes installation
Then it logs the AppKit container subview count.

### Scenario: hierarchy dump logs roles and frames

Test: `rg "role=.*frame=" platform/src/os/apple/macos/macos_window.rs`

Given the AppKit container has subviews
When dumping the native control hierarchy
Then each subview log includes role, class, frame, and hidden state.

### Scenario: hierarchy dump does not replace click validation

Test: `rg "hierarchy|real AppKit click|target-action" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

Given hierarchy diagnostics exist
When auditing completion
Then native controls still require real AppKit click and target/action evidence.
