spec: task
name: "AI Chat Liquid Glass Step 114 AI Chat Probe Click Diagnostics"
tags: [makepad, aichat, macos, liquid-glass, native-controls, diagnostics]
---

## Intent

Log aichat probe button clicks while `AICHAT_NATIVE_CONTROL_PROBE` is enabled so
runtime validation can distinguish Makepad-delivered clicks from AppKit native
target/action clicks.

## Decisions

- `send_button` logs `native-control-probe=makepad-click id=send_button` only
  while the native control probe is enabled.
- `clear_button` logs `native-control-probe=makepad-click id=clear_button` only
  while the native control probe is enabled.
- The logs do not change button behavior.
- These logs are diagnostics and do not prove native AppKit hit testing.

## Boundaries

- Do not log normal aichat button clicks outside the probe.
- Do not change the native control descriptor exporter.
- Do not change `send_message` or `clear_chat` semantics.
- Do not mark native controls complete without AppKit `target-action` logs.

## Out of Scope

- AppKit view hierarchy changes.
- UIKit controls.
- Accessibility ownership.
- Native button styling.

## Acceptance Criteria

### Scenario: send probe click is logged

Test: `rg "native-control-probe=makepad-click id=send_button" examples/aichat/src/main.rs`

Given the native control probe is enabled
When the Makepad `send_button` click path runs
Then a probe-only send click diagnostic is logged.

### Scenario: clear probe click is logged

Test: `rg "native-control-probe=makepad-click id=clear_button" examples/aichat/src/main.rs`

Given the native control probe is enabled
When the Makepad `clear_button` click path runs
Then a probe-only clear click diagnostic is logged.

### Scenario: audit separates Makepad and AppKit click evidence

Test: `rg "makepad-click|target-action" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

Given both diagnostic classes exist
When evaluating the native control probe
Then Makepad click logs are treated as routing evidence, not native AppKit
target/action completion evidence.
