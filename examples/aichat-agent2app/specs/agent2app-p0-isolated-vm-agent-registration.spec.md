spec: task
name: "Agent module registration on isolated Splash VMs"
tags: [makepad, agent2app, splash, vm]
estimate: 0.5d
---

## Intent

Every Splash instance lazily allocates its own isolated script VM
(`Splash::eval_body` → `cx.alloc_splash_vm()`), and that init path never
injected the `agent` module — so `agent.notify(...)` failed with
"variable agent not found" in exactly the VMs interactive generated UI runs
in. Make `register_agent_module` part of the widgets crate's `script_mod` so
main VM and every isolated VM both get the `agent` global, and remove the
now-redundant explicit registration in aichat.

## Decisions

- `register_agent_module(vm)` is called from `pub fn script_mod` in
  `widgets/src/lib.rs` (after `theme_mod` and `widgets_mod`), so
  `alloc_splash_vm` picks it up for every isolated VM.
- The explicit `register_agent_module(vm)` call in aichat's
  `AppMain::script_mod` (`examples/aichat-agent2app/src/main.rs` ~3331) is deleted;
  aichat relies on `makepad_widgets::script_mod`.
- `register_agent_module` stays in `widgets/src/splash.rs`; `makepad-script`
  (platform/script) must not gain any dependency on `Cx::post_action` or
  `SplashAction`.
- New tests live in a `#[cfg(test)]` module in `widgets/src/splash.rs`,
  building a headless `ScriptVm` the same way
  `platform/script/std/tests/headless.rs` does.

## Boundaries

### Allowed Changes
- widgets/src/lib.rs
- widgets/src/splash.rs
- examples/aichat-agent2app/src/main.rs

### Forbidden
- Do not modify platform/script/**
- Do not add new cargo dependencies
- Do not change the `SplashAction::Notify { event_id, payload }` shape

## Completion Criteria

Scenario: agent global is injected by widgets script_mod (critical)
  Test:
    Package: makepad-widgets
    Filter: test_script_mod_injects_agent_global
  Given a headless ScriptVm initialized via makepad_widgets::script_mod
  When the script expression `agent` is evaluated
  Then it resolves to a module object, not an undefined-variable error
  And the module exposes a callable `notify` method

Scenario: double registration is harmless
  Test:
    Package: makepad-widgets
    Filter: test_register_agent_module_twice_keeps_notify_callable
  Given a headless ScriptVm where register_agent_module has run once
  When register_agent_module runs a second time on the same VM
  Then evaluating `agent.notify` still resolves to a callable method
  And VM evaluation reports no error

Scenario: aichat no longer registers the agent module explicitly
  Test:
    Package: makepad-widgets
    Filter: test_aichat_has_no_explicit_agent_registration
  Given the file examples/aichat-agent2app/src/main.rs
  When it is scanned for the token "register_agent_module"
  Then the token does not appear

Scenario: notify with missing arguments does not poison the VM
  Test:
    Package: makepad-widgets
    Filter: test_agent_notify_missing_args_returns_nil
  Given a headless ScriptVm with the agent module registered
  When the script calls `agent.notify()` with no arguments
  Then the call evaluates to NIL and the VM reports no error
  And a subsequent expression evaluates normally on the same VM

## Out of Scope

- The Animator-per-VM refactor (parked on branch animator-per-vm-wip)
- Prompt/system-message changes (P1 spec)
- tools/canvas (P2 spec)
