spec: task
name: "AI Chat Liquid Glass Step 110 Studio Native Control Probe Runnable"
tags: [makepad, studio, aichat, macos, liquid-glass, native-controls]
---

## Intent

Expose the aichat native-control probe as a Studio runnable item so agents and
humans can validate the macOS `NSButton` installer through the required Studio
remote `RunItem` workflow.

## Decisions

- Extend `RunAichatMacosGlass` with `native_control_probe`.
- Pass `AICHAT_NATIVE_CONTROL_PROBE` through `hub.run`.
- Add `makepad-example-aichat-macos-native-clear-control-probe`.
- The runnable uses `backend: "macos-native-clear"` and
  `native_control_probe: "buttons"`.

## Boundaries

- Do not change the default `makepad-example-aichat-macos-native-clear` runnable.
- Do not use bridge `Cargo` requests for validation.
- Do not make native controls default in aichat.
- Do not claim visual validation until the runnable is actually launched and
  inspected.

## Out of Scope

- Studio screenshot interpretation.
- Manual click verdict.
- UIKit probe runnable.
- Native button styling changes.

## Acceptance Criteria

### Scenario: runnable passes native control env

Test: `rg "AICHAT_NATIVE_CONTROL_PROBE|native_control_probe" makepad.splash`

Given the Studio run item script
When running an aichat macOS glass item
Then the native control probe value is passed into the child process env.

### Scenario: control probe run item exists

Test: `rg "makepad-example-aichat-macos-native-clear-control-probe" makepad.splash`

Given Studio runnable discovery
When reading `makepad.splash`
Then there is a dedicated native clear control probe item.

### Scenario: default native clear runnable remains unchanged

Test: `rg "makepad-example-aichat-macos-native-clear\", backend: \"macos-native-clear\"" makepad.splash`

Given the normal native clear smoke path
When adding the control probe runnable
Then the default native clear item remains present without probe-specific
properties.

### Scenario: audit tracks remaining manual click verdict

Test: `rg "control-probe|real AppKit click/target-action manual verdict" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

Given the runnable exists
When reviewing the completion audit
Then it records the Studio runtime result while keeping the real AppKit
click/target-action verdict incomplete.
