# Agent-to-App Design for AI Chat

## Status

This document defines the `agent2app` direction for `makepad-example-aichat`.

`agent2app` has two different product meanings. They must stay separate:

- **Runtime A2A**: the agent renders and drives live UI inside the chat host.
- **Generative A2A**: the agent generates an independent Makepad app/project.

The current shipped implementation is **Runtime A2A D1**. It is not a
v0/bolt-style project generator yet.

## Two Tracks

### Runtime A2A: `agent2view`

Runtime A2A treats the generated app as an inline live view owned by the chat
host. The agent generates `runsplash`, the host owns state and capabilities, and
generated UI can call back into the host through a narrow action channel.

Current D1 loop:

```text
AI-generated runsplash UI
    -> Button on_click
    -> agent.notify(event_id, payload)
    -> aichat host action handler
    -> local state update or LLM prompt
    -> generated UI re-renders from state
```

Runtime A2A is the right home for:

- `runsplash` inline rendering.
- `agent.notify(event_id, payload)`.
- host-owned state such as `APP_DEMO_STATE`.
- state placeholders such as `{{state.count}}`.
- host action manifests and parameter validation.
- live re-rendering inside chat messages.

### Generative A2A: `agent2project`

Generative A2A treats the generated app as a real artifact outside the chat
view. The agent should produce files, resources, a Makepad crate, state modules,
action handlers, and packaging/run metadata.

Target future loop:

```text
user request
    -> appplan artifact contract
    -> generated file tree/resources
    -> Makepad crate
    -> Studio runnable item or export/package command
```

Generative A2A is the right home for:

- file tree generation.
- `Cargo.toml` and `src/main.rs` generation.
- resource generation and copying.
- generated app state modules.
- generated event/action handler code.
- Studio run/export/package integration.

Generative A2A must not reuse the Runtime A2A manifest as-is. It needs its own
artifact contract because its output is a codebase, not an inline view.

## Current Scope: Runtime A2A D1

The goal of D1 is not to implement the full Robrix-style
schema/template/reconciler architecture. The goal is to add the smallest
reliable live wire between AI-generated Splash UI and the aichat host.

## Core Decision

D1 uses two mechanisms:

- `agent.notify(event_id, payload)` as the reverse channel from Splash to host.
- Host-side state template replacement, starting with `{{state.count}}`, as the
  reliable D1 state display path.

VM-level `state.count` injection is useful, but it is not required for the first
demo and should not block D1. It can be added after the reverse channel and
host-rendered state loop are proven.

## Non-Goals

These non-goals apply to Runtime A2A D1 only:

- No Robrix schema.
- No widget whitelist.
- No generic capability registry.
- No reconciler.
- No path-scoped partial state updates.
- No multi-agent resolver/template-author split.
- No repair loop for invalid Splash.
- No product-grade state security model.
- No independent Makepad project generation.
- No file tree artifact creation.
- No export/package workflow for generated apps.

Generative A2A should address the last three items in a separate design, not by
expanding the D1 Runtime A2A protocol.

## Naming Boundary

Use these names in future specs and code comments:

- `agent2view` for Runtime A2A inside chat.
- `agent2project` for Generative A2A that creates an independent codebase.
- `agent2app` only as the umbrella term that includes both tracks.

The current `AppGen` tab is still Runtime A2A unless and until it writes a real
project artifact to disk. A generated `appplan json` block followed by a
`runsplash` block is not enough to call the result Generative A2A.

## Manifest Boundary

Runtime A2A manifests describe host capabilities:

- readable state paths, for example `{{state.count}}`.
- allowed host actions, for example `agent.notify("inc", {})`.
- action payload schemas.
- view/runtime lifecycle rules.

Generative A2A manifests describe project artifacts:

- generated files and directories.
- generated crate/package metadata.
- dependencies.
- resources.
- state modules.
- event/action handler code.
- run/export/package targets.

Do not mix the two manifest types. A Runtime A2A action manifest is a
capability boundary for a live view. A Generative A2A artifact manifest is a
build contract for a generated project.

## D1 User Story

The user asks:

```text
Draw a counter. Use the current state for the count.
```

The LLM returns:

