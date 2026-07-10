spec: task
name: "AI Chat Agent Notify Live Wires"
tags: [makepad, aichat, splash, agent-notify, state, demo]
---

## Intent

Add the smallest useful agent-to-app demo path to `makepad-example-aichat-agent2app`: LLMs
can generate interactive `runsplash` UI, generated buttons can call
`agent.notify(event_id, payload)`, the host can receive those events, update
local state, and re-render the generated UI from that state.

This spec targets the D1 counter demo only. It intentionally avoids the larger
Robrix schema/template/whitelist/reconciler design.

Implementation details are synchronized with
[AGENT-2-APP-DESIGIN.md](AGENT-2-APP-DESIGIN.md). If the two documents appear to
conflict, treat `AGENT-2-APP-DESIGIN.md` as the implementation-grade source and
update this task spec to match.

## Non-Goals

- No Robrix schema or state version negotiation.
- No widget template whitelist.
- No generic host function registry.
- No path-scoped incremental state updates.
- No resolver/template-author multi-LLM split.
- No repair loop for invalid Splash code.
- No Todo or Dashboard demo implementation in this phase.

## User-Facing Behavior

A user can ask:

```text
Draw a counter. The initial value should come from current state.
```

The LLM may respond with a `runsplash` block like:

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

- Clicking `+1` increments `state.count` and immediately re-renders the UI.
- Clicking `-1` decrements `state.count` and immediately re-renders the UI.
- Clicking `Reset` sets `state.count` to `0` and immediately re-renders the UI.
- Clicking `Ask AI` sends the current state and click event back to the active LLM.
- Restarting the app restores the last persisted count.

## Architecture

The minimum loop is:

```text
runsplash Button
    -> agent.notify(event_id, payload)
    -> Cx::post_action(SplashAction::Notify)
    -> App::handle_actions
    -> action whitelist
    -> APP_STATE update or agent prompt
    -> re-render visible assistant markdown from raw history plus current state
```

## Crate Boundary Rules

- `agent.notify` registration lives in `widgets/src/splash.rs`, not
  `platform/script/src`.
- `SplashAction` also lives in `widgets/src/splash.rs`, so both `Splash` and
  `examples/aichat-agent2app` can use the same action type.
- `makepad-script` must not depend on `Cx::post_action` or any Splash/aichat
  action type.
- `Cx::post_action` emits a bare/global action, not a widget action.

## Splash Agent API

Target file:

- `widgets/src/splash.rs`

Add a widget/global action:

```rust
#[derive(Clone, Debug, Default)]
pub enum SplashAction {
    Notify { event_id: String, payload: String },
    #[default]
    None,
}
```

Expose this API to every Splash eval context:

```splash
agent.notify("inc", {})
agent.notify("delete_todo", {id: 3})
```

Requirements:

- `agent.notify` accepts:
  - arg 0: event id, string-like; missing or invalid becomes an empty string.
  - arg 1: payload, any Splash value; serialized with the VM's JSON conversion.
- It posts a global Makepad action:

```rust
Cx::post_action(SplashAction::Notify { event_id, payload })
```

- It returns `nil`.
- It works for normal `Splash::eval_body`, streaming `Splash::stream_append`,
  and button `on_click` callbacks after initial eval.

Implementation notes:

- Do not assume a `make_native_fn(args: &[...])` API.
- Use the existing `ScriptVm` / `ScriptNative` native method registration style
  used by `platform/script/src/mod_std.rs`.
- Register `agent` once per VM lifecycle with an injected global, not once per
  `Splash::eval_body`. A clean shape is
  `pub fn register_agent_module(vm: &mut ScriptVm)` in `widgets/src/splash.rs`,
  called during widget/app script initialization before generated Splash code is
  evaluated.
- Use `vm.set_injected_global(id!(agent), agent_obj.into())` so Splash code can
  reference top-level `agent`.
- Serialize payload with `vm.bx.heap.to_json_inner(payload_value, &mut payload)`.
  Do not use `cast_to_string` for payload objects, because `{}` becomes
  `"[ScriptObject]"`.
