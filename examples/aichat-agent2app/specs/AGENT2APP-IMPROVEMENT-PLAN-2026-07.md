# Agent2App Improvement Plan — July 2026 (post upstream merge)

Status: active. Owner: AlexZ. Implementer: Codex (via agent-spec contracts).
Source review: Claude session 2026-07-07, based on merge commit `7fece45d`.

## Context (what the review found)

1. The agent2app core channel (`SplashAction::Notify` + `register_agent_module`
   in `widgets/src/splash.rs`, host dispatch `handle_splash_event` in
   `examples/aichat-agent2app/src/main.rs`) survived the upstream merge intact and is
   **fork-exclusive** — upstream has no equivalent.
2. **Critical gap**: per-Splash isolated VMs (`cx.alloc_splash_vm()` →
   `makepad_widgets::script_mod`) never called `register_agent_module`, so
   `agent` was not injected into the VMs interactive Splash instances actually
   run in. An uncommitted one-liner in `widgets/src/lib.rs` (`script_mod` now
   calls `crate::splash::register_agent_module(vm)`) fixes it. aichat's
   explicit registration (`main.rs` ~3331) becomes redundant.
3. Upstream independently shipped the design we abandoned in 2026-06:
   per-instance isolated VMs + subtree-scoped `ui.<id>` (`ac10a82c`), Drop +
   `gc_dead_splash_isolates` lifecycle (`3a82d260`), and async requests that
   carry `req.vm_id` through `pump_widget_async` (fixes the cross-heap crash).
   This unlocks: multiple concurrent interactive generated apps, and generated
   apps that use `net.http_request`.
4. The uncommitted Animator-per-VM refactor (`derive_animator.rs`,
   `animator.rs`, `extern crate self as makepad_widgets`) breaks
   `cargo check -p makepad-widgets` (16× E0782). HEAD compiles clean.
5. `splash.md` (lines ~441-447) still claims `ui` is only visible inside
   `on_click`/`on_return`/`on_change` closures — stale after `ac10a82c`.
6. Two parallel agent2app channels exist: aichat's typed
   `agent.notify(event_id, json_payload)` vs `tools/canvas`'s untyped
   button-name broadcast over WS/HTTP. No shared envelope.
7. CFP v0.3 (`cfp v0.3.md`) is adopted **as vocabulary only** at this stage:
   L4 `PermissionLevel` for the capability manifest (P3), AppBundle-subset
   metadata for the AppGen workspace (P4). No CFP runtime, no seven layers.

## Phases

### P0 — Git hygiene + registration fix (human + Codex)

Manual git steps (AlexZ, before Codex starts):

1. Move the in-progress Animator-per-VM refactor off the working tree:
   `git stash` or commit `widgets/derive_widget/src/derive_animator.rs`,
   `widgets/src/animator.rs`, and the `extern crate self` /
   `CxSplashVmExt` lines of `widgets/src/lib.rs` to a branch
   `animator-per-vm-wip`. The working tree must compile before any spec work.
2. Keep on dev: the `register_agent_module(vm)` line in `lib.rs::script_mod`,
   the `resolves_fn` tick fix in `splash.rs`, and the fmt-only files
   (commit fmt separately or discard).

Codex contract: `agent2app-p0-isolated-vm-agent-registration.spec.md`

### P1 — Regression proof + prompt unlock (Codex + human acceptance)

Codex contract: `agent2app-p1-multi-instance-regression.spec.md`

Manual acceptance checklist (run after Codex finishes, release build:
`env -u ANTHROPIC_API_KEY cargo run --release -p makepad-example-aichat-agent2app`):

- [ ] Ask for two calculators in one conversation; both stay interactive,
      no `widget 'display' not found`, no CPU spin.
- [ ] Ask for a weather app using `net.http_request`; response callback does
      not crash (historical cross-heap crash, memory: 67cc66fa).
- [ ] `agent.notify` works from a button inside a generated app
      (was silently broken on isolated VMs before P0).
- [ ] Kill/regenerate an app several times; no VM leak symptoms
      (gc_dead_splash_isolates path).

### P2 — Unify tools/canvas onto the agent.notify envelope (Codex)

Codex contract: `agent2app-p2-canvas-notify-unification.spec.md`

**CFP-lite alignment (2026-07-12,来源见下方 §CFP-lite)**:

