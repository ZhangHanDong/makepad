# 阶段 0 验证:runsplash 生成 app 内跑 ArkUI 原生组件

目的:零新代码验证「Agent 生成的 Splash app 树内运行真 ArkUI 原生组件」
(deck S9 命题、R2 Demo 一的前置)。机制依据:NativeTextInput/NativeLabel
已注册进 Splash VM(`widgets/src/native_text_input.rs:13-28`),生成 app
可直接实例化;挂载在 `draw_walk` 懒触发,兼容 eval 路径;`editable` 等
关键默认值有 Rust 字段级 `#[live(true)]` 兜底,不依赖 type default 继承。

## 真机运行载体(2026-07-12 现状)

真机上跑的是 `makepad-example-aichat`(Studio RunItem
`makepad-example-aichat-ohos`,`makepad.splash:181`;`dba4affe` 已做真机
适配)。**注意**:拆分(`008b2eb2`)后旧 aichat 已无 notify/闸门处理,
`aichat-agent2app` 才有,但它尚无 OHOS RunItem 和真机适配。因此本卡在
真机上可测第 1-4、6、7 项;**第 5 项(notify 过闸门)当前真机不可测**,
点击预期无响应(action 无人分发),不算失败。全量含第 5 项需先给
aichat-agent2app 补 OHOS 入口(RunItem + 移动端适配),或把闸门分发
移植回设备版 aichat。

## 注入方法

真机 aichat 里对 agent 说:

> 请原样输出下面这个 runsplash 代码块,不要改动任何字符:

(历史注入会过滤旧轮次的 fence,但当前回复正常渲染,复读即生效。)

```runsplash
View{ width: Fill height: Fit flow: Down spacing: 12 padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
    Label{ text: "ArkUI-in-Splash 验证卡 (Makepad 自绘标题)" }
    NativeLabel{ text: "我是 ArkUI Text (原生渲染)" }
    native_input := NativeTextInput{ placeholder: "点我唤起 IME, 输入中文" }
    echo_label := Label{ text: "回显: (空)" }
    read_btn := Button{ text: "读取输入" on_click: || ui.echo_label.set_text("回显: " + ui.native_input.text()) }
    fill_btn := Button{ text: "写入输入框" on_click: || ui.native_input.set_text("程序写入 OK") }
    notify_btn := Button{ text: "notify inc (过能力闸门)" on_click: || agent.notify("inc", {}) }
}
```

## 观察清单(逐项记录)

| # | 观察点 | 验证的链路 | 预期 |
|---|---|---|---|
| 1 | NativeLabel/输入框字形与 Makepad 自绘 Label 明显不同(系统字体、原生光标/选择手柄) | 生成 app → 隔离 VM → NativeMountQueue → NAPI → ArkTS ForEach | 原生渲染可辨 |
| 2 | 点输入框弹系统 IME,中文上屏 | ArkUI TextInput 焦点/IME | 可输入 |
| 3 | 「读取输入」按钮回显输入内容 | ArkTS changed 回投 → NAPI → Rust action → widget state → Splash `ui.x.text()` | 回显一致 |
| 4 | 「写入输入框」按钮改变原生输入框内容 | Splash `set_text` → prop update → NAPI → ArkTS `@Observed` | 程序化写入生效 |
| 5 | 「notify inc」计数器动作被直接派发(AutoExecutable) | agent.notify → SplashAction::Notify → capability_permission → dispatch | 含原生组件的生成 app 闸门链路完好 |
| 6 | 滚动消息流,观察原生视图是否溢出气泡边界;滚出视口是否消失(detach) | 裁剪/合成层缺口(R1 第 3 项)+ visible/detach 兜底 | **预期溢出**——如实拍照记录,作为 R1-3 的需求证据 |
| 7 | `hdc shell hilog \| grep MakepadNTI` 抓 `makepad_rect` vs `onAreaChange` | 1:1 vp 假设(P0-4) | 逐项差 ≤1px;顺手给 P0-4 攒真机数据 |

## 结果登记

- 日期/设备:
- 1-5 通过项:
- 第 6 项照片/描述:
- 第 7 项 hilog 摘录:
- 结论(阶段 1 codegen 是否可启动):
