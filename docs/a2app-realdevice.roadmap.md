# A2App 落地路线图（R0–R3 + 标准化）

真机（HarmonyOS 6.1，AGC 开发者账号已就绪）为主线的完整落地计划。
需求对照见 `docs/a2app-requirements-gap.md`；本文件是执行顺序与状态的
唯一权威，随进度更新。

## 当前状态（更新于 2026-07-12）

已完成并提交（未推送时在此标注）：

| 提交 | 内容 |
|---|---|
| `c4fcd52f` | agent2app 端到端验证：OHOS aichat → octos → Moonshot（模拟器，日志验证） |
| `1e816b1e` | 需求 gap 分析 + 本路线图初版 |
| `4eeea7d7` | 绑定审查 P0：布局 diff 去每帧阻塞 JS 调用、HTTP headers 先行、ohos_sim 真机隔离（含 `tools/ohos_device_run.sh`） |
| `b5ac2287` | 绑定审查 P1：coalesce Create+Close 消除、kind 显式分发、ArkTS 未知 id trace |
| `665c43f8` | secure 密码输入端到端（契约+schema+widget+四平台；OHOS 完整实现） |
| `aa80e38c` | 修 OpenHarmony Studio 运行与 workspace PGO 配置冲突 |
| `dba4affe` | aichat 真机适配（octos backend、rawfile、OpenGL、移动布局、ArkTS host） |
| `140d4e1b` | 真机 EGL context、沙箱目录、移动布局检测修复 |

绑定方案三线审查（Rust NAPI / ArkTS host / 契约一致性）已完成，
P0+P1 全部修复；P2 结构债排入 R1。

本机已有 DevEco Studio、hdc 和 `aarch64-unknown-linux-ohos` target；此前的
环境阻塞已经解除。当前进程仍可能继承 `MAKEPAD=ohos_sim`，真机运行必须
由 `makepad.splash` 的 `RunOhosPackage` 覆盖为 `MAKEPAD=ohos`，或在命令行
辅助脚本前显式 `env -u MAKEPAD`。`init_cx_os` 仍保留启动告警兜底。

## R0 · 真机收口（真机首跑已通过，剩证据采集与文档收口）

**2026-07-12 真机验证通过**（用户确认）：两步人工操作（USB 授权 +
DevEco AGC 自动签名）已完成；经 `aa80e38c`/`dba4affe`/`140d4e1b` 三个
修复（Studio PGO、aichat 真机适配、EGL context/沙箱目录/移动布局）后，
应用在 HarmonyOS 6.1 真机上运行验证通过。

原两步人工操作步骤（留档备查）：① 手机 USB 连接 + 开发者模式/USB
调试并接受授权；② 分别用 DevEco 打开
`target/makepad-open-harmony/makepad_example_native_text_input`
和 `target/makepad-open-harmony/makepad_example_aichat`，登录华为账号，在
`File > Project Structure > Signing Configs` 选择自动签名并 Apply。DevEco
会把 AGC 材料写入各工程的 `build-profile.json5`，Hvigor 随后的
`assembleHap` 会生成真机签名的 `makepad-default-signed.hap`。

接线审查结论（2026-07-11）：`RunOhosPackage` 调用的
`cargo-makepad ohos run` 会先执行 `build()`，其 `build_hap()` 通过 Hvigor
运行 `assembleHap`；生成工程中的 `signingConfigs` 会在这一步生效。因此
Studio 证据脚本和 AGC 真机签名可以走同一条管道，无需放宽 P5 的
`Command:` 规则。OHOS `Device:` 检查已同步拒绝 `127.0.0.1:*`/localhost，
防止模拟器证据冒充真机证据。

剩余收口步骤（真机已可跑，缺的是逐项证据落盘）：

1. ~~构建、签名、安装、启动~~ **已通**。复跑方法：启动并复用 Studio
   remote bridge；先 `ListBuilds`，对同 target 的旧构建发
   `ClearBuild`，再以 release `RunItem` 启动
   `makepad-example-native-text-input-ohos` 或
   `makepad-example-aichat-ohos`。RunItem 会构建、AGC 签名、安装并启动。
   若覆盖安装报异签名错误，先用 hdc 卸载同 bundle id 的旧包再重跑。
2. **验收 4 件**：
   - aichat 自绘界面真实可见（模拟器上因 GLES3-on-Metal 黑屏，真机首验）
   - NativeTextInput 可点击、可输入
   - 中文 IME + 选区。**范围限定**：契约目前只有 text+selection，
     无 composition 镜像（macOS 也没做，见 R1 第 4 项）——本项只验
     拼音上屏后的文本回投 + 选区，不验组合中状态。
   - octos agent2app 真机端到端。aichat 主链路 `OCTOS_BASE_URL`
     已完全可配置（env 优先，退化到配置目录同名文件），真机只需设
     `http://<host LAN IP>:port`，无需改代码；硬编码 `10.0.2.2` 的
     只有调试工具（`tools/ohos_tcp_forward.py` 服务侧、
     `ohos_sse_test_server.py`、native_text_input SSE smoke test），
     用到时手动改。octos 绑 127.0.0.1 时保留
     `tools/ohos_tcp_forward.py`（已绑 0.0.0.0，LAN 场景可直接用）。
3. **P0-4 采证**（唯一遗留 P0）：首跑抓 LayoutTrace
   （`[MakepadNTI]` 的 `makepad_rect` vs `onAreaChange`），验证
   Makepad 逻辑单位 = ArkUI vp 的 1:1 假设；不成立则补换算。
   采集无自动化，手动 `hdc shell hilog | grep MakepadNTI` 落盘到
   `docs/native-textinput-evidence/`。判定标准：两组矩形逐项相差
   ≤1px（取整误差）即认定 1:1 成立。
4. 证据文档收口：用 `native_textinput_device_runtime_evidence.sh` 的 OHOS
   模式生成新的 `ohos-runtime.md` 和 Studio transcript，替换当前模拟器
   切片；补充 hilog LayoutTrace 后运行 completion audit 关闭 P5 gate。

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
   `examples/splash` 已在仓库，从未上机；无 OHOS 平台 cfg 排除，
   不会被编译门阻止。注意：此前文档引用的 `examples/isolate`
   路径在仓库中不存在，isolate 沙箱 example 尚待创建）。
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
- 真机 UI 构建和运行走 Studio `RunOhosPackage` release RunItem；
  `tools/ohos_device_run.sh` 只保留为非 UI 自动化/故障排查辅助。模拟器走
  `tools/ohos_sim_sign_run.sh`。
- 机械性、跨文件量大的工程（codegen、批量接线、静态检查同步）分配给
  低成本模型执行，架构决策与验收由主线把关。
