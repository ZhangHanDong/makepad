# Agent2App 协议规范（中文草案）

> 状态：草案 v0.1  
> 范围：`agent2view` 协议；不包含 `agent2project` 代码生成协议。  
> 依据：`makepad-example-aichat` 当前 agent2app runtime、`AGENT-2-APP-DESIGIN.md`、Robrix2 `agent-to-app-simplified-design.md`、Robrix2 `agent-mission-room-design.md`。

## 1. 目标

本规范定义 Agent 如何把结构化应用视图交给宿主渲染，以及用户在视图中的操作如何回到宿主或 Agent。

规范的核心目标是统一两类已经出现的 agent2app 场景：

- aichat：LLM 在聊天里生成 `runsplash` 交互 UI，由 aichat 宿主持有状态并处理 `agent.notify`。
- Robrix2：Agent 通过 Matrix event 发送 `org.octos.app` envelope，由 Robrix2 用本地静态模板渲染 IM 内原生应用卡片。

这两个场景共享 “Agent 产生 app view，Host 负责渲染和边界控制” 的理念，但状态权威和 action 传输不同。本规范把差异显式化，避免把 aichat 的本地 runtime 机制错误复制到 Robrix2 的 Matrix 生产协议里。

## 2. 术语

### 2.1 Agent2App

`agent2app` 是总称，表示 Agent 输出可运行或可渲染应用能力的协议族。

它包含两条轨道：

- `agent2view`：Agent 输出一个由宿主渲染的内联应用视图。
- `agent2project`：Agent 生成独立项目、文件树、资源和运行/打包元数据。

本规范只定义 `agent2view`。

### 2.2 Agent2View

`agent2view` 表示 Agent 输出的内容最终成为宿主内的一个 live view。它不是独立项目，也不拥有自己的完整进程、文件树或打包生命周期。

### 2.3 AppType

App 类型的全局定义，例如：

- `counter`
- `timer`
- `calculator`
- `collection`
- `weather`
- `news`
- `mission_room`
- `mission_dashboard`

AppType 决定：

- 支持的协议版本。
- `initial_state` 的 schema。
- 可用模板。
- 可用 action。
- 本地 reducer 或共享 action response 的路由规则。

### 2.4 AppInstance

App 实例是某个 AppType 在某个 scope 下的具体运行实体。

AppInstance 不等同于单个消息。`room` scoped app 可以被多条消息渲染并指向同一个实例。

### 2.5 View

View 是 AppInstance 的一次可见渲染。

在 aichat 中，一个 assistant message 里的 `runsplash` block 是一个 view。

在 Robrix2 中，一个 Matrix timeline item 渲染出的 Splash card 是一个 view。

### 2.6 HostState

HostState 是宿主持有的状态。

它可以是：

- aichat 的 `APP_DEMO_STATE`。
- Robrix2 的 `AgentViewSession.state`。
- Matrix event snapshot 投影出的 session state。

Widget 内部状态不是 HostState，不能作为共享事实来源。

### 2.7 Template

Template 是把 state 渲染成 UI 的描述。

aichat 支持 LLM 生成的 `runsplash`。

Robrix2 v1 只允许本地静态 `.splash` template，不允许 Matrix event 或 LLM 直接提供运行时模板。

### 2.8 Action

Action 是用户或 view 发出的意图。

Action 分两类：

- Local View Action：只影响当前本地 view/session。
- Shared Fact Action：改变共享事实，必须由 Agent/producer 验证后产生新 snapshot。

## 3. 协议轨道

### 3.1 Agent2View

Agent2View 的基本流程是：

```text
Agent output
  -> Host parses app/view contract
  -> Host validates app type, state, template, actions
  -> Host renders live view
  -> User interacts
  -> Host routes action
  -> Host re-renders or emits action response
```

Agent2View 必须满足：

- 宿主控制渲染边界。
- 宿主控制 state authority。
- 宿主控制 action whitelist。
- 视图销毁和重建不能丢失权威状态。

### 3.2 Agent2Project

Agent2Project 是未来轨道，用于生成独立 Makepad 应用或项目。

它需要独立 artifact manifest，至少包含：

- 文件树。
- `Cargo.toml`。
- `src/main.rs`。
- 资源文件。
- 状态模块。
- 事件/action handler。
- Studio runnable item。
- 导出/打包元数据。

