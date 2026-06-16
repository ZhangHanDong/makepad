# A2App 最终需求 vs 当前实现 · Gap 分析

来源：`AgentOS_A2App_Plan.pptx`（Internal · 2026-06，18 页）。本文把 deck
的终局需求拆成可核对项，并标注当前分支
（`native-textinput-macos-poc`，截至 commit `c4fcd52f`）的落地状态。

## 一、deck 的终局愿景

标题 **AgentOS · A2App** —— "Agent 原生应用运行时 · OpenHarmony 手机端到端
验证"。要立的是一个**标准 + 参考实现**，对标 Google A2UI（标准层）+ Flutter
GenUI（实现层）的飞轮，去填"设备原生 A2App"这个对位空缺。

核心论点三层：

1. **形态反转 · 任务即应用（S2）**
   传统：需求 → 商店查找 → 安装 → 打开 App → 固定 UI 操作（人适配应用）。
   A2App：**一句意图 → Agent 规划 → 流式生成 UI → 交互 / 确认**（应用即时
   生成、应用适配人）。A2App 是 TUI / Web UI / Messaging 之后的第四形态，
   载体是"设备原生 runtime"，输出是"可交互应用"——前三种渲染对话，只有
   A2App 渲染应用。

2. **全栈架构（S5）**，自上而下：
   ```
   生成的 Agent App · 屏幕上的流式卡片
   A2App 运行时 · Makepad 渲染器 + ArkUI 包装层 + Splash VM
   AppUI · 面向应用的 AgentOS API
   AgentOS 内核 · LLM(端侧+远程路由) / Context·Memory / Tools·Sandbox / Harness
   OpenHarmony 手机 · 硬件 · ArkUI · 原生服务
   ```

3. **关键技术契约**
   - 原生为默认、自绘为探索（S7）：ArkUI 页面 + XComponent 宿主；列表/按钮/
     地图/TextInput 用 ArkUI 原生组件，液态玻璃/shader 才用 Makepad 自绘。
     两者经 **NAPI 桥 + `full_state_sync`** 同步。
   - 包装模式已被 iOS 验证（S8）：原生 UITextView 隐形、Makepad 自绘视觉，
     状态镜像（文本+选区+IME）经 `full_state_sync`。本提案把同一模式移植到
     ArkUI。
   - Splash · 一种语言编排两类组件（S9）：Splash script → Script VM → 既驱动
     Makepad 自绘 widget，也经 ArkUI wrapper 直调原生组件，**无 JS bridge
     热路径**。
   - A2App 容器 = Splash Isolate 沙箱（S11）：每 A2App 独立 Splash VM、指令
     预算 20 万/次、trap 抢占、能力闸门白名单（支付/网络/文件/设备）、类型化
     审批回到用户确认、Rust 内存安全。增量生成可交互：DSL 流入 → 增量 parse
     → 骨架渲染 → 增量填充 → patch 循环。

4. **里程碑（S17）**
   - M0–M3 底座：OHOS 构建/运行/调试稳定 + Splash isolate 上机
   - M3–M6 组合：ArkUI 包装层（NAPI state-sync）/ 可见区域合成 / Splash 绑定 /
     玻璃设计系统
   - M6–M9：两 Demo 达标 + AppUI v1 冻结 + A2App 标准草案
   - M9–M12：conformance 套件 + 国产模型 benchmark + 公开演示

5. **Demo 验收（S15/S16）**
   - Demo 一 · 流式 Trip Planner：树内运行真原生 ArkUI 组件（地图/支付面板）；
     首次可交互 ≤1.5s、整页 patch ≤5s、原生组件 ≥2、能力白名单全过闸、交互
     失败率 ≤0.1%。
   - Demo 二 · 健康仪表盘：60Hz、单帧 GPU ≤12ms、多 pass 管线
     (capture/blur/SDF/composite/motion)、包体 ≤30MB、冷启 ≤800ms、native IME。

## 二、逐项对照