````markdown
```runsplash
RoundedView{
    width: Fill
    height: Fit
    flow: Down
    spacing: 12
    padding: 16

    Label{ text: "Count: {{state.count}}" }

    View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8

        Button{ text: "+1" on_click: || agent.notify("inc", {}) }
        Button{ text: "-1" on_click: || agent.notify("dec", {}) }
        Button{ text: "Reset" on_click: || agent.notify("reset", {}) }
        Button{ text: "Ask AI" on_click: || agent.notify("ask_ai", {}) }
    }
}
```
````

Expected behavior:

- `+1` increments local `state.count` and immediately re-renders the displayed
  counter.
- `-1` decrements local `state.count` and immediately re-renders the displayed
  counter.
- `Reset` sets `state.count` to `0`.
- `Ask AI` sends the current state and click event back to the active LLM.
- Restarting the app restores the count from disk.

## Crate Boundary Rules

These rules are mandatory because they match the actual fork structure.

### `agent.notify` lives in `widgets/src/splash.rs`

Do not put this in `platform/script/src`.

Reason:

- `makepad-script` is a lower-level crate.
- It must not depend on `Cx::post_action`.
- It must not know about a Splash/aichat action type.

`Splash` is the correct boundary layer because it already owns the interaction
between the script VM and rendered UI.

### `SplashAction` lives in `widgets/src/splash.rs`

The action type must be visible to both:

- `Splash`, which posts it.
- `examples/aichat`, which handles it.

The action is a global/bare action, not a widget action:

```rust
#[derive(Clone, Debug, Default)]
pub enum SplashAction {
    Notify { event_id: String, payload: String },
    #[default]
    None,
}
```

No `DefaultNone`. No `MakepadAction` derive. `ActionTrait` is implemented by a
blanket impl for debug-able static types.

## Reverse Channel: `agent.notify`

Target file:

- `widgets/src/splash.rs`

Splash scripts should be able to call a top-level `agent` value:

```splash
agent.notify("inc", {})
```

Implementation shape:

- Create an `agent` script object/module in `widgets/src/splash.rs`.
- Add a native method `notify`.
- Register it with the actual VM API:
  - `vm.add_method(...)`
  - `script_args_def!(event = NIL, payload = NIL)`
  - `script_value!(vm, args.event)`
  - `script_value!(vm, args.payload)`
- Expose it with `vm.set_injected_global(id!(agent), agent_obj.into())`.
- Register `agent` once per VM lifecycle, not once per Splash eval. A clean shape
  is a `pub fn register_agent_module(vm: &mut ScriptVm)` helper in
  `widgets/src/splash.rs`, called during widget/app script initialization, before
  generated Splash code is evaluated.

Payload serialization:

- `event_id` may use `heap.cast_to_string`.
- `payload` must use the VM JSON path.
- Use `vm.bx.heap.to_json_inner(payload_value, &mut payload_string)` for payload.
- Do not use `cast_to_string` for payload objects, because `{}` becomes
  `"[ScriptObject]"`.

Expected native-method shape:

```rust
vm.add_method(
    agent,
    id_lut!(notify),
    script_args_def!(event = NIL, payload = NIL),
    |vm, args| {
        let event_value = script_value!(vm, args.event);
        let payload_value = script_value!(vm, args.payload);

        let mut event_id = String::new();
        vm.bx.heap.cast_to_string(event_value, &mut event_id);

        let mut payload = String::new();
        vm.bx.heap.to_json_inner(payload_value, &mut payload);

        Cx::post_action(SplashAction::Notify { event_id, payload });
        NIL
    },
);
```

The native method posts:

```rust
Cx::post_action(SplashAction::Notify { event_id, payload })
```

This is intentionally a bare action.

## Host Action Handling

Target file:

- `examples/aichat/src/main.rs`

Because `Cx::post_action` produces a bare action, aichat must not use
`as_widget_action()` for this event.

Correct handling shape:

```rust
for action in actions {
    if let SplashAction::Notify { event_id, payload } = action.cast() {
        self.handle_splash_event(cx, &event_id, &payload);
    }
}
```

Wrong handling shape:

```rust
if let Some(widget_action) = action.as_widget_action() {
    if let SplashAction::Notify { .. } = widget_action.cast() {
        // This will not receive Cx::post_action events.
    }
}
```

Allowed D1 events:

```text
inc
dec
reset
ask_ai
```

Behavior:

- `inc`: increment local count, persist state, re-render generated UI.
- `dec`: decrement local count, persist state, re-render generated UI.
- `reset`: set local count to `0`, persist state, re-render generated UI.
- `ask_ai`: send the current state and event payload to the active LLM.
- Unknown events: log and ignore.

