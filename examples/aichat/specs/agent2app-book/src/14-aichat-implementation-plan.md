# aichat 落地计划

本章根据会议讨论纪要，收敛出 Agent2App 在 aichat 中的工程落地路线。aichat 的近期目标不是成为完整的 Robrix/Matrix 协作运行时，也不是承载 `agent2project` 的完整软件工程生命周期，而是先成为一个稳定的本地 `agent2view` Host 和轻量 Agent2App Studio 原型。

核心链路如下：

```text
用户需求
  -> Agent 生成 appplan + template
  -> aichat 校验 capability / schema
  -> Splash 渲染 inline view
  -> 用户动作回到 Host
  -> Host 修改 data/state
  -> view 刷新
  -> 测试通过后可导出 artifact
```

换句话说，aichat 应先把 `生成 -> 校验 -> 渲染 -> action -> state -> 测试 -> artifact` 这条本地闭环跑稳，再向 Robrix room app、A2UI、多 Host artifact 等方向扩展。

## 1. 稳定 inline runsplash

当前 aichat 已经具备几个基础能力：

- `appplan json` 与 `runsplash` 的 prompt 约束。
- Markdown 中 `appplan` 隐藏、`runsplash` 渲染的 UI 绑定。
- `agent.notify(event_id, payload)` 到 Host action dispatcher 的反向通道。
- counter、calculator、timer、collection/todo 等初始 Host action。

第一阶段要解决的是：不要把 Agent2App 只当作 Markdown code block。aichat 在收到 assistant response 时，应解析出结构化 artifact：

```text
AssistantMessage
  raw_markdown
  app_artifact {
    appplan
    template_format: "runsplash"
    template_code
    template_hash
    capabilities
  }
```

这样切换焦点、重启、重新打开历史消息时，aichat 应使用 `app_artifact.template_code` 渲染 UI，而不是把 Markdown 中的 fenced code 显示给用户。此前出现过“初次生成可以渲染，重启后显示代码”的问题，本质上就是结构化持久化边界尚未落地。

本阶段验收标准：

- assistant message 能同时保存原始 Markdown 和结构化 app artifact。
- 历史消息重新加载后渲染 UI，不显示 `appplan` / `runsplash` 源码。
- `runsplash` 源码 hash 可稳定计算，用于后续 receipt 与缓存。
- 解析失败时有明确 fallback：显示普通 Markdown，并给出 Host 侧错误。

## 2. 让 appplan 成为本地 ViewSpec

当前 `appplan json` 主要是 prompt 约束。下一步应把它变成 aichat 的本地 ViewSpec，而不是只在文本里提示模型。

推荐结构：

```json
{
  "kind": "agent2view",
  "app_type": "collection",
  "title": "Todo List",
  "view": {
    "format": "runsplash",
    "template_hash": "sha256:..."
  },
  "data_paths": ["/data/collections/items"],
  "state_paths": ["/state/inputs/new_item/value"],
  "actions": [
    "app.collection.add_from_input",
    "app.collection.toggle",
    "app.collection.delete"
  ],
  "capabilities": [
    "app.collection",
    "app.input"
  ]
}
```

aichat 渲染前必须执行本地校验：

- `actions` 必须在 Host whitelist 中。
- action payload 必须符合 schema。
- template 只能访问允许的 data/state path。
- template 不得使用高风险 shader、无限循环或未声明 API。
- `app_type` 必须存在对应 Host handler。

这个 ViewSpec 是未来 `.splashapp` artifact、A2UI 兼容和 Robrix room binding 的共同前置。

## 3. Host 持有状态

Agent2App 的状态边界必须清晰：

```text
generated runsplash:
  只描述 UI 和用户意图

aichat HostState:
  持有 input draft、counter、calculator、todo rows、timer state

aichat dispatcher:
  处理 add/toggle/delete/calc/timer 等动作
```

generated widget 不能成为业务状态 owner。原因包括：

- Markdown 或 Splash 重新渲染会重建 widget。
- list/portal 类型 widget 可能回收不可见项。
- streaming eval 或错误恢复可能替换 view。
- 用户输入 draft 如果绑定到 template source，会导致每个字符都触发重建和失焦。

因此，输入框、todo、calculator、timer 的状态都应由 HostState 持有。`runsplash` 只读取 projection，并通过 action 通知 Host。

本阶段需要特别保证：

- `TextInput.on_change` 只更新 Host draft state。
- 每个 keypress 不重写 `runsplash` 源码。
- collection add 从 Host draft 中消费输入。
- view refresh 后输入焦点和 draft 尽量保持。

## 4. 从 agent.notify 演进到 Host API

`agent.notify` 可以继续作为 legacy transport primitive，但不应长期作为推荐给模型的公开 API。更清晰的 generated API 应是：

