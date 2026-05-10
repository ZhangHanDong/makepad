# Appendix: JSON Examples and Binding Evolution

The examples below keep the current Matrix binding field name `initial_state`. In the core semantics of this version, it carries the AppInstance initial data, namely the `data` content of a DataSnapshot.

## Mission Room Event

```json
{
  "body": "Mission update: plan is waiting for approval.",
  "msgtype": "m.text",
  "org.octos.app": {
    "type": "mission_room",
    "version": 1,
    "scope": "room",
    "app_id": "mission.main",
    "initial_state": {
      "goal": {
        "title": "Ship room-scoped agent2view runtime",
        "status": "planning"
      },
      "phase": "planning",
      "tasks": [],
      "agents": [],
      "pending_human_actions": [],
      "decisions": [],
      "blockers": []
    }
  },
  "org.octos.actions": [
    {
      "id": "approve_plan",
      "label": "Approve plan",
      "style": "primary"
    },
    {
      "id": "request_plan_changes",
      "label": "Request changes",
      "style": "secondary"
    }
  ]
}
```

## Action Response

```json
{
  "org.octos.action_response": {
    "action_id": "approve_plan",
    "source_event_id": "$mission123",
    "app": {
      "type": "mission_room",
      "version": 1,
      "scope": "room",
      "app_id": "mission.main"
    }
  }
}
```

## Account Dashboard Event

```json
{
  "body": "Mission dashboard update.",
  "msgtype": "m.text",
  "org.octos.app": {
    "type": "mission_dashboard",
    "version": 1,
    "scope": "account",
    "app_id": "missions.global",
    "initial_state": {}
  }
}
```

## Splash Host API Evolution

The current aichat generated `runsplash` uses `agent.notify(event_id, payload)` as the local reverse channel:

```splash
Button {
    text: "+1"
    on_click: || agent.notify("inc", {})
}

TextInput {
    text: "{{state.input.new_item.value}}"
    on_change: |text| agent.notify("app.input.set", {key: "new_item", text: text})
}
```

This mechanism solves the controlled action boundary from Template to Host. It is not tied to the specific function name. Therefore, if Splash provides a higher-level Host API, the implementation can gradually reduce direct exposure of `agent.notify` and treat it as a legacy primitive.

Recommended generated code should use semantic APIs:

```splash
Button {
    text: "+1"
    on_click: || app.action("inc", {})
}

TextInput {
    text: "{{state.input.new_item.value}}"
    on_change: |text| app.input.set("new_item", text)
}

Button {
    text: "Add"
    on_click: || app.collection.add_from_input("items", "new_item")
}
```

The Host API should aim to:

- let generated UI express intent without directly mutating Host internals;
- provide a stable semantic API surface for common capabilities such as action, input, and collection;
- let the capability manifest declare API families instead of only free-form event strings;
- validate payload structure earlier at the API boundary;
- keep the Host as the data/state owner.

One compatible implementation shape is:

```text
app.collection.add_from_input("items", "new_item")
    -> app.action("app.collection.add_from_input", {collection: "items", input: "new_item"})
    -> internal SplashAction::Notify
    -> Host action dispatcher
```

In other words, `agent.notify` may remain as an internal transport primitive at first, while generated code should prefer the `app.*` API. After the Host API becomes stable enough, `agent.notify` can be reserved for debugging, low-level compatibility, or restoring old messages.

The boundary remains unchanged:

- Host API calls must pass action whitelist and payload schema validation.
- Unknown API/action calls must be logged and ignored.
- Draft state in input fields must not rewrite the `runsplash` source on every keypress.
- Local Host API must not be used to submit remote shared fact actions in scenarios such as Robrix2.
- Remote shared actions should still use their corresponding transport binding, such as `org.octos.action_response`.