Agent2Project 不得复用 Agent2View 的 runtime capability manifest。前者是构建产物合同，后者是宿主运行能力边界。

## 4. 两种 Agent2View 模式

### 4.1 Local Runtime 模式

Local Runtime 模式用于 aichat。

```text
LLM response
  -> appplan json block
  -> runsplash block
  -> Splash inline render
  -> agent.notify(event_id, payload)
  -> Host action handler
  -> HostState mutation or LLM prompt
  -> visible view re-render
```

特征：

- Agent 与 Host 在同一个本地交互闭环里。
- Host 拥有当前 LLM session。
- Host 拥有 local state。
- Host 可以立即处理按钮事件并更新 UI。
- `agent.notify` 是可接受的本地 reverse channel。

### 4.2 Matrix Event-Sourced 模式

Matrix Event-Sourced 模式用于 Robrix2。

```text
Agent-produced Matrix event
  -> org.octos.app envelope
  -> AppRegistry lookup
  -> initial_state validation
  -> static Splash template render
  -> RoomScreen displays card
```

特征：

- Matrix timeline 是共享审计日志。
- Agent/producer 是共享事实的来源。
- Robrix2 是 event consumer，不是 Agent 的私有 RPC peer。
- 共享变更必须通过 Matrix/OctOS action response，再由 Agent 发送新 snapshot event。
- Robrix2 v1 不接受 LLM 生成的运行时 Splash template。

## 5. Envelope

### 5.1 标准 Envelope

Matrix Event-Sourced 模式使用 `org.octos.app` envelope：

```json
{
  "body": "Plain text fallback",
  "msgtype": "m.text",
  "org.octos.app": {
    "type": "mission_room",
    "version": 1,
    "scope": "room",
    "app_id": "mission.main",
    "initial_state": {}
  }
}
```

### 5.2 字段定义

`body`：

- 必须提供。
- 是富渲染失败时的 fallback。
- 应该能独立表达消息摘要。

`org.octos.app.type`：

- 必须提供。
- 必须匹配本地注册的 AppType。

`org.octos.app.version`：

- 必须提供。
- 表示该 AppType 的协议版本。
- 宿主必须拒绝不支持的版本并 fallback 到 `body`。

`org.octos.app.scope`：

- 可选，默认 `message`。
- 允许值：`message`、`room`、`account`。

`org.octos.app.app_id`：

- `message` scope 可省略。
- `room` 和 `account` scope 必须提供。
- 同一 scope 内必须稳定。

`org.octos.app.initial_state`：

- 必须提供。
- 必须是完整 snapshot。
- 必须由 AppType 对应 schema 校验。
- 不应被解释为 patch。

### 5.3 Fallback 规则

任一环节失败，宿主必须渲染 `body`，不得进入第二条不受控 rich render 路径。

失败包括：

- 未知 AppType。
- 不支持的 version。
- 非法 scope。
- 缺失必需 `app_id`。
- `initial_state` 校验失败。
- 模板缺失。
- 模板 preflight 失败。
- state binding 无法解析。

## 6. Scope 与实例身份

### 6.1 ScopeKey

Agent2View 使用三个 scope：

```text
message: room_id + event_id
room:    room_id + app_id
account: account_id + app_id
```

### 6.2 Message Scope

`message` scope 表示一个 Matrix event 拥有一个独立 app instance。

适用场景：

- weather card。
- news card。
- 单条消息内的静态信息视图。

实例 key：

```text
room_id + event_id
```

### 6.3 Room Scope

`room` scope 表示同一房间内多条消息可以指向同一个 app instance。

适用场景：

- `mission_room`。
- 房间级任务面板。
- 房间级 agent roster。

实例 key：

```text
room_id + app_id
```

### 6.4 Account Scope

`account` scope 表示同一账号下跨房间共享一个 app instance。

适用场景：

- `mission_dashboard`。
- 全局 agent operations overview。

实例 key：

```text
account_id + app_id
```

Account scope 不应替代 room-scoped truth。它只能汇总多个 room 的状态。

## 7. State 模型

### 7.1 三层状态

Agent2View 必须区分三层状态：