- 统一信封**不要自己发明**:直接采用 a2app-harness 演进计划的
  `cfp-lite/0.1` envelope(protocol / event_id / request_id /
  parent_event_id / app_id / claimed_actor / action / payload /
  created_at)。aichat、canvas、harness 三方共用一个信封,将来互操作
  零翻译。
- **命名冲突必须在本阶段修正**:当前 `agent.notify(event_id, payload)`
  里的 "event_id" 语义上是 CFP-lite 的 **action**(动作名 "inc");
  CFP-lite 的 event_id 是**事件实例身份**(ULID)。统一信封时把参数
  改名为 action,同时补事件实例 ID——现在只是语义混淆,加事件日志后
  就是真 bug。

### P3 — Capability manifest with CFP permission levels (Codex)

Codex contract: `agent2app-p3-capability-permission-levels.spec.md`

**CFP-lite alignment(闸门增强三件套,可并入本契约或另立 P3.5)**:

1. **ExecutionIntent 固化 + 参数完整性哈希**:审批时把
   action + canonical payload(+ manifest 内容哈希)绑定进
   `PendingConfirmation`,approve 时复核哈希,防"审查 A、执行 B"。
   生成代码是不可信输入,pending 期间的状态漂移必须防。
2. **两级默认值**:区分"registry 外非法输入 → 拒绝"与"合法建模但未列
   权限 → 默认 RequiresConfirmation"。当前一刀切 unknown→Forbidden,
   引入中间档后新动作可"先能用但要确认",不必每次改 manifest。
3. **manifest 内容哈希**:确认卡与执行记录携带 manifest 版本哈希,
   pending 期间 manifest 变化必须重新过闸。

### P4 — AppBundle-subset metadata for AppGen workspace (spec deferred)

Persist per generated app: `user_intent`, `decisions[]`, `commitments[]`,
splash source, `parent_version` chain — the minimal CFP AppBundle subset that
enables incremental modification instead of regeneration. **Do not write this
spec until P2/P3 land**; the storage shape depends on the unified envelope and
the manifest format.

**CFP-lite alignment**:schema **与 a2app-harness 计划的 P5 AppBundle 共享
定义**(字段清单本就一致:bundle_id/version/parent_version、user_intent、
claims/decisions/commitments、splash source + 内容哈希、capability
manifest、runtime binding、provenance),不要做两套。采纳其两条规则:
① bundle-scoped manifest **只能收紧宿主内置上限,不能放宽**——这同时回答
了 R2"原生组件(NativeXxx kind)实例化是否进白名单"的设计:appplan 声明
用哪些 kind,宿主上限裁决;② 大对象(splash source、模型输出)走内容哈希
引用,不进事件记录本体。

## CFP-lite 对齐来源与分工边界(2026-07-12)

来源:`a2app-harness` 仓库 `docs/cfp-crdt-evolution-plan` 分支
`docs/cfp-crdt-evolution-plan.md`(基线 main@f6f14b2,其 §3 引用本项目
P3 作为参考实现)。

**分工边界(避免重复建设)**:该计划已明确——本项目是"单进程 UI 交互
治理样例",harness 是"三进程运行系统"。因此本项目**不自建** AuthorityStore、
持久 ReviewChain、事件溯源、transport principal;唯一约束是闸门逻辑保持
纯函数、与 UI 解耦(现状已满足),便于将来上移到 harness 式权威边界。
流式 token 不入持久日志的原则同样适用于 R2 流式生成闭环的日志设计。

## Spec index & dependencies

| Spec | Depends | Estimate |
|---|---|---|
| agent2app-p0-isolated-vm-agent-registration | — | 0.5d |
| agent2app-p1-multi-instance-regression | p0 | 1d |
| agent2app-p2-canvas-notify-unification | p0 | 1d |
| agent2app-p3-capability-permission-levels | p0, p1 | 2d |

Graph: `agent-spec graph --spec-dir examples/aichat-agent2app/specs`

## Codex operating notes

- Verify each contract with `agent-spec verify <spec> --code .` before handing
  back; boundaries are mechanically enforced.
- Headless ScriptVm test precedent: `platform/script/std/tests/headless.rs`.
  Widget-tree test precedent: `#[cfg(test)]` mods in `widgets/src/widget_tree.rs`.
- Never move `agent.notify`/`SplashAction` below the widgets crate —
  `makepad-script` must not depend on `Cx::post_action` (crate boundary rule
  from AGENT-2-APP-DESIGIN.md, still binding).
- Run everything with `env -u ANTHROPIC_API_KEY` (stale API key causes 401s
  against subscription auth).
