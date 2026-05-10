# Versioning, Compatibility, and Validation

Agent2App versioning has three layers: core version, binding version, and AppType/profile version.

## Core Version

`core_version` describes the protocol kernel version.

It controls:

- object model;
- ScopeKey semantics;
- Snapshot semantics;
- Action / ActionResult semantics;
- CapabilityManifest semantics;
- shared validation order.

Local Agents and remote Agents must use consistent core version semantics.

## Binding Version

`binding_version` describes the transport mapping version.

For example:

- how aichat local binding encodes `appplan json`, `runsplash`, and `agent.notify`;
- how Robrix2 Matrix binding encodes `org.octos.app` and `org.octos.action_response`.

Binding version changes should not change core semantics. They should only change carrier format or transport-specific metadata.

When a binding carries A2UI, it should also declare a view encoding version, for example `view.format = "a2ui"` and `view.version = "v0.9"`. A2UI version changes affect only view encoding and should not change Agent2App Scope, Snapshot, or ActionResult semantics.

## AppType / Scenario Version

`version` is the AppType or profile-level protocol version.

It controls:

- concrete state schema;
- concrete template id;
- concrete action set;
- concrete profile policy.

The Host may support multiple versions.

Unsupported versions must fall back.

## Field Compatibility

When adding optional fields, older Hosts may ignore them.

When changing the semantics of an existing field, the relevant version must be bumped.

Version bump rules:

- Change shared object semantics: bump core version.
- Change transport encoding: bump binding version.
- Change a specific AppType state/action/template semantic: bump AppType/profile version.

## State Compatibility

State schema should define:

- required fields;
- optional fields;
- default values;
- enum values;
- maximum array lengths or render truncation rules.

The Mission Room v1 template may limit visible counts, for example:

- first 6 tasks;
- first 4 agents;
- first 3 pending actions.

## aichat Validation

Non-UI validation:

```bash
cargo check -p makepad-example-aichat --release
cargo test -p makepad-example-aichat --release input
cargo test -p makepad-example-aichat --release collection
```

UI validation must use a Studio remote release run. A raw `cargo run` is not accepted as UI validation.

## Robrix2 Validation

Robrix2 v1 validation:

- valid `org.octos.app` envelope renders a static Splash card;
- invalid envelope falls back to `body`;
- unknown AppType falls back;
- unsupported version falls back;
- unsafe template fails preflight;
- `message` scope is isolated by `room_id + event_id`;
- `room` scope is shared by `room_id + app_id`;
- mission room event uses `room_id + mission.main` as the instance key;
- shared action sends `org.octos.action_response` and does not directly rewrite shared truth.
