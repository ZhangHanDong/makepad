spec: task
name: "CFP-lite Core S2b Aichat Integration"
tags: [cfp-lite, makepad, aichat, gate, review, s2b]
---

## 意图

完成 `docs/cfp-lite-core-split-plan.md` 的 S2b：让 makepad-dev
`aichat-agent2app` 固定依赖已通过 S2a 的 `cfp-lite-core` rev，删除本地
PermissionLevel、capability table、PendingConfirmation 与 review reducer，
改由 core 的 ExecutionIntent、gate、ReviewChain 和同一 14-case fixture
提供治理语义。Aichat 只保留 UI/legacy event adapter，迁移前可见行为不变。

## 约束

- `cfp-lite-core` git dependency 必须精确 pin rev `4f798f00c34a3ec1416ae41e549983c226b4cb38`
- 运行时 capability registry 与 manifest 必须从 core 的 `AICHAT_GATE_V1_JSON` 单一数据源加载，不得复制 action/permission table
- Aichat 不得重新定义 PermissionLevel、ExecutionIntent、PendingConfirmation、gate decision 或 review reducer
- 所有普通 Splash capability event 必须先构造 ExecutionIntent，再调用 `evaluate_gate`
- 所有 Propose/Approve/Reject 状态转移必须调用 `ReviewChain::apply`，adapter 不得直接改写 review status、sequence、ID 或 history
- `host.confirm` 无 active review 时保持 Refuse；`host.deny` 无 active review 时保持成功 `Discard(None)` no-op
- auto/readonly Dispatch 与 forbidden/unknown Refuse 不得清除或替换已有 active review
- Aichat 只绑定固定 transport Principal `transport:aichat` 与 Scope `app:aichat-agent2app`，不得从 notify payload 读取授权身份
- legacy raw payload cache 只用于保持可见字符串兼容，不得作为权限、review status、intent hash 或 dispatcher action 的权威来源
- 不得修改 prompt、UI DSL、dispatcher 业务分支、后台 agent、存储、渲染或 Makepad 平台代码
- 本阶段不得实现 S3 makepad-host 依赖切换或 S4 AuthorityStore/持久 ReviewChain

## 已定决策

- makepad-dev 基线: `dev@06e2d4ce860bde839f4798c8332a28e42a375104`
- 集成分支: `cfp-lite-core-s2-integration`
- Core dependency: `git = "https://github.com/ZhangHanDong/a2app-harness.git"`、`rev = "4f798f00c34a3ec1416ae41e549983c226b4cb38"`、`package = "cfp-lite-core"`
- `serde_json = "1"` 作为 aichat 直接依赖，用于运行时一次性解析 core 共享 profile 与测试 fixture
- `parse_gate_profile(&str) -> Result<GateProfile, String>` 是可注入纯解析 helper；runtime 通过 `OnceLock<Result<GateProfile, String>>` 缓存 core fixture profile，失败 fail closed，不 panic/fallback
- GateProfile 保留从 fixture 派生的 action name Vec，并用同一 Vec 构造私有 ActionRegistry；Vec 是可审计 derived data，不是第二份 table
- App 只持有一个 `SplashGateAdapter`；adapter 内部恰有 core ReviewChain 与 `BTreeMap<review_id, original_payload_json>` legacy display cache
- `SplashAdapterDecision` 精确为 `Dispatch { intent: ExecutionIntent, raw_payload: String }`、`Park { review_id: String, replaced_review_id: Option<String> }`、`Discard { review_id: Option<String> }`、`Refuse { code: ProtocolErrorCode, action: String, log: String }`
- Refuse code 映射固定为：profile parse failure → InvalidManifest，confirm 无 active → ReviewNotFound，raw cache 缺失/非法 payload → InvalidField，raw cache 语义不匹配 → ReviewBindingMismatch，其余使用 core Refuse code
- confirmation card 每次从 adapter active ReviewRecord 的 intent action 渲染，不复制 pending record
- Auto Dispatch 直接携带当前原始 payload；Park 后 cache 以新 review_id 保存原始 payload并删除 replaced ID
- Confirm 前必须从 cache 取原始 payload，重新构造 ExecutionIntent 并与 active record intent_hash 精确比较；缺失/不匹配则 Refuse且链不变，匹配后才 Approve并原样 Dispatch
- Reject 后删除对应 cache；终态/替换后 cache 不得保留失效 review_id
- 正常 runtime 每次 adapter event 后 raw cache key 集合必须精确等于 active_review_id 的零或一元素集合；只有 fault-injection test 可构造缺失/篡改 cache
- UnknownAction 与 ActionForbidden 都保持迁移前外部 Refuse；日志继续包含原 action 且不包含敏感 payload
- replacement 日志继续报告旧/新 action；same-intent 与 different-intent 都由 core 生成不同 review ID 与历史记录
- `test_aichat_uses_shared_gate_fixture` 必须读取 core 暴露的同一 fixture bytes，实际执行全部 14 case；两个 without-pending case 由 legacy adapter 执行
- fixture runner 必须直接遍历共享 JSON，不得复制 case ID、steps 或 expected projection table；只实现通用 op interpreter
- `syn = { version = "2", features = ["full", "visit"] }` 仅作为 dev-dependency，用 token/AST audit 本地重复治理定义与 App adapter 字段
- 完整外部门禁必须运行 `cargo test -p makepad-example-aichat-agent2app`，不得由测试内部递归调用 cargo/agent-spec
- S2 总阶段只有本合同 lifecycle、完整 aichat package tests、duplicate-definition audit 与独立 review 都通过后才完成

