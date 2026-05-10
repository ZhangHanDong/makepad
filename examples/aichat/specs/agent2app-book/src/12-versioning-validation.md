# 版本、兼容与验证

Agent2App 版本分为三层：core version、binding version 和 AppType/profile version。

## Core Version

`core_version` 描述协议内核版本。

它控制：

- 对象模型。
- ScopeKey 语义。
- DataSnapshot 语义。
- Host State 语义。
- ViewSpec 语义。
- Action / ActionResult 语义。
- CapabilityManifest 语义。
- 通用校验顺序。

本地 Agent 与远程 Agent 必须使用一致的 core version 语义。

## Binding Version

`binding_version` 描述 transport 映射版本。

例如：

- aichat local binding 如何编码 `appplan json`、`runsplash` 和 `agent.notify`。
- Robrix2 Matrix binding 如何编码 `org.octos.app` 和 `org.octos.action_response`。

Binding version 变化不应改变 core semantic，只应改变承载格式或 transport-specific metadata。

当 binding 承载 A2UI 时，还应声明 view encoding version，例如 `view.format = "a2ui"` 与 `view.version = "v0.9"`。A2UI 版本变化只影响 view encoding，不应改变 Agent2App 的 Scope、DataSnapshot、Host State 或 ActionResult 语义。

## AppType / 场景版本

`version` 是 AppType 或 profile 级协议版本。

它控制：

- 具体 data schema。
- 可暴露的 state path。
- 具体 template id。
- 具体 action set。
- 具体 profile policy。

宿主可以支持多个 version。

不支持的 version 必须 fallback。

## 字段兼容

新增可选字段时，旧宿主可以忽略。

改变既有字段语义时，必须提升相关版本。

版本提升规则：

- 改变通用对象语义：提升 core version。
- 改变 transport 编码：提升 binding version。
- 改变某个 AppType data/state/action/template 语义：提升 AppType/profile version。

## Data / State 兼容

Data schema 应定义：

- 必需字段。
- 可选字段。
- 默认值。
- 枚举值。
- 最大数组长度或渲染截断规则。

State path policy 应定义：

- 哪些 Host state path 可被 view 读取。
- 哪些 Host state path 可被 input 组件本地写入。
- 哪些 state path 可以作为 action payload 来源。
- state 是否需要随 session、room 或 account 持久化。

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