```text
Shared truth      = Agent-produced Matrix event snapshot
Host view state   = Host session state
Widget state      = disposable UI state
```

### 7.2 Shared Truth

Shared truth 是可以被房间参与者审计和同步的事实。

在 Robrix2 中，Shared truth 必须来自 Matrix event 的原始 content。

共享状态变更必须由 Agent/producer 发送新 snapshot event，不得由本地 widget 静默改写 Matrix 历史。

### 7.3 Host View State

Host view state 是宿主为渲染和交互保留的本地状态。

它用于：

- 抵抗 `PortalList` virtualization 导致的 widget 销毁。
- 支持本地 UI reducer。
- 支持 room/account scoped instance 的可见 view 同步重绘。
- 保存 volatile input、selection、filter、expanded/collapsed 等 UI 状态。

Robrix2 初版可只在内存中保存 Host view state。持久化是后续工作。

### 7.4 Widget State

Widget state 只能是临时渲染状态，例如：

- 输入法 composition。
- hover/focus。
- scroll offset。
- animation progress。

Widget state 不得作为 AppInstance 的权威状态。

### 7.5 Snapshot Projection

对 `room` 和 `account` scoped app，Matrix event 是完整 snapshot。

投影规则：

- 较新的合法 snapshot 替换 session shared state。
- 较旧 timeline item 被重绘时不得回滚 session。
- 同一 event 的重复 redraw 不得清空本地 dirty reducer state。
- 新 producer snapshot 到达时清除 dirty，因为共享事实已经前进。

## 8. Template 与 Rendering

### 8.1 Aichat Rendering

aichat 使用 LLM 生成的 `runsplash`：

````markdown
```runsplash
View {
    Label { text: "Count: {{state.count}}" }
    Button { text: "+1" on_click: || agent.notify("inc", {}) }
}
```
````

渲染规则：

- 原始 assistant message 必须保留。
- 显示时执行 `{{state.path}}` 替换。
- state mutation 后只重绘可见 assistant message。
- offscreen message 由 PortalList 在重新进入视口时从 raw message + current state 渲染。

### 8.2 Volatile Input 规则

输入框 draft state 不得每个字符写回 `runsplash` 源码。

例如：

```splash
TextInput {
    text: "{{state.input.new_item.value}}"
    on_change: |text| agent.notify("app.input.set", {key: "new_item", text: text})
}
```

宿主显示渲染时应该把 `{{state.input.*.value}}` 当作 volatile placeholder，避免每次输入导致 Markdown/Splash `set_text` 看到新源码并重建整个 UI。

HostState 仍然保存 draft text；添加 item 等 action 可以从 HostState 读取该 input。

### 8.3 Robrix2 Rendering

Robrix2 v1 使用本地静态 `.splash` template。

流程：

```text
org.octos.app envelope
  -> app_registry
  -> AppFactory::init(initial_state)
  -> RenderedApp::render(language)
  -> SplashHost loads static template
  -> $state.* binding
  -> RoomScreen displays card
```

模板允许：

- 使用 `widget_manifest` 中允许的 Makepad widget。
- 使用 AppType 声明过的 `$state.path`。
- 使用注册过的 local render helper。

模板禁止：

- 由 Matrix event 提供。
- 由 LLM 运行时生成。
- 引用未知 widget。
- 引用未知 helper。
- 引用未声明 state path。
- 设置 host-owned attribution 字段，例如 `capability_id`、`display_name`、`icon`、`trust_badge`。

## 9. Action 协议

### 9.1 Action 分类

Action 分为两类：

- Local View Action：只改变本地 view/session。
- Shared Fact Action：改变共享事实，必须通过 Agent/producer。

### 9.2 Local View Action

Local View Action 适用于：

- 展开/折叠任务详情。
- 选择 agent。
- filter/sort/tab 切换。
- aichat counter/timer/calculator/collection 操作。

Local reducer lookup 必须按 AppType + action id 白名单分发。

Robrix2 不应使用无限字符串的全局 `agent.notify("anything")` 作为生产 action 面。

### 9.3 Aichat `agent.notify`

aichat 使用 `agent.notify(event_id, payload)`：

```splash
agent.notify("inc", {})
agent.notify("app.collection.add_from_input", {collection: "items", input: "new_item"})
```

实现边界：

