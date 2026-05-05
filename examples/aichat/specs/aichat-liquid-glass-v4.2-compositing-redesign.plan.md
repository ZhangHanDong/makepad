# aichat Liquid Glass v4.2 Compositing Redesign Plan

## Goal

Prove whether Makepad can deliver recognizable interior Liquid Glass with Apple
native APIs, or whether complete interiors must move to `ShaderBackdrop`.

## Scope

This plan is for evidence gathering and architecture selection. It should not
rewrite aichat production UI until a prototype proves the compositing route.

## Task 1: Above-Metal Sampling Probe

Files:

- Create: `examples/aichat/specs/aichat-liquid-glass-step-14-above-metal-probe.spec`
- Create or modify a small macOS-only proof target/module under the existing
  Makepad examples or platform test area.

Steps:

- [ ] Add a task spec for an above-Metal sampling probe.
- [ ] Build a release Studio runnable that draws a moving high-contrast Metal
      pattern.
- [ ] Add one `NSGlassEffectView` above that Metal layer.
- [ ] Verify through Studio run logs that the glass view is installed.
- [ ] Manually inspect whether the glass surface visibly samples the Metal
      pattern.
- [ ] Record result and screenshot limitation notes.
- [ ] Commit.

Acceptance:

- The probe can distinguish "native glass samples Metal content" from "native
  glass is only a flat overlay/edge effect."

## Task 2: Two-Layer Interleave Probe

Files:

- Create: `examples/aichat/specs/aichat-liquid-glass-step-15-two-layer-interleave-probe.spec`
- Modify the same proof target/module created in Task 1.

Steps:

- [ ] Add a lower Metal layer/pass that draws the patterned background.
- [ ] Add native glass between lower and upper content.
- [ ] Add an upper transparent Metal layer/pass for text and controls.
- [ ] Verify resize and DPI alignment.
- [ ] Run a Studio release target.
- [ ] Manually inspect whether native glass visibly treats the lower Metal
      layer while upper text remains readable.
- [ ] Commit.

Acceptance:

- If successful, this is the first candidate for `AppleNativeInterleave`.
- If unsuccessful, full native Liquid Glass is unlikely without a deeper
  platform-specific renderer split.

## Task 3: Input And Studio Probe

Files:

- Create: `examples/aichat/specs/aichat-liquid-glass-step-16-input-studio-probe.spec`
- Modify the proof target/module from Tasks 1-2.

Steps:

- [ ] Add button, text input, scroll, and drag regions to the winning visual
      hierarchy.
- [ ] Verify clicks and typing through Studio remote.
- [ ] Verify widget dump still reports Makepad widgets.
- [ ] Verify screenshot behavior and document whether native layers appear.
- [ ] Commit.

Acceptance:

- Input remains owned by Makepad or the forwarding policy is explicit and
  testable.
- Studio limitations are documented before aichat integration is considered.

## Task 4: Route Decision

Files:

- Create: `examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
- Update: `examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.spec.md`
- Update: `examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`

Steps:

- [ ] Summarize prototype outcomes.
- [ ] Choose one route: `NativeInterleave`, `ShaderBackdropInterior`, or
      `NativeUnderlayOnly`.
- [ ] Define final user-facing backend names so logs do not overclaim.
- [ ] List the first aichat integration slice, if any.
- [ ] Commit.

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
