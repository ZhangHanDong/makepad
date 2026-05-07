# Robrix2 Profile

Robrix2 is the Matrix Event-Sourced profile. It renders Matrix event history and does not own a private RPC session with the producing Agent.

## V1 Display Cards

Robrix2 v1 cards should remain display-only:

- Weather.
- News.
- Static mission room phase 1.

These apps do not need LLM-generated templates or local reducers.

## AgentViewSession

The Robrix2 interactive runtime should use a structure similar to:

```rust
struct AgentViewSession {
    scope_key: AgentViewScopeKey,
    source_event_id: OwnedEventId,
    app_type: String,
    version: u32,
    template_id: String,
    state: serde_json::Value,
    dirty: bool,
}
```

The session map preserves state while a room is open.

```mermaid
flowchart LR
    EV[Matrix event] --> SK[ScopeKey]
    SK --> SM[SessionMap]
    SM --> S[AgentViewSession]
    S --> V1[Visible card A]
    S --> V2[Visible card B]
```

## Local Reducer Loop

The first local reducers should stay narrow:

```text
counter.inc
counter.dec
counter.reset
timer.toggle
timer.reset
```

Click loop:

```text
button action
  -> action carries scope key + action_id + payload
  -> lookup AgentViewSession
  -> run whitelisted local reducer
  -> re-render affected views
```

`message` scope redraws only one card.

`room` scope redraws all visible cards pointing to the same `room_id + app_id`.

## Relationship to aichat

Robrix2 can borrow aichat's narrow live wire, but it must not directly copy `agent.notify` as the production shared action protocol.

The aichat loop works because aichat owns:

- LLM session;
- local state;
- Splash VM;
- immediate event loop.

Robrix2 must keep shared truth in the Matrix timeline.
