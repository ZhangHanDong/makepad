# Splash: 在 MAIN vm 里求值 runsplash app 导致持续重绘 / CPU 占满一核

**日期**: 2026-06-04
**组件**: `widgets/src/splash.rs`、`widgets/src/widget_async.rs`、`platform/script/std`
**状态**: 已定位根因；推荐基线 = 隔离 vm 求值（commit `8ca20bd2`）。本文为留档/可提上游。

---

## 现象

aichat 里加载 AI 生成的 `runsplash` app 后，进程持续占满约 1 个 CPU 核（UI 卡顿 / 鼠标转圈），即使是体积很小的 app。空会话时正常（release 约 10% CPU）。

实测对照：
- commit `8ca20bd2`（per-instance 作用域 ui，两个计算器可共存交互）—— **不卡**。
- 其后的 `67cc66fa`（"止血"提交）—— **卡**。

## 一句话根因

把 Splash 的求值从「每个 Splash 各自的**隔离 vm**」改成「共享的 **MAIN vm**」即触发持续重绘：

```rust
// 8ca20bd2（不卡）
cx.with_script_vm_id(self.vm_id, |vm| /* eval app body */)   // 隔离 vm，按需执行
// 67cc66fa（卡）
cx.with_vm(|vm| /* eval app body */)                          // MAIN vm，被宿主每帧驱动
```

`register_view_owners` + `widget_tree_mark_dirty` 在两个版本里**都有**，且 `8ca20bd2` 不卡 —— 因此它们**不是元凶**。

## 机制（已用只读探查验证的代码路径）

1. **MAIN vm 挂着每帧执行的 task-pump hook。**
   - 每个事件后 `cx.handle_script_tasks()` — `platform/src/os/cx_shared.rs:798`
   - while 循环，只要任一 hook 报告"有进展"就继续 — `platform/script/std/src/task.rs:109`
   - `pump_widget_async_hook`（`widgets/src/widget_async.rs:1013-1017`，由 `register_task_hooks` 在 `widget_async.rs:998-1001` 注册一次）

2. **app 在 MAIN vm 求值后，其 render/async 任务进入 MAIN vm 的任务队列，被每帧 pump。**
   - render 完成时 `View::script_result` 调 `self.redraw(...)` — `widgets/src/view.rs:710`
   - `redraw` 插入下一帧（`new_next_frames`），平台据此再请求一帧 — `platform/src/os/apple/macos/macos.rs:535`
   - → 每帧重绘 → 占满一核。用 `ui.<id>.render()` / `net.*` / 任意 widget async 的 app 尤其明显。

3. **隔离 vm 不被每帧驱动。**
   - `alloc_splash_vm` / `with_script_vm_id`（`widgets/src/widget_async.rs:168-245`）临时换入一套**独立 `ScriptStd`**，**未注册 pump hook**；其任务只在 `with_script_vm_id` 调用期间执行，宿主不会每帧 pump 它。

4. **附带因素**：`67cc66fa` 还去掉了 `with_instruction_limit(200_000)` 与隔离 vm 的 `run_budget`（64ms 软/硬，`widget_async.rs:224-228`）。MAIN vm 的 `run_budget` 本为 `None`（`platform/script/src/vm.rs:1314`），故 MAIN vm 上脚本（含 `call_fn`/`tick`）无预算上限。

## 复现

把 `widgets/src/splash.rs` 的 `eval_body` 求值从 `cx.with_script_vm_id(self.vm_id, …)` 改成 `cx.with_vm(…)`，加载任一带 `render`/async 的 runsplash app → CPU 占满一核。改回隔离 vm 即恢复正常。

## 结论 / 推荐

- **Splash app 体必须在每个 Splash 各自的隔离 vm 里求值**（`8ca20bd2` 的做法）。**绝不要**放到 MAIN vm。

## 相关独立问题：异步 `net.*` app 在隔离 vm 下闪退

`8ca20bd2`（隔离 vm）下，异步网络 app（如天气）会闪退：隔离 vm 有自己的 heap，但 HTTP 响应在 MAIN vm 上派发（`platform/script/std/src/net.rs:666` 的 `with_vm_and_async(host, std, script_vm, …)` → `vm.call(handler)`），于是隔离 heap 里捕获的 `on_response` 闭包被**跨 heap** 调用 → 崩溃。

**正确修法（不会重新引入 CPU 转圈）**：让异步 std 回调在**其所属的隔离 vm** 里恢复——
- 每个 `net.http_request` 记住发起它的 `SplashVmId`；
- 在 `platform/script/std/src/net.rs:659-690` 按该 `vm_id` 用 `cx.with_script_vm_id(owning_vm_id, …)` 派发 `on_response`/`on_error`，而非无条件用主 `script_vm`；
- 复用 `widgets/src/widget_async.rs` 既有的 `SplashVmId ↔ owner` 机制（`widget_owners` / `current_vm_id` / `with_script_vm_id`）。

属 platform 级改动，建议单独立项 + 运行时验证（计算器仍可交互 + 天气 app 不崩 + CPU 低）。