## 边界

### 允许修改
- examples/aichat-agent2app/Cargo.toml
- examples/aichat-agent2app/src/main.rs
- specs/cfp-lite-core-s2b-aichat-integration.spec
- docs/superpowers/plans/2026-07-13-cfp-lite-core-s2b.md

### 禁止修改
- Cargo.toml
- makepad.splash
- widgets/**
- platform/**
- draw/**
- examples/aichat-agent2app/resources/**
- examples/aichat-agent2app/src/app_ui.rs
- 不得修改非 gate/review adapter 的 prompt、UI、agent、storage 或 dispatcher 行为
- 不得 vendoring、复制或 path-depend `cfp-lite-core`

## 验收标准

场景: Aichat 精确 pin 已通过的 core rev
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_cfp_core_git_dependency_is_pinned
  Level: integration
  Targets: aichat Cargo.toml and compiled crate API
  假设读取 aichat Cargo.toml
  当检查 cfp-lite-core dependency
  那么 git URL、package 与 rev 精确等于已定决策
  并且依赖不是 branch、tag、path 或宽松版本

场景: 本地 gate 与 review 定义被删除
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_local_gate_review_definitions_are_removed
  Level: integration
  Targets: aichat source duplicate-definition audit
  假设用 syn AST 解析 `src/main.rs` 的生产 items 与 App/SplashGateAdapter 字段
  当检查本地治理定义与并行状态
  那么不存在 `enum PermissionLevel`、`struct PendingConfirmation`、`fn capability_permission`、`CAPABILITY_MANIFEST` 或本地 review status reducer
  并且 App 只有一个 SplashGateAdapter；adapter 只含 core ReviewChain 与按 review_id 键控的 legacy raw cache，并调用 core gate/review API

场景: PermissionLevel 词汇直接来自 core
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_permission_levels_match_cfp_vocabulary
  Level: unit
  假设测试导入 core 的 PermissionLevel
  当枚举四个 permission 值
  那么顺序与名称为 ReadOnly、RequiresConfirmation、AutoExecutable、Forbidden
  并且 main.rs 不含同名本地 enum

场景: AutoExecutable event 保持直接 Dispatch
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_auto_executable_event_dispatches
  Level: unit
  假设 ReviewChain 为空且 action 为 `inc`
  当 legacy adapter 处理 notify
  那么 core gate 返回 AutoExecutable Dispatch 并执行 count increment
  并且不创建 confirmation card 或 active review

场景: 九个既有 auto action 都经 core 原样 Dispatch
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_all_aichat_auto_actions_dispatch_via_core
  Level: unit
  假设输入 inc、dec、reset 与六个 timer action 及各自原始 payload
  当 adapter 逐项调用 core gate
  那么九项均返回 AutoExecutable Dispatch并保留原 action 与原始 payload 字符串
  并且没有一项创建或改变 active review

场景: Forbidden 与 unknown event 保持拒绝
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_forbidden_and_unknown_events_are_refused_by_core
  Level: unit
  假设分别输入 `fs.write` 与 registry 外 action
  当 adapter 调用 core gate
  那么两个结果均为 Refuse 且不 Dispatch
  并且错误码分别为 ActionForbidden 与 UnknownAction

场景: RequiresConfirmation event 进入 core ReviewChain
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_requires_confirmation_parks_event
  Level: unit
  假设 action 为 `ask_ai` 且链为空
  当 adapter 执行 gate 与 Propose
  那么不调用 dispatcher且链新增 Pending record
  并且 card 与 active review 只引用该 core record

场景: host.confirm 只释放绑定的原始 intent
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_host_confirm_releases_pending
  Level: unit
  假设 core ReviewChain 含一个 ask_ai Pending record
  当 adapter 处理 `host.confirm`
  那么 adapter 使用正确 review_id 与 intent_hash 调用 Approve
  并且 dispatcher 收到 record 内原始 action 与对应 cache raw payload，链进入 Approved且active/cache清空

场景: host.confirm 保留原始 payload 可见字节
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_host_confirm_preserves_original_payload_bytes
  Level: unit
  假设 ask_ai payload 含多键、嵌套对象、非 canonical key 顺序与额外空白
  当 Park 后处理 host.confirm
  那么 adapter 重算的 intent_hash 与 record 匹配后才 Approve
  并且 Dispatch 的 raw_payload 与输入字符串逐字节相同，canonical hash 仍绑定相同 JSON 语义

场景: host.confirm 拒绝缺失或篡改的 raw cache
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_confirm_refuses_missing_or_mismatched_payload_cache
  Level: unit
  假设 core ReviewChain 含 Pending record，但对应 raw cache 缺失或 JSON 语义被篡改
  当 adapter 处理 host.confirm
  那么两种输入均 Refuse 且不得调用 Approve 或 dispatcher
  并且 ReviewChain、active ID、history 与现有 cache 保持逐字节不变

场景: host.deny 丢弃 pending 且不执行
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_host_deny_discards_pending
  Level: unit
  假设 core ReviewChain 含一个 ask_ai Pending record
  当 adapter 处理 `host.deny`
  那么 adapter 调用 Reject且不 Dispatch
  并且链进入 Rejected、active 清空、外部结果为 Discard(Some)

场景: 无 pending 的 confirm 与 deny 保持非对称兼容
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_confirm_and_deny_without_pending_preserve_legacy_behavior
  Level: unit
  假设 ReviewChain 为空
  当分别处理 host.confirm 与 host.deny
  那么 confirm 为 Refuse
  并且 deny 为成功 Discard(None) no-op，二者均不伪造 core review ID

场景: 第二个 pending 使用 core replacement history
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_second_pending_replaces_first
  Level: unit
  假设链分别处于 Pending 或 Deferred
  当对每种状态分别提交 same-intent 与 different-intent 的新 ask_ai
  那么旧 record 为 Withdrawn、新 record 为 Pending且 active 指向新 ID
  并且四种组合的旧 raw cache 被删除、新 ID cache 保存对应原始 payload，历史未被 adapter 重写

场景: 非 review 决策不改变 active review
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_gate_outcomes_preserve_active_review
  Level: unit
  假设链已有 Pending review
  当依次处理 auto Dispatch、synthetic ReadOnly Dispatch、forbidden Refuse 与 unknown Refuse
  那么每次处理后 active review 与完整 history 字节均不变
  并且 auto/readonly 被 Dispatch，两个 Refuse 不执行；synthetic ReadOnly 不进入 runtime cached profile

场景: 非法 profile 与 payload fail closed
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_invalid_profile_and_payload_fail_closed_preserving_review
  Level: unit
  假设 `parse_gate_profile` 收到 malformed/缺字段/重复 registry profile，或 adapter 收到 malformed payload JSON
  当分别解析 profile 或在已有 Pending review 时处理 payload
  那么 profile 返回 Err，runtime path 映射为 Refuse且不 panic/fallback；payload 返回 InvalidField Refuse
  并且原 ReviewChain、active ID、history 与 raw cache 逐字节不变

场景: transport authority 固定且 payload 无法覆盖
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_transport_identity_is_fixed_and_payload_cannot_override_authority
  Level: unit
  假设 payload 声明伪造 principal、scope 与 app_id
  当 adapter Park 该 ask_ai intent
  那么 ReviewRecord Principal 仍精确为 `transport:aichat`
  并且 Scope 仍精确为 `app:aichat-agent2app`，伪造字段只属于被哈希 payload

场景: 运行时 profile 与共享 fixture 根数据一致
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_aichat_profile_comes_from_shared_fixture
  Level: integration
  Targets: cached GateProfile and shared fixture bytes
  假设解析 core 的 AICHAT_GATE_V1_JSON
  当构造运行时 registry 与 manifest
  那么 GateProfile 派生 action name Vec 精确为 fixture 的 13 个具体 action，并用同一 Vec 构造 private registry；manifest 精确为 8 个 pattern/permission
  并且 fixture hash 等于 `af8cb79b560c6dceee75502b966494aa7216bc26bfafd1e45c94b5a82c727cc0`

场景: Aichat 实际执行共享 fixture 全部 case
  测试:
    包: makepad-example-aichat-agent2app
    过滤: test_aichat_uses_shared_gate_fixture
  Level: integration
  Targets: core fixture and legacy adapter
  假设读取共享 fixture 的固定 14 个 case IDs 与 steps
  当 aichat adapter 逐项执行 12 个 core cases 与 2 个 without-pending cases
  那么 decision、active review、history、dispatch 与 adapter outcome 均符合 fixture expected projection
  并且测试没有复制第二份 fixture JSON、case ID/steps/expected table 或本地 permission table，只含通用 op runner

## 排除范围

- aichat prompt、UI、模型 provider、diagram、storage、window 或 renderer 改动
- S3 makepad-host 切换 fork rev 与 agent.notify smoke
- S4 manifest hash 重审、reviewer authority、持久化、expiry、并发决定与 execution ledger
- OHOS 单进程 profile
