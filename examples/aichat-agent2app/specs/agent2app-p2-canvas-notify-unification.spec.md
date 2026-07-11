spec: task
name: "Canvas adopts the agent.notify envelope"
tags: [makepad, agent2app, canvas, ws, protocol]
depends: [agent2app-p0-isolated-vm-agent-registration]
estimate: 1d
---

## Intent

The repo has two parallel agent2app channels: aichat's typed
`agent.notify(event_id, json_payload)` and tools/canvas's untyped button-name
broadcast (`ButtonAction::Clicked` → `bridge.send_event(name)`). Unify them:
canvas registers the same `agent` module in its Splash VMs, handles
`SplashAction::Notify`, and forwards a typed JSON envelope over its WS/HTTP
bridge, so external agents receive structured events from generated UIs
through one protocol.

## Decisions

- Canvas VM init calls `makepad_widgets::script_mod` (which registers the
  agent module per the P0 spec); no canvas-local copy of the registration.
- Bridge event envelope is a single-line JSON object:
  `{"type":"notify","event":"<event_id>","payload":<json>}`.
- Legacy button-name broadcast is kept for backward compatibility and sent as
  `{"type":"click","name":"<button_name>"}`; plain-string events are no
  longer emitted.
- Envelope construction lives in `tools/canvas/src/ws/` next to
  `stdio_bridge.rs`; `SplashAction::Notify` handling goes in
  `tools/canvas/src/app.rs` `handle_actions`.
- Envelope unit tests live in a `#[cfg(test)]` module beside the envelope
  constructor.

## Boundaries

### Allowed Changes
- tools/canvas/src/**
- tools/canvas/skills/**
- tools/canvas/examples/**

### Forbidden
- Do not modify widgets/** or examples/aichat-agent2app/**
- Do not change the WS/HTTP endpoint surface (ws://, POST /splash*, GET /event)
- Do not add new cargo dependencies

## Completion Criteria

Scenario: notify reaches the bridge as a typed envelope (critical)
  Test:
    Package: makepad-canvas
    Filter: test_notify_action_produces_notify_envelope
  Given a SplashAction::Notify with event_id "inc" and payload "{}"
  When the canvas action handler converts it for the bridge
  Then the emitted line is the JSON object
    | field   | value  |
    | type    | notify |
    | event   | inc    |
    | payload | {}     |

Scenario: button clicks become typed click envelopes
  Test:
    Package: makepad-canvas
    Filter: test_button_click_produces_click_envelope
  Given a ButtonAction::Clicked from a button named "start"
  When the canvas action handler converts it for the bridge
  Then the emitted line is `{"type":"click","name":"start"}`
  And no bare plain-string event is emitted

Scenario: malformed payload fails safe by wrapping as a string
  Test:
    Package: makepad-canvas
    Filter: test_invalid_payload_is_wrapped_as_string
  Given a SplashAction::Notify whose payload string is malformed, unparseable JSON
  When the envelope is constructed
  Then construction does not fail and does not drop the event
  And the payload field carries the raw text as a JSON string
  And the envelope itself is valid JSON

Scenario: empty event id is rejected with a log line
  Test:
    Package: makepad-canvas
    Filter: test_empty_event_id_is_logged_and_skipped
  Given a SplashAction::Notify with an empty event_id (invalid input)
  When the canvas action handler processes it
  Then the event is rejected: no envelope is sent to the bridge
  And a log line records the rejected event

Scenario: skill doc teaches the envelope
  Test:
    Package: makepad-canvas
    Filter: test_canvas_skill_documents_notify_envelope
  Given tools/canvas/skills/app/SKILL.md
  When it is scanned for the notify envelope
  Then it documents `agent.notify` and the `{"type":"notify"...}` format

## Out of Scope

- Removing the legacy click envelope (needs downstream driver migration first)
- aichat-side changes
- Capability/permission enforcement (P3 spec)