- Do not use `DefaultNone`; use `#[derive(Default)]` and `#[default]`.

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

## Host Action Handling

Target file:

- `examples/aichat-agent2app/src/main.rs`

Because `Cx::post_action` produces a bare action, aichat must not use
`as_widget_action()` for this event.

Correct:

```rust
for action in actions {
    if let SplashAction::Notify { event_id, payload } = action.cast() {
        self.handle_splash_event(cx, &event_id, &payload);
    }
}
```

Wrong:

```rust
if let Some(widget_action) = action.as_widget_action() {
    if let SplashAction::Notify { .. } = widget_action.cast() {
        // This will not receive Cx::post_action events.
    }
}
```

## Host App State

Target files:

- `examples/aichat-agent2app/src/main.rs`
- optionally `examples/aichat-agent2app/Cargo.toml`

Initial state shape:

```json
{
  "count": 0
}
```

Requirements:

- State is process-global or App-owned and protected by `RwLock`.
- State loads during app startup.
- State saves after every local state-changing action.
- State is included in every prompt sent to the LLM.

Acceptable persistence:

- A demo-simple JSON file next to the existing chat history save file.
- Or the existing project style with `makepad_micro_serde`.

If `serde_json` is added, update `examples/aichat-agent2app/Cargo.toml` explicitly.

## State Template Rendering

Problem:

LLM output may be reused after local state changes. If the generated Splash
contains hardcoded text, replaying the same raw response will not update UI.

D1 solution:

- Host supports template placeholders inside assistant markdown before rendering:

```text
{{state.count}}
```

Requirements:

- Store raw assistant response unchanged.
- Before calling `Markdown::set_text`, render a display string by replacing
  supported state placeholders.
- For D1, only `{{state.count}}` is required.
- Unknown state paths should log and preserve the original placeholder text.
- Do not run state-template replacement over an in-progress streaming message.
  Streaming text renders from `streaming_text` only; once `TurnComplete` stores
  it as an assistant message, it joins the normal raw-history display path.
- Local actions trigger re-render of visible assistant messages.
- D1 state is chat-global, not message-local. If multiple visible generated
  counters reference `{{state.count}}`, they intentionally show the same count.

Implementation shape:

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

Non-requirements:

- No full JSONPath.
- No arbitrary expression evaluation.
- No Splash-level `state` object in D1.

## Action Whitelist

Target file:

- `examples/aichat-agent2app/src/main.rs`

Allowed D1 actions:

```text
inc
dec
reset
ask_ai
```

Behavior:

- `inc`: increment `state.count`, persist, re-render visible assistant UI.
- `dec`: decrement `state.count`, persist, re-render visible assistant UI.
- `reset`: set `state.count = 0`, persist, re-render visible assistant UI.
- `ask_ai`: send a prompt to the current agent with event, payload, and current state.
- Unknown events: log and drop.

Requirements:

- Unknown event IDs must not invoke arbitrary behavior.
- Payload should be logged or forwarded only for whitelisted actions.
- First implementation can use `match event_id.as_str()` rather than a `HashMap`.

## Prompt Injection

Update aichat prompt construction so every user prompt includes current state:

```text
[Current app state]
{
  "count": 3
}

[User]
<user text>
```

Click-driven AI actions should send:

```text
[Current app state]
{
  "count": 3
}

[Event]
User clicked "ask_ai" with payload: {}
```

For backends that need the system prompt prepended into user text, preserve the
existing behavior and include state inside the user prompt body.

## System Prompt Update

Update Splash-specific instructions with the minimum new capability:

```text
When generating interactive runsplash UI, you may attach button actions with:

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

To display host state, use template placeholders in string literals:
- {{state.count}}

Example:
Label{ text: "Count: {{state.count}}" }

The host will replace placeholders before rendering and will handle allowed
notify events.
```

The prompt should tell the LLM not to hardcode mutable state when it should
reflect host state. Keep the prompt small; do not introduce schema or capability
declarations.

## Replay And Re-Render Semantics

After a local state mutation:

- Save state.
- Re-render visible assistant messages from raw markdown through
  `render_state_templates`.
- Redraw the chat list.

Implementation choices:

- Either update the stored assistant message display path without changing raw
  history.
- Or invalidate the visible Markdown/Splash widget so it receives new rendered
  text.

Must account for:

- `Markdown::set_text` skips work if text is identical.
- `Splash::set_text` skips work if body is identical.
- Therefore the rendered text must actually change when `count` changes, or a
  force-refresh path must be used.
- Iterate the `chat_list` visible `PortalList` items and call
  `markdown.set_text(cx, &render_state_templates(raw, state))` for each visible
  assistant message. PortalList virtualization handles offscreen items: when they
  scroll into view, they render from raw history plus current state.

For D1, prefer rendered text changes via `{{state.count}}`.

## Optional D1.5: VM-Level State

After D1 works, VM-level state may be added as a nicer Splash authoring surface:

```splash
Label{ text: "Count: " + state.count.to_string() }
```

Rules:

- aichat injects state into the VM.
- `widgets/src/splash.rs` must not read aichat `APP_STATE`.
- aichat updates injected `state` before forcing Splash re-eval.
- `{{state.count}}` remains supported as a markdown-layer convenience. VM-level
  `state` is the preferred form for new LLM output after D1.5, but both forms
  can coexist without conflict.

## Testing

Minimum non-UI checks:

- `cargo check -p makepad-example-aichat-agent2app`
- Focused unit tests if helper functions are extracted:
  - `render_state_templates("Count: {{state.count}}")`
  - unknown placeholders
  - prompt-with-state formatting
  - action handler state updates if easily separable

Runtime validation:

- Use the Studio remote protocol per the repository runbook.
- Launch the release runnable item:

```json
{"RunItem":{"mount":"makepad","name":"makepad-example-aichat-agent2app"}}
```

Before each rerun:

```json
{"ListBuilds":[]}
{"ClearBuild":{"build_id":[N]}}
```

Validation steps:

1. Start app.
2. Send or load a prompt that produces counter runsplash.
3. Click `+1`.
4. Verify label changes from `Count: 0` to `Count: 1`.
5. Click `-1`.
6. Verify label changes.
7. Click `Reset`.
8. Verify `Count: 0`.
9. Restart app.
10. Verify persisted count.
11. Click `Ask AI`.
12. Verify a new LLM prompt is sent with current state.

First wire-level verification:

- Paste or hardcode `Button{text:"x" on_click: || agent.notify("test", {})}`
  directly into a Splash body.
- Click it.
- Confirm aichat `handle_actions` logs `event_id=test`.
- If nothing logs, agent injection failed.
- If the log shows the wrong event, the native args binding failed.

## Acceptance Criteria

D1 is complete when:

- A generated Splash button can call `agent.notify`.
- aichat receives `SplashAction::Notify` as a bare action in
  `App::handle_actions`.
- `inc`, `dec`, and `reset` update host state.
- Counter UI updates without a new LLM request.
- `ask_ai` sends current state back to the LLM.
- State survives app restart.
- Unknown events are ignored with a log.
- Validation is done through a Studio release run, not raw `cargo run`.

## Suggested Implementation Order

1. Add `SplashAction` in `widgets/src/splash.rs`.
2. Register/inject `agent.notify` in `widgets/src/splash.rs`.
3. Handle bare `SplashAction` in `examples/aichat-agent2app/src/main.rs`.
4. Phase 1 gate: hardcode a minimal Splash button path and verify
   `agent.notify("test", {})` reaches aichat as a bare
   `SplashAction::Notify`. Do this before prompt or state work.
5. Add D1 count state and persistence.
6. Add `render_state_templates`, starting with `{{state.count}}`.
7. Route chat display through rendered markdown while preserving raw history.
8. Add `inc`, `dec`, `reset`, and `ask_ai` handlers.
9. Add state prompt injection.
10. Update system prompt.
11. Run `cargo check -p makepad-example-aichat-agent2app`.
12. Run Studio release validation.
