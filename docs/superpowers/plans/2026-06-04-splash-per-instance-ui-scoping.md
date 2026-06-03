# Splash Per-Instance `ui` Scoping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make AI-generated interactive Splash apps (`runsplash` blocks that use `ui.<id>.set_text(...)`, `on_click`, local state) work correctly **even when several of them coexist** in the same chat — today the second one breaks both.

**Architecture:** Each `Splash` widget already owns an isolated script VM (`SplashVmId`). Evaluate each Splash's body in its **own** isolated VM and inject a `ui` global **scoped to that Splash's own subtree** (a `ui` handle rooted at `self.uid`). Then `ui.<id>` resolution (`find_flood`) resolves within the owning Splash's subtree first and never collides with another Splash that reused the same widget id (e.g. `display`).

**Tech Stack:** Rust, makepad-widgets (`widgets/src/splash.rs`, `widgets/src/widget_async.rs`), makepad script VM (`platform/script`).

---

## Background / Root Cause (read before implementing)

A `runsplash` calculator generates Splash like:

```
display := Label{ text: "0" }
... Button{ text: "5" on_click: || ui.display.set_text("5") }
```

When the button is clicked, the closure runs `ui.display.set_text(...)`. Resolution of `ui.display` happens in `widgets/src/widget_async.rs` (the `ui` handle getter, ~line 776):

```rust
let child_ref = cx.widget_tree().find_flood(target_uid, &[prop]);   // search target_uid's subtree first
if !child_ref.is_empty() { return child_ref; }
let mut matches = cx.widget_tree().find_all_anywhere_including_skipped(&[prop]); // global fallback
if matches.len() == 1 { return matches.pop().unwrap(); }            // must be UNIQUE
WidgetRef::empty()                                                  // 0 or >1 -> "widget not found"
```

`find_flood(origin_uid, ...)` (`widgets/src/widget_tree.rs:1710`) searches `origin_uid`'s **own subtree first and returns immediately on a local hit**, only expanding to ancestors when nothing is found locally (verified by reading).

Two independent facts cause the bug:

1. The `ui` global is built by `update_global_ui_handle(cx, root_uid)` with the **global app root** as `target_uid`. So `ui.display` floods from the app root.
2. The chat message list is a `PortalList` whose rows are `skip_search` nodes. `find_flood`'s subtree-first phase (`find_within_graph`, passes `skip_search = true`) therefore does **not** descend into the message rows from the app root, returns empty, and falls through to the global fallback `find_all_anywhere_including_skipped` (passes `skip_search = false`). From the app root the fallback reaches **both** calculators → `matches.len() == 2` → returns empty → `widget 'display' not found in tree`. With a single calculator `matches.len() == 1`, so it works — which is exactly the observed "first one works, second one breaks both".

**Why rooting `ui` at `self.uid` fixes it:** `find_flood` treats its `origin_uid` as a root that is **exempt** from both the self-match guard and the `skip_search` skip, so it always descends into the origin's own children. With `ui` rooted at the Splash's own `self.uid`, `ui.display` finds that Splash's `display` locally and returns **before** ever reaching the ambiguous global fallback.

**Why isolated VMs alone don't fix it (already tried):** `ui` is only ever `set_injected_global`'d into the MAIN vm. Splash bodies evaluated in an isolated vm have no `ui` at all → `variable ui not found in scope`. The fix must inject `ui` into the isolated vm **and** root that `ui` handle at the Splash's own `self.uid` so resolution is locally scoped.

**This bug exists identically in upstream and in this fork** — `widget_async.rs` is byte-identical between them, and the `splash.md` system prompt (which hands the model generic ids like `display`) is shared. So this is a net-new fix, not a port.

**Key APIs (already exist):**
- `ScriptVm::set_injected_global(key: LiveId, value: ScriptValue)` — `pub`, `platform/script/src/vm.rs:1034`.
- `build_ui_handle_for_uid(target_uid) -> ScriptValue` — builds a `ui` handle with `gc.uid = target_uid`. Currently on the **private** trait `WidgetToScriptCallExt` (`widgets/src/widget_async.rs:357`), impl for `ScriptVm`. Must be reachable from `splash.rs` (Task 1).
- `CxSplashVmExt::{alloc_splash_vm, with_script_vm_id, register_widget_vm_id}` — `pub`, `widgets/src/widget_async.rs:153`.

