# Agent2App Improvement Plan — July 2026 (post upstream merge)

Status: active. Owner: AlexZ. Implementer: Codex (via agent-spec contracts).
Source review: Claude session 2026-07-07, based on merge commit `7fece45d`.

## Context (what the review found)

1. The agent2app core channel (`SplashAction::Notify` + `register_agent_module`
   in `widgets/src/splash.rs`, host dispatch `handle_splash_event` in
   `examples/aichat-agent2app/src/main.rs`) survived the upstream merge intact and is
   **fork-exclusive** — upstream has no equivalent.
2. **Critical gap**: per-Splash isolated VMs (`cx.alloc_splash_vm()` →
   `makepad_widgets::script_mod`) never called `register_agent_module`, so
   `agent` was not injected into the VMs interactive Splash instances actually
   run in. An uncommitted one-liner in `widgets/src/lib.rs` (`script_mod` now
   calls `crate::splash::register_agent_module(vm)`) fixes it. aichat's
   explicit registration (`main.rs` ~3331) becomes redundant.
3. Upstream independently shipped the design we abandoned in 2026-06:
   per-instance isolated VMs + subtree-scoped `ui.<id>` (`ac10a82c`), Drop +
   `gc_dead_splash_isolates` lifecycle (`3a82d260`), and async requests that
   carry `req.vm_id` through `pump_widget_async` (fixes the cross-heap crash).
   This unlocks: multiple concurrent interactive generated apps, and generated
   apps that use `net.http_request`.
4. The uncommitted Animator-per-VM refactor (`derive_animator.rs`,
   `animator.rs`, `extern crate self as makepad_widgets`) breaks
   `cargo check -p makepad-widgets` (16× E0782). HEAD compiles clean.
5. `splash.md` (lines ~441-447) still claims `ui` is only visible inside
   `on_click`/`on_return`/`on_change` closures — stale after `ac10a82c`.
6. Two parallel agent2app channels exist: aichat's typed
   `agent.notify(event_id, json_payload)` vs `tools/canvas`'s untyped
   button-name broadcast over WS/HTTP. No shared envelope.
7. CFP v0.3 (`cfp v0.3.md`) is adopted **as vocabulary only** at this stage:
   L4 `PermissionLevel` for the capability manifest (P3), AppBundle-subset
   metadata for the AppGen workspace (P4). No CFP runtime, no seven layers.

## Phases

### P0 — Git hygiene + registration fix (human + Codex)

Manual git steps (AlexZ, before Codex starts):

1. Move the in-progress Animator-per-VM refactor off the working tree:
   `git stash` or commit `widgets/derive_widget/src/derive_animator.rs`,
   `widgets/src/animator.rs`, and the `extern crate self` /
   `CxSplashVmExt` lines of `widgets/src/lib.rs` to a branch
   `animator-per-vm-wip`. The working tree must compile before any spec work.
2. Keep on dev: the `register_agent_module(vm)` line in `lib.rs::script_mod`,
   the `resolves_fn` tick fix in `splash.rs`, and the fmt-only files
   (commit fmt separately or discard).

Codex contract: `agent2app-p0-isolated-vm-agent-registration.spec.md`

### P1 — Regression proof + prompt unlock (Codex + human acceptance)

Codex contract: `agent2app-p1-multi-instance-regression.spec.md`

Manual acceptance checklist (run after Codex finishes, release build:
`env -u ANTHROPIC_API_KEY cargo run --release -p makepad-example-aichat-agent2app`):

- [ ] Ask for two calculators in one conversation; both stay interactive,
      no `widget 'display' not found`, no CPU spin.
- [ ] Ask for a weather app using `net.http_request`; response callback does
      not crash (historical cross-heap crash, memory: 67cc66fa).
- [ ] `agent.notify` works from a button inside a generated app
      (was silently broken on isolated VMs before P0).
- [ ] Kill/regenerate an app several times; no VM leak symptoms
      (gc_dead_splash_isolates path).

### P2 — Unify tools/canvas onto the agent.notify envelope (Codex)

Codex contract: `agent2app-p2-canvas-notify-unification.spec.md`

### P3 — Capability manifest with CFP permission levels (Codex)

Codex contract: `agent2app-p3-capability-permission-levels.spec.md`

### P4 — AppBundle-subset metadata for AppGen workspace (spec deferred)

Persist per generated app: `user_intent`, `decisions[]`, `commitments[]`,
splash source, `parent_version` chain — the minimal CFP AppBundle subset that
enables incremental modification instead of regeneration. **Do not write this
spec until P2/P3 land**; the storage shape depends on the unified envelope and
the manifest format.

## Spec index & dependencies

| Spec | Depends | Estimate |
|---|---|---|
| agent2app-p0-isolated-vm-agent-registration | — | 0.5d |
| agent2app-p1-multi-instance-regression | p0 | 1d |
| agent2app-p2-canvas-notify-unification | p0 | 1d |
| agent2app-p3-capability-permission-levels | p0, p1 | 2d |

Graph: `agent-spec graph --spec-dir examples/aichat-agent2app/specs`

## Codex operating notes

- Verify each contract with `agent-spec verify <spec> --code .` before handing
  back; boundaries are mechanically enforced.
- Headless ScriptVm test precedent: `platform/script/std/tests/headless.rs`.
  Widget-tree test precedent: `#[cfg(test)]` mods in `widgets/src/widget_tree.rs`.
- Never move `agent.notify`/`SplashAction` below the widgets crate —
  `makepad-script` must not depend on `Cx::post_action` (crate boundary rule
  from AGENT-2-APP-DESIGIN.md, still binding).
- Run everything with `env -u ANTHROPIC_API_KEY` (stale API key causes 401s
  against subscription auth).
