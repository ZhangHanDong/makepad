spec: task
name: "AI Chat Liquid Glass Step 109 aichat Native Control Probe"
tags: [makepad, aichat, macos, liquid-glass, native-controls, probe]
---

## Intent

Add an explicit aichat opt-in probe for the macOS native control installer so
Studio/runtime validation can exercise native button descriptors without
changing default aichat UI behavior.

## Decisions

- Gate the probe behind `AICHAT_NATIVE_CONTROL_PROBE`.
- Accepted values are `buttons`, `1`, `true`, and `on`.
- The probe enables `native_control: true` on `clear_button` and `send_button`.
- Default aichat behavior remains unchanged when the environment variable is
  absent.

## Boundaries

- Do not enable native controls by default.
- Do not change chat send/clear action handling; existing `button.clicked`
  handlers remain the only app-level path.
- Do not claim native button glass styling is complete.
- Do not add UIKit probe wiring in this step.

## Out of Scope

- Full visual tuning for native buttons.
- Accessibility validation.
- Native icon transport for the send arrow.
- Studio run evidence; this step only makes the probe runnable.

## Acceptance Criteria

### Scenario: probe env parser is deterministic

Test: `cargo test -p makepad-example-aichat aichat_native_control_probe_env_accepts_button_values -- --nocapture`

Given `AICHAT_NATIVE_CONTROL_PROBE`
When parsing accepted and rejected values
Then only `buttons`, `1`, `true`, and `on` enable the probe.

### Scenario: startup applies native control probe only when enabled

Test: `rg "AICHAT_NATIVE_CONTROL_PROBE|apply_native_control_probe|native-control-probe=buttons-enabled" examples/aichat/src/main.rs`

Given the probe environment variable is enabled
When aichat starts
Then `clear_button` and `send_button` are marked `native_control: true`.

### Scenario: default aichat remains gated

Test: `rg "if native_control_probe_enabled\\(\\)|native_control: true" examples/aichat/src/main.rs`

Given normal startup without the probe environment variable
When reviewing aichat startup
Then native control enabling remains inside the probe gate.

### Scenario: audit remains honest

Test: `rg "Step 109 adds an opt-in aichat probe|native interactive controls are not complete" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

Given the probe is opt-in and unvalidated
When reviewing the completion audit
Then native interactive controls are still not marked complete.
