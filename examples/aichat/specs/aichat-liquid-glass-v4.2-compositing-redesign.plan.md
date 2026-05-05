# aichat Liquid Glass v4.2 Compositing Redesign Plan

## Goal

Prove whether Makepad can deliver recognizable interior Liquid Glass with Apple
native APIs, or whether complete interiors must move to `ShaderBackdrop`.

## Scope

This plan is for evidence gathering and architecture selection. It should not
rewrite aichat production UI until a prototype proves the compositing route.

## Task 1: Above-Metal Sampling Probe

Files:

- Create: `examples/aichat/specs/aichat-liquid-glass-step-15-above-metal-probe.spec`
- Create or modify a small macOS-only proof target/module under the existing
  Makepad examples or platform test area.

Steps:

- [x] Add a task spec for an above-Metal sampling probe.
- [x] Build a release Studio runnable that draws a moving high-contrast Metal
      pattern.
- [x] Add one `NSGlassEffectView` above that Metal layer.
- [x] Verify through Studio run logs that the glass view is installed.
- [x] Manually inspect whether the glass surface visibly samples the Metal
      pattern.
- [x] Record result and screenshot limitation notes.
- [x] Commit.

Acceptance:

- The probe can distinguish "native glass samples Metal content" from "native
  glass is only a flat overlay/edge effect."

## Task 2: Two-Layer Interleave Feasibility Probe

Files:

- Create: `examples/aichat/specs/aichat-liquid-glass-step-16-two-layer-interleave-probe.spec`
- Update route documentation based on the Task 1 evidence and current renderer
  constraints.

Steps:

- [x] Add a task spec for the interleave feasibility decision.
- [x] Record that true `AppleNativeInterleave` requires two Makepad-rendered
      surfaces or passes.
- [x] Reject a native-only upper label/control proof as fake Makepad
      interleave evidence.
- [x] Record that the current macOS renderer has one primary Makepad
      `CAMetalLayer`.
- [x] Close the route decision without selecting `AppleNativeInterleave`.
- [x] Commit.

Acceptance:

- `AppleNativeInterleave` is only a future candidate after a deeper
  platform-specific renderer split.
- v4.2 does not overclaim a fake interleave proof.

## Task 3: Input And Studio Probe

Files:

- Create: `examples/aichat/specs/aichat-liquid-glass-step-17-input-studio-probe.spec`
- Update route documentation from Task 1 and Task 2 evidence.

Steps:

- [x] Add a task spec for input and Studio implications.
- [x] Verify `WidgetTreeDump` still reports Makepad widgets for the probe.
- [x] Verify Studio framebuffer screenshot behavior.
- [x] Document that above-Metal native AppKit overlays require system
      screenshots for visual evidence.
- [x] Keep production input ownership with Makepad.
- [x] Commit.

Acceptance:

- Input remains owned by Makepad for production targets.
- Studio limitations are documented before aichat integration is considered.

## Task 4: Route Decision

Files:

- Create: `examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
- Update: `examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.spec.md`
- Update: `examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`

Steps:

- [x] Summarize prototype outcomes.
- [x] Choose one route: `NativeInterleave`, `ShaderBackdropInterior`, or
      `NativeUnderlayOnly`.
- [x] Define final user-facing backend names so logs do not overclaim.
- [x] List the first aichat integration slice, if any.
- [x] Commit.

Acceptance:

- The next implementation step is no longer ambiguous.
- The selected route is backed by visual evidence, input evidence, and Studio
  evidence.

## Verification Commands

Run after each task as applicable:

```bash
agent-spec parse <step-spec>
agent-spec lint <step-spec>
cargo check -p makepad-platform
cargo check -p makepad-example-aichat
git diff --check
```

Studio release validation must use RunItem through the Studio remote protocol.
Do not use raw `cargo run` for UI verification.