Use a direct `match event_id.as_str()` for D1. A `HashMap` dispatch table is not
needed until there are many actions.

## D1 State Model

Target file:

- `examples/aichat/src/main.rs`

Initial state:

```json
{
  "count": 0
}
```

Implementation options:

- Prefer a small D1 state struct if only counter is required.
- Use `serde_json::Value` only if D2 Todo/Dashboard will follow immediately.
- If `serde_json` is added, mark it as a demo dependency in
  `examples/aichat/Cargo.toml`.

Persistence:

- Load state during startup.
- Save after every local state mutation.
- Keep chat history persistence unchanged.

## State Rendering: D1 Path

D1 should use host-side template replacement before Markdown/Splash rendering.

Supported placeholder:

```text
{{state.count}}
```

Rendering flow:

```text
raw assistant markdown
    -> render_state_templates(raw, APP_STATE)
    -> Markdown::set_text(rendered)
    -> runsplash block receives concrete text
```

Example:

```text
Label{ text: "Count: {{state.count}}" }
```

becomes:

```text
Label{ text: "Count: 3" }
```

This avoids relying on VM-level `state` for D1 and guarantees that
`Markdown::set_text` sees a changed string when count changes.

Requirements:

- Keep raw assistant messages unchanged for history and replay.
- Apply template rendering only on display.
- Do not run state-template replacement over an in-progress streaming message.
  Streaming text should render from `streaming_text` only; once `TurnComplete`
  stores it as an assistant message, it joins the normal raw-history display path.
- On `inc`, `dec`, or `reset`, redraw/re-render visible chat content from the
  raw message plus current state.
- Iterate the `chat_list` visible `PortalList` items and call
  `markdown.set_text(cx, &render_state_templates(raw, state))` for each visible
  assistant message. PortalList virtualization handles offscreen items: when they
  scroll into view, they render from raw history plus current state.
- D1 state is chat-global, not message-local. If multiple visible generated
  counters reference `{{state.count}}`, they intentionally show the same count.

The first implementation should avoid a template dependency. A small scanner is
enough:

```rust
fn render_state_templates(raw: &str, state: &AppDemoState) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;

    while let Some(start) = rest.find("{{state.") {
        out.push_str(&rest[..start]);
        let after_open = &rest[start + "{{state.".len()..];

        let Some(end) = after_open.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };

        let path = after_open[..end].trim();
        match path {
            "count" => out.push_str(&state.count.to_string()),
            _ => {
                log!("[render_state_templates] unknown path: {}", path);
                out.push_str("{{state.");
                out.push_str(path);
                out.push_str("}}");
            }
        }

        rest = &after_open[end + "}}".len()..];
    }

    out.push_str(rest);
    out
}
```

If D2 needs nested paths such as `{{state.todos.0.title}}`, extend this helper
or switch to a small template/parser dependency then. D1 should stay explicit.

## Why D1 Does Not Require VM `state.count`

VM-level state injection is attractive, but it creates extra implementation
surface:

- aichat owns `APP_STATE`, but `Splash` lives in `widgets`.
- `Splash` must not read `APP_STATE` directly.
- The host must update `vm.set_injected_global(id!(state), ...)` before forcing
  Splash re-eval.
- Existing `Splash::set_text` short-circuits when the body string is unchanged.

For D1, `{{state.count}}` is simpler and more reliable:

- crate boundaries stay clean.
- rendered markdown changes when state changes.
- no force-eval dependency is required for the counter display.

## Optional D1.5: VM-Level `state`

After D1 works, add VM-level `state` as a nicer authoring surface:

```splash
Label{ text: "Count: " + state.count.to_string() }
```

Rules:

- aichat injects state into the VM.
- `widgets/src/splash.rs` must not read aichat `APP_STATE`.
- aichat updates injected `state` before forcing Splash re-eval.
- `SplashRef::force_eval(cx)` may be added as an explicit API.
- `{{state.count}}` remains supported as a markdown-layer convenience. VM-level
  `state` is the preferred form for new LLM output after D1.5, but the two forms
  can coexist without conflict.

Possible force API:

```rust
impl Splash {
    pub fn force_eval(&mut self, cx: &mut Cx) {
        self.eval_body(cx);
        cx.redraw_all();
    }
}
```

This is a follow-up, not the required D1 path.

## Prompt Injection

