# 目标与边界

Agent2App 的核心目标是定义 Agent 如何把结构化应用视图交给宿主渲染，以及用户在视图中的操作如何回到宿主或 Agent。

本规范当前只覆盖 `agent2view`。

## 核心目标

`agent2view` 需要解决六个问题：

1. Agent 如何声明要渲染的 app view。
2. Host 如何区分业务 data、运行 state 和渲染 view。
3. Host 如何校验 app type、data、state path、template 与 action。
4. Host 如何保存运行状态，避免 widget 销毁后丢失本地交互上下文。
5. 用户 action 如何回到 Host 或 Agent，并保持共享事实边界清晰。
6. 同一套协议语义如何同时适配本地 Agent 和远程 Agent。

## 分层边界

```mermaid
flowchart TB
    Core[Protocol Core Kernel]
    Core --> Obj[Object model]
    Core --> Data[Data model]
    Core --> State[State model]
    Core --> View[View model]
    Core --> Act[Action model]
    Core --> Val[Validation model]

    Core --> Bind[Transport Binding]
    Bind --> Local[Local response / in-process callback]
    Bind --> Remote[Remote event / action response]

    Core --> Profile[Application Profile]
    Profile --> A[aichat]
    Profile --> R[Robrix2]
    Profile --> M[Mission Room]
```

协议内核不关心 Agent 是本地还是远程，也不关心 transport 是 LLM response、Matrix event、HTTP、WebSocket 还是未来的其它事件系统。内核只定义稳定语义：AppType、AppInstance、Scope、DataSnapshot、Host State、ViewSpec、Template、Action、ActionResult、Capability 与 Validation。

Transport binding 负责把这些语义映射到具体承载格式。例如 aichat 把 view 放在 assistant message 的 `runsplash` block 中；Robrix2 把 view 放在 Matrix event 的 `org.octos.app` envelope 中。

Application profile 负责定义具体宿主的能力、限制和 UX 策略。aichat 和 Robrix2 是 profile，不是两套互斥协议。

## 两条产品轨道

```mermaid
flowchart TB
    U[User request or external event] --> A{Agent2App track}
    A -->|inline view| V[agent2view]
    A -->|generated project| P[agent2project]

    V --> VH[Host renders app view]
    V --> VS[Host owns data/state/action boundary]

    P --> PF[File tree]
    P --> PC[Cargo.toml / src/main.rs]
    P --> PR[Studio runnable / package metadata]
```

## Agent2View

`agent2view` 表示 Agent 输出的内容最终成为宿主内的 live view。它不是独立项目，也不拥有自己的完整进程、文件树或打包生命周期。

它适合：

- aichat inline `runsplash` demo。
- Robrix2 weather/news card。
- Robrix2 mission room control surface。

## Agent2Project

`agent2project` 表示 Agent 生成独立 Makepad 项目或应用产物。

它至少需要定义：

- 文件树。
- `Cargo.toml`。
- `src/main.rs`。
- 资源文件。
- 状态模块。
- 事件/action handler。
- Studio runnable item。
- 导出和打包元数据。

Agent2Project 不得复用 Agent2View 的 runtime capability manifest。前者是构建产物合同，后者是宿主运行能力边界。

## 本版非边界

本版不定义：

- 代码生成项目协议。
- 插件市场。
- LLM 生成模板在 Robrix2 生产路径的执行协议。
- 多 Agent resolver/template-author 分工。
- 自动 repair loop。