- `agent.notify` 注册在 `widgets/src/splash.rs`。
- `SplashAction` 定义在 `widgets/src/splash.rs`。
- `SplashAction::Notify` 是 bare action，不是 widget action。
- aichat 在 `handle_actions` 中直接遍历 `actions` 并 cast。

宿主必须：

- 校验 event id。
- 校验 payload JSON。
- 对未知 event log + ignore。
- 对非法 payload log + ignore。

### 9.4 Robrix2 Shared Action

Robrix2 的共享 action 使用 `org.octos.action_response`。

示例：

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

规则：

- `source_event_id` 指向产生该 action 的原始 Matrix event。
- `app` 字段用于在同一房间多 app 实例时明确路由。
- Producer 负责验证 action response。
- Producer 应用策略后发送新的 app snapshot event。
- Robrix2 可以做 optimistic local view update，但不得把 optimistic state 当作 shared truth。

## 10. Capability Manifest

### 10.1 Runtime Capability Manifest

Runtime manifest 描述宿主当前 view runtime 能力：

- 可读 state path。
- 可调用 action。
- action payload schema。
- view 生命周期规则。

aichat prompt 当前使用此类 manifest 引导 LLM 生成 `runsplash`。

### 10.2 Artifact Manifest

Artifact manifest 属于 `agent2project`，描述生成项目：

- 文件。
- 目录。
- dependency。
- resource。
- runnable item。
- package/export target。

Runtime manifest 与 Artifact manifest 不得混用。

## 11. Aichat Profile

### 11.1 AppPlan

aichat AppGen response 应包含一个 `appplan json` block 和一个 `runsplash` block。

`appplan` 是生成 UI 的计划与能力声明，不是独立项目 artifact。

### 11.2 支持的 Host State

当前 aichat HostState 包含：

- `count`
- `timer`
- `calculator`
- `collections`
- `inputs`

示例 state path：

```text
{{state.count}}
{{state.timer.display}}
{{state.calculator.display}}
{{state.collection.items.rows}}
{{state.collection.items.count}}
{{state.input.new_item.value}}
```

### 11.3 支持的 Actions

Counter：

```text
inc
dec
reset
```

Timer：

```text
timer.start
timer.pause
timer.toggle
timer.reset
timer.add_minute
timer.subtract_minute
```

Calculator：

```text
calculator.digit.0 ... calculator.digit.9
calculator.decimal
calculator.operator.add
calculator.operator.subtract
calculator.operator.multiply
calculator.operator.divide
calculator.equals
calculator.clear
calculator.backspace
calculator.sign
calculator.percent
```

Collection：

```text
app.input.set
app.collection.add_from_input
app.collection.add
app.collection.toggle
app.collection.delete
app.collection.clear
```

AI callback：

```text
ask_ai
```

## 12. Robrix2 Profile

### 12.1 V1 Display Cards

Robrix2 v1 card 应保持 display-only：

- Weather。
- News。
- Static mission room phase 1。

这些 app 不需要 LLM-generated template，不需要 local reducer。

### 12.2 AgentViewSession

Robrix2 interactive runtime 应使用类似结构：

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

Session map 负责在 room open 期间保存状态。

### 12.3 Local Reducer Loop

第一批 local reducer 应保持窄：

```text
counter.inc
counter.dec
counter.reset
timer.toggle
timer.reset
```

Click loop：

```text
button action
  -> action carries scope key + action_id + payload
  -> lookup AgentViewSession
  -> run whitelisted local reducer
  -> re-render affected views
```

`message` scope 只重绘一张 card。

`room` scope 重绘所有指向同一 `room_id + app_id` 的可见 card。

## 13. Mission Room Profile

### 13.1 App Envelope

Mission Room 必须使用 `room` scope：

```json
{
  "org.octos.app": {
    "type": "mission_room",
    "version": 1,
    "scope": "room",
    "app_id": "mission.main",
    "initial_state": {}
  }
}
```

默认 mission app id：

```text
mission.main
```

除非一个 room 明确承载多个 mission，否则 producer 应使用该默认 id。

### 13.2 Mission State V1

Mission state v1 应保持显式且小：

