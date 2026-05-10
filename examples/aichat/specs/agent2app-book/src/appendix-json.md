# 附录：JSON 示例与绑定演进

下面示例沿用当前 Matrix binding 的 `initial_state` 字段名。按本版核心语义，它承载的是 AppInstance 的 initial data，也就是 DataSnapshot 的 `data` 内容。

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

## Splash Host API 演进方向

当前 aichat 的 generated `runsplash` 使用 `agent.notify(event_id, payload)` 作为本地反向通道：

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

这个机制解决的是 Template 到 Host 的受控 action 边界，而不是某个具体函数名。因此，若 Splash 提供更高层的 Host API，协议实现可以逐步把直接暴露的 `agent.notify` 收敛为 legacy primitive。

推荐生成代码改为使用语义化 API：

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

Host API 的设计目标：

- 让 generated UI 只表达 intent，不直接修改 Host 内部结构。
- 让 action、input、collection 等常见能力有稳定、语义化的 API surface。
- 让 capability manifest 可以声明 API family，而不是只声明一组自由字符串事件。
- 让 payload 结构在 API 边界更早被校验。
- 保持 Host 是 data/state owner。

一种兼容实现可以是：

```text
app.collection.add_from_input("items", "new_item")
    -> app.action("app.collection.add_from_input", {collection: "items", input: "new_item"})
    -> internal SplashAction::Notify
    -> Host action dispatcher
```

也就是说，`agent.notify` 可以先作为内部 transport primitive 存在，外部生成代码优先使用 `app.*` API。等 Host API 覆盖稳定后，`agent.notify` 可以只保留给调试、低层兼容或旧消息恢复。

边界仍然不变：

- Host API 必须经过 action whitelist 和 payload schema 校验。
- 未知 API/action 必须 log + ignore。
- 输入 draft state 不应每个 keypress 重写 `runsplash` 源码。
- 本地 Host API 不应被用于提交 Robrix2 这类远程 shared fact action。
- 远程共享 action 仍应通过对应 transport binding，例如 `org.octos.action_response`。
