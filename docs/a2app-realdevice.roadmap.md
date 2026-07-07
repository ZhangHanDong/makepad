# A2App 落地路线图（R0–R3 + 标准化）

真机（HarmonyOS 6.1，AGC 开发者签名已就绪）为主线的完整落地计划。
需求对照见 `docs/a2app-requirements-gap.md`；本文件是执行顺序与状态的
唯一权威，随进度更新。

## 当前状态（更新于 2026-06-17）

已完成并提交（未推送时在此标注）：

| 提交 | 内容 |
|---|---|
| `c4fcd52f` | agent2app 端到端验证：OHOS aichat → octos → Moonshot（模拟器，日志验证） |
| `1e816b1e` | 需求 gap 分析 + 本路线图初版 |
| `4eeea7d7` | 绑定审查 P0：布局 diff 去每帧阻塞 JS 调用、HTTP headers 先行、ohos_sim 真机隔离（含 `tools/ohos_device_run.sh`） |
| `b5ac2287` | 绑定审查 P1：coalesce Create+Close 消除、kind 显式分发、ArkTS 未知 id trace |
| `665c43f8` | secure 密码输入端到端（契约+schema+widget+四平台；OHOS 完整实现） |

绑定方案三线审查（Rust NAPI / ArkTS host / 契约一致性）已完成，
P0+P1 全部修复；P2 结构债排入 R1。

环境注意：本机 `~/.zshrc` 曾全局 `export MAKEPAD=ohos_sim`（已注释）。
真机构建必须无此 flag；`tools/ohos_device_run.sh` 用 `env -u MAKEPAD`
强制隔离并在检测到时拒绝运行，`init_cx_os` 亦有启动告警兜底。

## R0 · 真机收口（当前阻塞点：两步人工操作）

等待用户：① 手机 USB 连接 + 开发者模式/USB 调试 ② DevEco 打开生成
工程登录华为账号做一次 AGC 自动签名（注册设备 UDID、生成 .p12/.cer/.p7b）。

然后：

1. `tools/ohos_device_run.sh -p <crate>` 构建（正常 shader 路径）、
   签名、安装、启动。
2. **验收 4 件**：
   - aichat 自绘界面真实可见（模拟器上因 GLES3-on-Metal 黑屏，真机首验）
   - NativeTextInput 可点击、可输入
   - 中文 IME + 选区
   - octos agent2app 真机端到端（octos 寻址从 `10.0.2.2` 改 host LAN IP；
     octos 绑 127.0.0.1 时保留 `tools/ohos_tcp_forward.py`）
3. **P0-4 采证**（唯一遗留 P0）：首跑抓 LayoutTrace
   （`[MakepadNTI]` 的 `makepad_rect` vs `onAreaChange`），验证
   Makepad 逻辑单位 = ArkUI vp 的 1:1 假设；不成立则补换算。
4. 证据文档收口：completion audit 目前标 `ohos-runtime.md` incomplete；
   真机证据落盘后关闭 P5 gate。

## R1 · 包装层成体系（R0 后；与 R2 可部分并行）

审查确认的结构债（P2）全部落在这里，**顺序有讲究**：

1. **schema codegen 先行**：把 `platform/src/native_host_schema.rs` 从
   一致性校验器变成生成器（Rust enums / ArkTS 方法桩 / ObjC/Java 桥）。
   这是消除多平台漂移的最大单点，必须在加第二个组件 kind 之前完成，
   否则每个新 kind 都在手写四平台胶水。
2. **泛化注册表**：OHOS 侧用 `createNativeView(id, kind, propsJson)` 的
   通用 NAPI 面替掉字符串方法名分发（对齐 macOS `dyn MacosNativeHost`
   trait 注册表）；ArkTS 侧收敛两套平行的 state-array + ForEach 为共享
   host 抽象。
3. **裁剪 / z-order 合成层**：macOS `clipped_native_host_layout` /
   `order_native_host_view` 的 OHOS 对应物。Map/List 等可滚动组件的
   前置条件，也是 deck「可见区域合成」缺口②。
4. **选区 + IME composition 镜像**：deck S8 完整 `full_state_sync` 契约。
   注意 NativeTextInput 的契约目前只有 text+selection，composition
   **macOS 也没做**，两平台一起补。
5. 低优先记档项：macOS secure 运行时切换（cell class 重建）、
   iOS/Android secure 真实现、napi_ref 释放路径、OHOS WebSocket backend
   （octos 走 SSE 已够，Splash 双向通道需要时再做）。

codegen（第 1 项）是机械工程，适合交给低成本模型执行。

## R2 · Splash 上机 + 生成式 UI 最小闭环（deck 的灵魂）

从「证明了管道」到「证明了 deck 论点」的一步，前面全部工作都是铺路：

1. Splash VM / isolate 搬上 OHOS（`widgets/src/splash.rs` /
   `examples/isolate` 已在仓库，从未上机）。
2. 能力闸门 + 类型化审批（S11 容器：指令预算 20 万/次、trap 抢占、
   支付/网络/文件/设备白名单、高风险操作回到用户确认）。
3. **流式生成闭环**：Agent 输出 Splash DSL → 增量 parse → 骨架渲染 →
   增量填充 → patch 循环（生成未完成即可交互）。
4. **Demo 一 Trip Planner 最小版**：Agent 生成的界面里跑真 ArkUI 原生
   组件——先用 Text/Button/List + 一张自绘玻璃卡片，不上地图/支付，
   把「树内运行真原生组件」的命题证明掉。
5. 玻璃设计系统与 `examples/aichat/specs/aichat-apple-glass-shader-plan.md`
   的视觉改造归入本阶段。

## R3 · Demo 达标 + 指标采集（真机才可测）

- Demo 一指标：首字节→首次可交互 ≤1.5s、整页 patch ≤5s、原生组件 ≥2、
  能力白名单全过闸、交互失败率 ≤0.1%。
- Demo 二（健康仪表盘）：60Hz、单帧 GPU ≤12ms、多 pass 管线
  (capture/blur/SDF/composite/motion)、包体 ≤30MB、冷启 ≤800ms、
  native IME、暗/亮切换 0 帧损。
- 固定 fixture + 自动采集，跨运行可比。

## R3 之后 · 标准化（deck M6–M12）

跑起来的手机变成标准：AppUI v1 冻结（turn / context / tool / ledger
API 表面）→ A2App 标准草案 → conformance 测试套件 → 国产模型 Splash
生成 benchmark → 公开演示。到 R3 完成前不展开细节。

## 执行原则

- 每阶段收口以证据文档为准（`docs/native-textinput-evidence/` +
  `tools/native_textinput_completion_audit.sh`），不以"代码写完"为准。
- 真机构建永远走 `tools/ohos_device_run.sh`（ohos_sim 隔离）；模拟器
  走 `tools/ohos_sim_sign_run.sh`。
- 机械性、跨文件量大的工程（codegen、批量接线、静态检查同步）分配给
  低成本模型执行，架构决策与验收由主线把关。