**Verification reality:** This is a GUI + VM integration path; there are no inline unit tests for `splash.rs`/`widget_async.rs`. Primary verification is running the `aichat` example and creating **two** interactive apps. A `/tmp/makepad-upstream-aichat` worktree already exists and is wired to build quickly — use it as the fast validation harness before/while changing `dev`.

---

## File Structure

- `widgets/src/widget_async.rs` — add one small `pub fn` that exposes scoped-`ui` injection (wraps the private `build_ui_handle_for_uid` + `set_injected_global`). Single responsibility: "inject a subtree-scoped `ui` global into the current vm".
- `widgets/src/splash.rs` — change `eval_body`, `stream_append`, and `call_fn` to evaluate/inject/run in the Splash's **own** isolated vm with a scoped `ui`. This is the only behavioural change.
- `examples/aichat` — no code change; used for verification.

---

## Task 1: Expose subtree-scoped `ui` injection from `widget_async`

**Files:**
- Modify: `widgets/src/widget_async.rs` (add a `pub fn` near `update_global_ui_handle`, ~line 343)

- [ ] **Step 1: Add the helper**

Add this free function in `widgets/src/widget_async.rs` (it lives in the same module as the private `WidgetToScriptCallExt`, so it can call `build_ui_handle_for_uid`). Place it directly after `update_global_ui_handle`:

```rust
/// Inject a `ui` global into the *current* script vm, scoped so that
/// `ui.<id>` resolves within `root_uid`'s subtree first (see `find_flood`).
///
/// Used by `Splash` so each isolated Splash vm gets its own `ui` rooted at
/// that Splash's widget, preventing id collisions (e.g. two `display`s) across
/// multiple concurrently-rendered Splash apps.
pub fn inject_scoped_ui_global(vm: &mut ScriptVm, root_uid: WidgetUid) {
    let ui_handle = vm.build_ui_handle_for_uid(root_uid);
    vm.set_injected_global(id!(ui), ui_handle);
}
```

- [ ] **Step 2: Build the crate to confirm it compiles and the private trait is in scope**

Run: `cargo build -p makepad-widgets`
Expected: PASS (no privacy/visibility error on `build_ui_handle_for_uid`).
If it errors with "method not found", add `use` of the trait at call site or confirm the fn is inside the module that declares `WidgetToScriptCallExt`.

- [ ] **Step 3: Commit**

```bash
git add widgets/src/widget_async.rs
git commit -m "feat(widgets): add inject_scoped_ui_global for per-Splash ui scoping"
```

---

## Task 2: Evaluate `eval_body` in the isolated vm with a scoped `ui`

**Files:**
- Modify: `widgets/src/splash.rs` (`fn eval_body`)

Current `dev` `eval_body` evaluates in the MAIN vm (`cx.with_vm`) with no registration — that is why a single app works but multiple collide (shared global `ui`/tree). Change it to evaluate in the Splash's own vm, inject a scoped `ui`, and register the resulting widgets under that vm.

- [ ] **Step 1: Ensure the isolated vm is allocated**

Keep / restore the allocation at the top of `eval_body`:

```rust
if self.vm_id == MAIN_SPLASH_VM_ID {
    self.vm_id = cx.alloc_splash_vm();
}
```

- [ ] **Step 2: Replace the eval + commit block**

Replace the `cx.with_vm(...)` eval block and the commit with:

```rust
let vm_id = self.vm_id;
let self_uid = self.uid;
let new_view = cx.with_script_vm_id(vm_id, |vm| {
    // Scope `ui.<id>` to THIS Splash's own subtree so ids like `display`
    // don't collide with other Splash apps in the same chat.
    crate::widget_async::inject_scoped_ui_global(vm, self_uid);
    let value = vm.with_instruction_limit(SPLASH_EVAL_INSTRUCTION_LIMIT, |vm| {
        vm.eval_with_append_source(script_mod, &code, NIL.into())
    });
    if !value.is_err() && !value.is_nil() {
        Some(View::script_from_value(vm, value))
    } else {
        None
    }
});

if let Some(view) = new_view {
    self.unregister_view_owners(cx);
    self.view = view;
    self.view.set_visible(cx, true);
    self.register_view_owners(cx);          // registers widgets under self.vm_id
    cx.widget_tree_mark_dirty(self.uid);    // so find_flood(self.uid, ...) sees them
}
```

- [ ] **Step 3: Build**

