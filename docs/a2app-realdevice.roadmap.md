# A2App 真机落地路线（R0–R3）

前提：放弃模拟器（GLES3-on-Metal 自绘起不来的限制不可解），改在 **OHOS 真机**
推进。设备 + 开发者签名已就绪。

为什么换真机是关键判断：之前压着进度的几块（玻璃/自绘、可见区域合成、IME、
Demo 指标）**不是没实现，而是被这台 Mac + 模拟器的 GPU 限制挡住**。真机一上，
deck 标"未做"的项里有一半从"无法验证"变成"可推进"。

对照 deck 里程碑见 `docs/a2app-requirements-gap.md`。

## 真机相比模拟器要改的

1. **签名**：模拟器用 SDK community 自签 + UDID 重写；真机走开发者证书
   sideload（`hdc` over USB，不再 `tconn 127.0.0.1:5555`）。
2. **连接**：`hdc list targets` 认物理设备（USB）。
3. **网络**：真机与 host 同一 LAN，octos 直接用 host LAN IP 可达——不再需要
   `10.0.2.2` + `tools/ohos_tcp_forward.py` 这套寻址 hack（octos 仍绑
   127.0.0.1 时，forwarder 转发到 0.0.0.0 仍保留，但寻址简化）。
4. **配置投递**：`rawfile` + `Cx::read_ohos_rawfile` 与模拟器完全一致，不动。
5. **GPU 容错**：真机不需要 `ohos_sim` 的 `GlShaderState::Failed` 容错，走正常
   shader 编译路径——这本身就是"自绘上机"的验收。

## R0 · 现有成果搬上真机 + 肉眼收口

目标：把"日志验证"升级成"视觉 + 交互验证"，正式收口 deck 的 M0–M3 底座。

- 去掉 `MAKEPAD=ohos_sim` 容错，走正常 shader 路径（真机 GPU 应能编过）。
- 签名/安装脚本改真机路径（`ohos_sim_sign_run.sh` → 真机变体）。
- octos 寻址改 host LAN IP。
- **验收 4 件**：
  1. aichat 自绘界面真实可见
  2. NativeTextInput host 可点击/输入
  3. 中文 IME + 选区镜像
  4. octos agent2app 真机端到端跑通（这次能看见流式文字进 UI）

证据：`docs/native-textinput-evidence/` 下补真机 runtime 记录 + 截图。

## R1 · 包装层 state-sync 成体系（M3–M6 ①②）

- `full_state_sync` 从"只有 text"补到 **选区 + IME 状态镜像**（deck S8 完整契约）。
- **可见区域合成**：自绘层与 ArkUI 原生层的 z-order / 裁剪（真机能渲染后才有
  意义）。

## R2 · Splash 上机 + 生成式 UI 最小闭环 ← 核心 gap

deck 的灵魂（S2 流式生成 UI）在这里第一次落地：

- Splash VM / isolate 上 OHOS（仓库已有 `widgets/src/splash.rs` /
  `examples/isolate`，搬上机 + 接能力闸门）。
- Agent 生成 widget subtree → 增量 parse → 骨架渲染 → 增量填充 → patch（S11 右）。
- 落成 **Demo 一 Trip Planner 最小版**：先不上地图/支付，用 Text/Button/List
  原生 ArkUI 组件 + 一个自绘玻璃卡片，证明"Agent 生成的应用、树内跑真原生
  组件"这条命题。

## R3 · Demo 达标 + 指标采集（M6–M9）

真机才能测的：首次可交互 ≤1.5s、整页 patch ≤5s、60Hz、单帧 GPU ≤12ms、冷启
≤800ms、Demo 二玻璃多 pass 管线、包体 ≤30MB。

## 判断

- R0 + R1 主要是"把已验证的东西在真硬件上收口"，工作量可控。
- **R2 才是从"证明了管道"跨到"证明了 deck 论点"的那一步**，是真正的新工作：
  把已验证的「网络 + 包装层 + Agent 拓扑」三块，接上 deck 真正想要的
  「Splash 流式生成」。

## R0 启动所需

- 真机 `hdc list targets` 能认到设备（USB 已连）。
- octos 跑在 host 上，host 的 LAN IP（真机与电脑同一 WiFi）。