Every prompt sent to an LLM should include current app state:

```text
[Current app state]
{
  "count": 3
}

[User]
<user text>
```

For click-driven AI events:

```text
[Current app state]
{
  "count": 3
}

[Event]
User clicked "ask_ai" with payload: {}
```

Backends that currently prepend system prompt text into the user message should
keep that behavior and add state inside the user message body.

## System Prompt Update

Add a small Splash-specific instruction block:

```text
When generating interactive runsplash UI, you may attach host actions to buttons:

Button{ text: "+1" on_click: || agent.notify("inc", {}) }

Supported demo actions:
- inc
- dec
- reset
- ask_ai

For the D1 counter, always use an empty object payload:

agent.notify("inc", {})
agent.notify("dec", {})
agent.notify("reset", {})

To display host state in D1, use template placeholders:
- {{state.count}}

Example:
Label{ text: "Count: {{state.count}}" }

Do not hardcode mutable state when the UI should reflect host state.
```

Do not introduce schema, capability declarations, or Robrix terminology in the
D1 prompt.

## Validation

Non-UI validation:

```bash
cargo check -p makepad-example-aichat
```

Runtime validation must use Studio remote release runs, not raw `cargo run`.

Before rerunning:

```json
{"ListBuilds":[]}
{"ClearBuild":{"build_id":[N]}}
```

Launch:

```json
{"RunItem":{"mount":"makepad","name":"makepad-example-aichat"}}
```

Manual acceptance flow:

1. Start a fresh release run through Studio.
2. Generate or paste a counter `runsplash` block using `{{state.count}}`.
3. Click `+1`.
4. Confirm the visible label changes from `Count: 0` to `Count: 1`.
5. Click `-1`.
6. Confirm the visible label changes.
7. Click `Reset`.
8. Confirm the visible label becomes `Count: 0`.
9. Restart the app.
10. Confirm persisted count is restored.
11. Click `Ask AI`.
12. Confirm the outgoing prompt includes current state.

First wire-level verification:

- Paste or hardcode `Button{text:"x" on_click: || agent.notify("test", {})}`
  directly into a Splash body.
- Click it.
- Confirm aichat `handle_actions` logs `event_id=test`.
- If nothing logs, agent injection failed.
- If the log shows the wrong event, the native args binding failed.

## Acceptance Criteria

D1 is complete when:

- `agent.notify("inc", {})` can be called from a generated Splash button.
- aichat receives `SplashAction::Notify` as a bare action.
- unknown event IDs are ignored with a log.
- `inc`, `dec`, and `reset` update local state.
- generated UI updates without a new LLM request.
- `ask_ai` sends current state back to the active LLM.
- state survives restart.
- validation is performed via Studio release run.

## Implementation Order

1. Add `SplashAction` in `widgets/src/splash.rs`.
2. Register/inject `agent.notify` in `widgets/src/splash.rs`.
3. Handle bare `SplashAction` in `examples/aichat/src/main.rs`.
4. Phase 1 gate: hardcode a minimal Splash button path and verify
   `agent.notify("test", {})` reaches aichat as a bare
   `SplashAction::Notify`. Do this before prompt or state work.
5. Add D1 count state and persistence.
6. Add `render_state_templates`, starting with `{{state.count}}`.
7. Route chat display through rendered markdown while preserving raw history.
8. Add `inc`, `dec`, `reset`, and `ask_ai` handlers.
9. Update system prompt.
10. Run `cargo check -p makepad-example-aichat`.
11. Validate in Studio release run.

## Later Extensions

### Runtime A2A / `agent2view`

- D2 Todo: add `todos` state and whitelisted `add_todo`, `delete_todo`,
  `toggle_todo` handlers.
- D3 Dashboard: add multiple state slices and handler groups.
- VM-level `state`: expose `state.count` after D1 is stable.
- Runtime capability declaration: optional fenced JSON block listing supported
  host state paths and actions.
- Handler table: replace `match` only when event count justifies it.

### Generative A2A / `agent2project`

Generative A2A needs a separate design before implementation. That design should
define:

- `appplan` as an artifact contract, not only a model planning block.
- output directory layout for generated Makepad apps.
- required files such as `Cargo.toml` and `src/main.rs`.
- resource handling.
- generated state modules.
- generated action/event handler code.
- Studio runnable-item integration.
- export/package validation.

Do not add these responsibilities to Runtime A2A D1.