Run: `cargo build -p makepad-widgets`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add widgets/src/splash.rs
git commit -m "fix(widgets/splash): eval_body in isolated vm with subtree-scoped ui"
```

---

## Task 3: Inject the scoped `ui` on the streaming path too

**Files:**
- Modify: `widgets/src/splash.rs` (`fn stream_append`)

`stream_append` already evaluates in `cx.with_script_vm_id(self.vm_id, ...)` and registers owners + marks dirty, but does **not** inject `ui` — so streamed interactive apps hit `variable ui not found in scope`. Add the same injection.

- [ ] **Step 1: Inject inside the existing `with_script_vm_id` block**

Immediately inside the `cx.with_script_vm_id(vm_id, |vm| { ... })` closure in `stream_append`, before the `with_instruction_limit` eval, add:

```rust
crate::widget_async::inject_scoped_ui_global(vm, self_uid);
```

(Hoist `let self_uid = self.uid;` above the closure, mirroring Task 2.)

- [ ] **Step 2: Build**

Run: `cargo build -p makepad-widgets`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add widgets/src/splash.rs
git commit -m "fix(widgets/splash): inject scoped ui on stream_append path"
```

---

## Task 4: Keep `call_fn` (tick) in the same isolated vm

**Files:**
- Modify: `widgets/src/splash.rs` (`fn call_fn`)

`call_fn` looks up the body by `unique_id`. Now that the body lives in `self.vm_id` (not the MAIN vm), the lookup must use that vm or `fn tick()` silently never fires.

- [ ] **Step 1: Switch the vm**

Change `cx.with_vm(|vm| {` in `call_fn` to:

```rust
cx.with_script_vm_id(self.vm_id, |vm| {
```

- [ ] **Step 2: Build**

Run: `cargo build -p makepad-widgets`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add widgets/src/splash.rs
git commit -m "fix(widgets/splash): call_fn looks up body in the isolated vm"
```

---

## Task 5: Verify — two interactive apps coexist (the real gate)

**Files:** none (manual/integration verification)

This is the acceptance test. Automated unit testing of the VM+widget-tree+event path is impractical; verify by running the example.

- [ ] **Step 1: Build the aichat example**

Run: `cargo build -p makepad-example-aichat`
Expected: PASS.

- [ ] **Step 2: Run with subscription auth (strip the stale API key)**

Run: `env -u ANTHROPIC_API_KEY cargo run -p makepad-example-aichat`
(See session notes: a stale `ANTHROPIC_API_KEY` in the environment forces headless `claude -p` onto a dead key → 401. Stripping it falls back to subscription OAuth.)

- [ ] **Step 3: Acceptance scenario**

In the running app, select **Claude Splash**, then:
1. Ask for a "calculator app". Wait for the `runsplash` block to render. Click digits/operators — the display must update and compute.
2. In the **same chat**, ask for a "scientific calculator". Wait for it to render.
3. Click buttons on **both** calculators.

Expected: **both** remain interactive and update independently.
Expected log (`stderr`): no `variable ui not found in scope` and no `widget 'display' not found in tree`.

- [ ] **Step 4: Negative check (regression guard)**

Confirm a single non-interactive `runsplash` (e.g. "draw a card layout") still renders, and that streaming a long app still renders incrementally.

- [ ] **Step 5: Tag the verified state**

```bash
git commit --allow-empty -m "test(splash): verified two concurrent interactive runsplash apps"
```

---

## Risks & Notes

- **Primary risk (now low):** that `find_flood(self.uid, ["display"])` still collides. Mitigated by reading `find_flood` — it resolves the origin subtree first and returns before expanding to ancestors. If a verification shows collision anyway, inspect whether `self.uid` is actually present as a node in `cx.widget_tree()` (mark_dirty's `refresh_node_children` is a no-op if the Splash node isn't in the graph) and whether the calculator widgets are nested under it.
- **`fn tick()` interaction:** Task 4 keeps tick working in the isolated vm. If a Splash uses both `tick` and `ui.<id>`, both now share the isolated vm — intended.
- **Fast pre-validation:** the `/tmp/makepad-upstream-aichat` worktree can host Tasks 1–4 first (identical files) to prove the mechanism in ~20s build loops before committing to `dev`. The worktree currently has a MAIN-vm prototype; reset its `widgets/src/splash.rs` to `dev`'s before applying these tasks to avoid confusion.
- **Out of scope:** changing the `splash.md` system prompt or id-namespacing the generated code. This fix keeps generated apps untouched and solves the collision structurally in the host.