```json
{
  "goal": {
    "title": "Ship room-scoped agent2view runtime",
    "status": "planning"
  },
  "phase": "planning",
  "tasks": [
    {
      "id": "task-1",
      "title": "Define AgentViewSession state model",
      "status": "planning",
      "owner_agent": "planner",
      "priority": "high",
      "requires_human_approval": true
    }
  ],
  "agents": [
    {
      "id": "planner",
      "role": "planner",
      "status": "waiting_human",
      "current_task_id": "task-1"
    }
  ],
  "pending_human_actions": [
    {
      "id": "approve-plan-1",
      "kind": "approve_plan",
      "label": "Approve proposed plan"
    }
  ],
  "decisions": [],
  "blockers": []
}
```

### 13.3 状态枚举

`goal.status`：

```text
planning | active | paused | completed
```

`task.status`：

```text
planning | approved | doing | review | blocked | done
```

`agent.status`：

```text
idle | working | blocked | waiting_human
```

Agent role：

```text
planner | executor | reviewer | operator
```

Autonomy mode：

```text
manual | supervised | autonomous | paused
```

### 13.4 Mission Shared Actions

第一批共享 action：

```text
approve_plan
request_plan_changes
pause_mission
resume_mission
```

后续 task-level actions：

```text
reassign_task
change_priority
mark_blocked
request_review
approve_result
```

这些 action 改变 mission truth，必须通过 `org.octos.action_response`，由 Agent 产生新的 mission snapshot event。

### 13.5 Mission Local View Actions

Mission card 可以支持本地 action：

- 展开任务详情。
- 选择 agent。
- 切换 board/decision/blocker view。
- 过滤 lane。

这些 action 不应产生 Matrix 共享事实。

## 14. 安全模型

### 14.1 通用原则

Agent2View v0.1 安全原则：

1. Rich UI 只为注册 AppType 渲染。
2. 每个 AppType 校验自己的 state。
3. Robrix2 v1 template 必须本地静态。
4. 模板 preflight 必须拒绝未知 widget、未知 helper、未声明 state path。
5. 任意失败 fallback 到 plain text。
6. Action 必须按 AppType + action id 白名单分发。
7. Widget state 不得作为共享事实。

### 14.2 Aichat 安全边界

aichat 是本地 demo/runtime：

- 可以接收 LLM-generated `runsplash`。
- 可以使用 `agent.notify`。
- 必须限制 host action manifest。
- 必须校验 payload。
- 不应把任意 Agent 输出提升为系统权限。

### 14.3 Robrix2 安全边界

Robrix2 是 Matrix client：

- 不接受 event-supplied Splash template。
- 不接受 runtime-generated template 作为 v1 生产路径。
- 不用 `agent.notify` 表达共享 action。
- 不通过本地 reducer 静默修改 mission truth。
- 不读取 `m.replace` edit 来改变 app state 或 action set。

## 15. 版本与兼容

### 15.1 AppType Version

`version` 是 AppType 级协议版本。

宿主可以支持多个 version。

不支持的 version 必须 fallback。

### 15.2 字段兼容

新增可选字段时，旧宿主可以忽略。

改变既有字段语义时，必须提升 version。

### 15.3 State 兼容

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

## 16. 验证要求

### 16.1 Aichat

非 UI 验证：

```bash
cargo check -p makepad-example-aichat --release
cargo test -p makepad-example-aichat --release input
cargo test -p makepad-example-aichat --release collection
```

UI 验证必须通过 Studio remote release run，不以 raw `cargo run` 作为 UI 验证依据。

### 16.2 Robrix2

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

## 17. 非目标

本规范 v0.1 不包含：

- Agent2Project artifact manifest。
- LLM-generated template 在 Robrix2 生产路径中运行。
- 插件市场。
- 动态 capability discovery。
- 自动 repair loop。
- 多 Agent resolver/template-author 分工协议。
- 客户端持久化 room/account scoped state。
- 递归 subtask/project-management 全功能模型。

## 18. 后续扩展

可能的后续协议：

- Agent2Project artifact protocol。
- Robrix2 L2 action-capable cards。
- Robrix2 L3 stateful runtime。
- Generated template opt-in profile。
- Capability discovery protocol。
- Mission dashboard account scope profile。
- Multi-mission room routing rules。

这些扩展应各自有独立 spec，不应塞进 v0.1 Agent2View 核心协议。

