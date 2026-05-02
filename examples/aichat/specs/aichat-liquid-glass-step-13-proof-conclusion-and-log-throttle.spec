spec: task
name: "AI Chat Liquid Glass Step 13 Proof Conclusion And Log Throttle"
tags: [makepad, aichat, liquid-glass, macos-native, compositing, diagnostics]
---

## Intent

Document the Phase 12 visual proof result and reduce diagnostic log noise. The
striped proof substrate is visible and resizes correctly, but the user cannot
visually identify a meaningful interior glass treatment; v4.1 must therefore be
documented as a native-substrate proof, not complete Liquid Glass.

## Decisions

- Treat the striped proof result as evidence that the native underlay is visible.
- Treat the lack of recognizable interior glass as evidence against the current
  v4.1 under-Metal compositing model for full Liquid Glass.
- Keep the proof runnable for future manual validation.
- Log `compositing-proof=transparent-overlay` only once per process.
- Treat subpixel native batch jitter as equivalent so the macOS backend does not
  reinstall and relog the same native glass hierarchy unnecessarily.

## Constraints

- Must not remove or weaken the existing native proof runnable.
- Must not claim v4.1 delivers full Liquid Glass.
- Must not hide real resize changes; only subpixel jitter may be coalesced.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-13-proof-conclusion-and-log-throttle.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`
- `examples/aichat/specs/aichat-liquid-glass-v4.1-manual-visual-checklist.md`
- `examples/aichat/src/main.rs`
- `platform/src/os/apple/macos/macos_window.rs`

### Forbidden

- `makepad.splash`
- `widgets/**`
- ShaderBackdrop implementation

### Out of Scope

- The v4.2 compositing redesign.
- Native-hosted Makepad layers.
- Offscreen blur/refraction.

## Acceptance Criteria

Scenario: proof conclusion is documented
Test: `rg "striped proof|not complete Liquid Glass|under-Metal" examples/aichat/specs/aichat-liquid-glass-v4.1-*.md`
Given the striped proof target has been reviewed visually
When the v4.1 notes are read
Then they state that v4.1 proves native underlay visibility but not complete Liquid Glass
And the file output is limited to the v4.1 release notes and manual visual checklist

Scenario: compositing proof log is emitted once
Test: `rg "AICHAT_NATIVE_COMPOSITING_PROOF_LOGGED|compositing-proof=transparent-overlay" examples/aichat/src/main.rs`
Given proof appearance is reapplied repeatedly
When aichat logs the proof mode
Then it uses a process-level guard to avoid repeated identical proof logs

Scenario: native batch cache tolerates subpixel jitter
Test: `cargo test -p makepad-platform native_glass_batch_equivalent -- --nocapture`
Given two native batches differ only by subpixel rect jitter
When the macOS backend compares them
Then they are treated as equivalent for cache reuse

Scenario: affected crates still compile
Test: `cargo check -p makepad-platform && cargo check -p makepad-example-aichat`
Given the conclusion and log throttle changes are applied
When the affected crates are checked
Then they compile without errors