| deck 要求 | 里程碑 | 当前状态 | 评估 |
|---|---|---|---|
| OHOS 构建/运行/调试稳定 | M0–M3 | HAP 签名+安装+启动跑通；NAPI 桥、事件循环、日志全通 | **达到**（自绘受模拟器 GPU 限制，真机可解） |
| ArkUI 原生组件为默认、XComponent 宿主 | M3–M6 | NativeTextInput + NativeLabel 经 ArkUI TextInput/Text 渲染，`@Observed`/`@ObjectLink` 状态同步 | **首个组件达到** |
| 包装层 `full_state_sync`（iOS 模式移植 ArkUI） | M3–M6① | typed NAPI（ArkTsObjRef）+ text/programmatic 回投 + revision guard；只覆盖 TextInput，选区/IME 镜像未做 | **起步，未成体系** |
| 可见区域合成 | M3–M6② | rect/layout 更新到 ArkUI；自绘层与原生层合成未做 | **部分** |
| OHOS 网络（Agent 路由打通） | 桥能力 | OHOS NetworkBackend（HTTP+SSE，requestInStream）跨 NAPI 跑通 | **达到，超出 deck 既有清单** |
| Agent 作为应用后端（thin client 拓扑） | 贯穿 | aichat → octos agent host → Moonshot，data-only SSE replace/token/done 流式回渲 | **拓扑达到** |
| **Splash DSL 流式生成 UI（核心命题）** | M3–M6③ / M6–M9 | aichat 是手写应用；无 Agent 生成 widget subtree、无 Splash VM 上机、无增量 parse→骨架→填充→patch | **未做（核心 gap）** |
| Splash Isolate 沙箱（预算/trap/闸门/审批） | M3–M6 | 仓库有 `splash.rs`/`examples/isolate`，未在 OHOS 上机 | **未做** |
| 玻璃设计系统 / 多 pass shader 管线 | M3–M6 | 模拟器 GLES3-on-Metal 自绘起不来；玻璃/blur/SDF 未做 | **未做（真机解锁）** |
| Demo 一 / Demo 二 + 指标 | M6–M9 | 无 | **未做** |
| AppUI v1（turn/context/tool/ledger API 表面） | M6–M9 | 用 makepad_ai 的 AiBackend，非 deck 定义的 AppUI 表面 | **未做** |
| A2App 规范 + conformance + benchmark | M9–M12 | 无 | **未做** |

## 三、结论

**一句话：扎实完成了 M0–M3「底座」的大部分，打通了 M3–M6 的第一根桩
（ArkUI 包装层首组件 + OHOS 网络），但 A2App 的核心命题「Splash 流式生成 UI」
还完全没碰。**

真正达到 deck 要求的：
1. OHOS 桥三块基础设施（ArkTS / NAPI / Rust）—— 并补上 deck 清单里没有的
   OHOS 网络栈（HTTP/SSE 跨 NAPI），这是 Agent 路由上机的前提。
2. 包装模式从 iOS 移植到 ArkUI 的可行性证明（NativeTextInput/NativeLabel
   typed NAPI + 状态镜像最小闭环）—— 证明 S14 缺口① 路径成立。
3. Agent-as-backend 拓扑（octos agent2app 的会话/线程/流式数据通路）。

与终局最大差距（按重要性）：
1. **生成式 UI 整条链路缺失**。deck 的灵魂（S2 流式生成 UI）没落地：aichat
   是人手写死的界面，Agent 只往里流文本，没有生成界面。octos 链路证明了
   管道，没证明 deck 的核心论点。
2. **Splash VM / Isolate 没上 OHOS**（指令预算/trap/能力闸门/类型化审批一行
   没在手机上跑）。
3. **玻璃/自绘视觉在模拟器上是死的**（已确认 GPU 限制，需真机）。

用 deck 里程碑标尺：M0–M3 接近完成、M3–M6 完成约 1/3（包装层首组件 + 网络，
缺可见区域合成、Splash 绑定、玻璃系统），M6 之后（Demo/AppUI/标准）尚未开始。

落地路径见 `docs/a2app-realdevice.roadmap.md`。
