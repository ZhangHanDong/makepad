# 版本、兼容与验证

## AppType Version

`version` 是 AppType 级协议版本。

宿主可以支持多个 version。

不支持的 version 必须 fallback。

## 字段兼容

新增可选字段时，旧宿主可以忽略。

改变既有字段语义时，必须提升 version。

## State 兼容

State schema 应定义：

- 必需字段。
- 可选字段。
- 默认值。
- 枚举值。
- 最大数组长度或渲染截断规则。

Mission Room v1 template 可以限制显示数量，例如：

- tasks 前 6 个。
- agents 前 4 个。
- pending actions 前 3 个。

## aichat 验证

非 UI 验证：

```bash
cargo check -p makepad-example-aichat --release
cargo test -p makepad-example-aichat --release input
cargo test -p makepad-example-aichat --release collection
```

UI 验证必须通过 Studio remote release run，不以 raw `cargo run` 作为 UI 验证依据。

## Robrix2 验证

Robrix2 v1 验证：

- valid `org.octos.app` envelope 能渲染静态 Splash card。
- invalid envelope fallback 到 `body`。
- unknown AppType fallback。
- unsupported version fallback。
- unsafe template preflight fail。
- `message` scope 以 `room_id + event_id` 隔离。
- `room` scope 以 `room_id + app_id` 共享。
- mission room event 以 `room_id + mission.main` 作为实例 key。
- shared action 发送 `org.octos.action_response`，不直接改写 shared truth。
