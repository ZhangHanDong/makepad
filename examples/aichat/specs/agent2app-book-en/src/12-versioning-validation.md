# Versioning, Compatibility, and Validation

## AppType Version

`version` is the protocol version at the AppType level.

The Host may support multiple versions.

Unsupported versions must fall back.

## Field Compatibility

When adding optional fields, older Hosts may ignore them.

When changing the semantics of an existing field, the version must be bumped.

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
