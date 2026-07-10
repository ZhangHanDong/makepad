spec: task
name: "Multi-instance regression proof and prompt unlock"
tags: [makepad, agent2app, splash, aichat, prompt]
depends: [agent2app-p0-isolated-vm-agent-registration]
estimate: 1d
---

## Intent

Upstream commits `ac10a82c`/`3a82d260` shipped per-Splash isolated VMs with
subtree-scoped `ui.<id>` resolution and vm_id-tracked async resumption —
the fixes that previously blocked running two interactive generated apps at
once. Prove the merged tree actually delivers this with regression tests, then
update the aichat system prompt and `splash.md` to stop imposing the
now-obsolete restrictions on generated code.

## Decisions

- Scoped-ui regression tests live in `widgets/` tests, following the headless
  patterns in `widgets/src/widget_tree.rs` and
  `platform/script/std/tests/headless.rs`.
- A manual smoke test `test_multi_instance_manual_smoke` is added under
  `#[ignore]` and run via `cargo test -- --ignored`; the interactive checklist
  itself lives in AGENT2APP-IMPROVEMENT-PLAN-2026-07.md (P1 section).
- The aichat splash system prompt (`examples/aichat-agent2app/src/main.rs`, the
  `ClaudeSplash | GeminiSplash` branch, ~2383-2418) is updated to state that
  multiple interactive `runsplash` apps may coexist in one conversation and
  that `ui.<id>` resolves within the emitting app only.
- `splash.md` drops the stale claim (~lines 441-447) that `ui` is only
  visible inside `on_click`/`on_return`/`on_change` closures; helper `fn`s
  may use `ui` since upstream `ac10a82c`.

## Boundaries

### Allowed Changes
- widgets/src/widget_async.rs
- widgets/src/splash.rs
- widgets/tests/**
- examples/aichat-agent2app/src/main.rs
- splash.md

### Forbidden
- Do not change the generated-code contract (no new required syntax in
  runsplash blocks)
- Do not re-introduce any register_widget_vm_id-style manual VM registry
- Do not modify platform/script/**

## Completion Criteria

Scenario: two isolated instances resolve the same widget id independently (critical)
  Test:
    Package: makepad-widgets
    Filter: test_scoped_ui_resolves_per_splash_instance
  Given two Splash bodies that each define a widget with id `display`
  And each body is evaluated in its own isolated VM with a scoped ui global
  When each VM resolves `ui.display`
  Then each VM gets the widget belonging to its own subtree
  And neither resolution falls back to an ambiguous global lookup

Scenario: single instance keeps working after the second one appears
  Test:
    Package: makepad-widgets
    Filter: test_first_instance_survives_second_instance
  Given one Splash instance whose `ui.display` already resolves
  When a second Splash instance defining `display` is evaluated
  Then `ui.display` in the first VM still resolves to the first instance's widget

Scenario: dead isolated VMs are reclaimed
  Test:
    Package: makepad-widgets
    Filter: test_dead_splash_isolate_is_gc_reclaimed
  Given an isolated VM allocated for a Splash instance
  When the instance is dropped and a new isolated VM is allocated
  Then the dead VM id is absent from the live VM and thread maps

Scenario: lookup of a widget absent from the subtree fails cleanly
  Test:
    Package: makepad-widgets
    Filter: test_ui_lookup_missing_widget_errors
  Given a Splash body evaluated in an isolated VM with a scoped ui global
  When the VM resolves `ui.nonexistent` for an id defined in no subtree
  Then the lookup returns a widget-not-found error
  And it does not silently match a widget from another instance

Scenario: prompt declares multi-app support
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_prompt_declares_multi_app_support
  Given the aichat splash system prompt string
  When it is inspected
  Then it contains an explicit statement that multiple interactive runsplash
    apps can coexist in one conversation

Scenario: stale ui-scope restriction is gone from prompt and manual
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_no_stale_ui_closure_restriction
  Given the aichat splash system prompt string and splash.md
  When both are scanned for the claim that ui is only visible inside
    on_click/on_return/on_change closures
  Then the claim appears in neither text

## Out of Scope

- Interactive end-to-end acceptance (human checklist in the plan doc, P1)
- glass.* widget adoption in the prompt (optional follow-up, not gated here)
- Capability manifest changes (P3 spec)

<!-- lint-ack: output-mode-coverage — this task edits prompt text and docs; there are no CLI output modes or file-output behaviors to cover -->

## Questions

- [x] RESOLVED: `net.http_request` resumption is NOT given a headless test in
  this task — the vm_id plumbing in `pump_widget_async` requires a live Cx.
  Record this in the test module comment; the manual weather-app check in
  AGENT2APP-IMPROVEMENT-PLAN-2026-07.md (P1 checklist) is the acceptance path.