```splash
app.input.set("new_item", text)
app.collection.add_from_input("items", "new_item")
app.collection.toggle("items", id)
app.collection.delete("items", id)
ai.talk("analyze this view")
```

内部可以继续映射到现有机制：

```text
app.collection.add_from_input(...)
  -> SplashAction::Notify
  -> aichat action dispatcher
```

这样做的价值是：

- capability manifest 可以声明 API family，而不是自由字符串事件。
- payload schema 可以绑定到具体 API。
- prompt 更短，更不容易写错。
- 未来 Robrix/Web Host 可以实现同名 API，而不暴露 aichat 内部事件名。

兼容策略：

- 短期继续支持 `agent.notify(event_id, payload)`。
- prompt 优先示范 `app.*` API。
- Host API 覆盖稳定后，把 `agent.notify` 降级为 legacy/debug primitive。

## 5. 建立 component skills

aichat 不应每次从零生成完整 widget tree。常见 app 类型应变成 component skill：

- counter
- calculator
- timer
- todo / collection
- poll
- dashboard card
- health card
- trip planner

每个 component skill 应包含：

- appplan schema。
- `runsplash` template skeleton。
- allowed actions。
- default state。
- payload schema。
- interaction tests。
- prompt examples。

模型生成时优先选择 AppType 并填配置。复杂自由 UI 仍然可以保留，但 calculator、todo、timer 这类必须尽量走组件化路径。这样能降低 token 成本，也能显著提升交互正确率。

## 6. Verification Harness

aichat 的评价标准不能停留在“能画出来”。最小 harness 应覆盖：

```text
render_ok
no_vm_error
click_button_dispatches_action
payload_schema_ok
state_mutation_ok
view_refresh_ok
text_input_keeps_focus
reload_renders_ui_not_code
```

建议先固定四类测试：

| AppType | 核心测试 |
| --- | --- |
| counter | `+1`、`-1`、reset 后状态正确 |
| calculator | `1 + 1 = 2`，operator 不应提前重复输入 |
| todo | 输入、添加、toggle、delete |
| timer | start、pause、reset、add minute |

后续再加入：

- screenshot 或视觉模型检查，确认图表、波形、关键内容实际可见。
- bad template 测试，确认 shader/math/VM 错误能被 guard 拒绝。
- streaming eval 测试，确认 incomplete block 不污染 VM。

这个 harness 既用于本地回归，也用于离线优化 prompt、component skill 和 per-model harness。

## 7. 导出 Agent2App artifact

当 aichat 能稳定生成、渲染和测试后，应支持导出 portable artifact。

推荐 envelope：

```json
{
  "agent2app_artifact": {
    "version": "0.1",
    "kind": "agent2view",
    "app_id": "todo.local",
    "title": "Todo List",
    "template": {
      "format": "runsplash",
      "hash": "sha256:...",
      "code": "RoundedView{...}"
    },
    "data_schema": {},
    "capabilities": [
      "app.input",
      "app.collection"
    ],
    "receipt": {
      "tests": [
        {
          "name": "add_item",
          "status": "pass"
        }
      ]
    }
  }
}
```

这个 artifact 是 Robrix/Matrix 的前置条件。aichat 先实现 import/export，Robrix 再消费同一个 artifact contract，而不是重新定义一套私有格式。

## 8. 推荐实施顺序

推荐按以下顺序推进：

1. 结构化保存 `appplan + runsplash`，解决重启后显示源码的问题。
2. 定义 aichat `AppArtifact` / `ViewSpec` Rust struct。
3. 把 action whitelist 和 payload schema 从 prompt 文本中抽出来，变成 Host 真实校验。
4. 把 `agent.notify` 包一层 `app.*` Host API。
5. 固化 `counter`、`calculator`、`todo`、`timer` 四个 AppType。
6. 建立交互测试 harness，先覆盖这四类。
7. 增加 artifact export/import。
8. 再考虑 A2UI view format 和 Robrix room binding。

## 9. 阶段边界

aichat 的近期边界：

- 做 local `agent2view` Host。
- 做 lightweight Agent2App Studio。
- 允许 LLM-generated `runsplash`，但受 manifest、schema、guard 和 harness 约束。
- 不直接承担 Matrix room state、多人协作和加密 room data。
- 不承担完整 `agent2project` 构建、发布和审核生命周期。

Robrix 的后续边界：

- 作为 room Host 消费已定义的 artifact。
- 负责 Matrix event/state projection。
- 负责 room action 写回和多用户同步。
- 第一阶段不直接运行任意 LLM-generated template，优先使用静态模板或受信 artifact。

最终关系应保持清晰：

```text
aichat creates and verifies local agent2view artifacts.
Robrix imports, shares, and collaborates on room-scoped artifacts.
Protocol connects both without binding to either product.
```

