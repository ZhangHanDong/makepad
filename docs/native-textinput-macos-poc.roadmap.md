# NativeTextInput macOS POC：阶段路线

## Context

`native-textinput-macos-poc` 分支已落地 P0 最小可编译骨架（spec/plan 详见 `docs/native-textinput-macos-poc.{spec,plan}.md`，架构原则详见 `docs/rust-makepad-native-wrapping-architecture.md`）。本文档把"已完成 / 接下来"拆开：当前是 **P0A**（编译链路成立），但架构文档定义的 P0 还未关闭。真正的 P0 验收要求 events 联通 + lifecycle 不残留，统一归入 **P0B**。后续抽通用、跨平台、Hybrid Composition、transaction 按依赖关系顺延。逐阶段 completion audit 见 `docs/native-textinput-macos-poc.audit.md`。

---

## P0A：已完成 — 最小可编译骨架（保留）

已落地：

- `NativeTextInput` 注册到 widget / Splash 系统（`widgets/src/native_text_input.rs:10`、`widgets/src/lib.rs`）。
- widget 可发 native view create / layout / props / command ops（`widgets/src/native_text_input.rs`、`platform/src/cx_api.rs`）。
- macOS 能创建并 attach `NSTextField`（`platform/src/os/apple/apple_native_text_input.rs:37`、`platform/src/os/apple/macos/macos.rs:1242`）。
- `text()` / `set_text()` / `focus()` / `blur()` 入口存在。
- `cargo check -p makepad-widgets` + `git diff --check` 通过。

**状态**：P0A done，**P0 未关闭**。

---

## P0B：完成 macOS 真实运行时闭环（下一步）

目标：让 `NativeTextInput` 成为可验收的 macOS Host Widget POC。

### Step 0：读现有 TextInput 模板（前置）

在动手写 callback bridge 之前，把以下三处读穿、镜像 API 形状：

- `widgets/src/text_input.rs:627` — `cx.widget_action(uid, TextInputAction::Changed(text))` emit 模式。
- `widgets/src/text_input.rs:2527` — `pub fn changed(&self, actions: &Actions) -> Option<String>` helper 形状。
- `widgets/src/text_input.rs:2583` — `enum TextInputAction { None / KeyFocus / KeyFocusLost / Returned / Escaped / Changed / KeyDownUnhandled }` 完整签名。

`NativeTextInputAction` / `NativeTextInputRef` 完全镜像这个 shape，零额外认知负担。

### Step 1：加最小 runtime example

- 路径 `examples/native_text_input/`，crate 命名 `makepad-example-native-text-input`（沿用 `examples/text_input/Cargo.toml` 模板）。
- 内容：`NativeTextInput` + Focus / Set Text / Blur 按钮 + preview Label。
- P0B 主 smoke 只验收非滚动容器里的 `native_input`，避免把裁剪/z-order
  问题混进事件闭环验收。
- P1.5 另用同一个 example 里的 `clipped_native_input` 验证
  `ScrollYView` / clipped host / z-order 行为。

**当前实现进度**：`examples/native_text_input` 已按仓库现有 example 结构落地，并注册到 `makepad.splash` 的 Studio release RunItem。example 同时展示 `changed`、`key_focus`、`key_focus_lost`、`selection_changed` 四类 widget action 的状态反馈，并提供按钮触发 `NativeLabel` 文本更新以覆盖共享 host registry 的第二个组件；`set_both_button` 覆盖同帧多 native host 更新；`clipped_native_input` 覆盖 P1.5 滚动/裁剪场景。`cargo check -p makepad-example-native-text-input --release` 已通过。

### Step 2：接 AppKit 输入回调

路径：

```
NSTextField delegate/action（controlTextDidChange:）
  -> Cx::post_action(NativeTextInputChanged { id, text })
     // post_action 本身就是全局 sender + SignalToUI，见 platform/src/action.rs:116
  -> 主循环 handle_action_receiver -> Event::Actions 派发
  -> NativeTextInput::handle_event 匹配自己 uid
     -> 更新 self.text / self.last_text
     -> cx.widget_action(uid, NativeTextInputAction::Changed(text))
     -> 调 on_change(text)
```

- ObjC delegate 子类用 `ClassDecl::new(...) + decl.add_method(...)` 模式（仓库中没有 `declare_class!` 宏）。直接参考：
  - `platform/src/os/apple/macos/macos_delegates.rs` —— `PostDelegate` / `MenuDelegate` / `RenderWindowDelegate` 等多个 macOS delegate 模板。
  - `platform/src/os/apple/ios/ios_text_input.rs` —— iOS text input / IME protocol 样本，可参考文本生命周期与输入协议边界；它不是直接的 AppKit delegate 模板。
- 关键：widget action emit 在前，`on_change` 回调在后，确保非 Splash 用户也能订阅 `NativeTextInputAction::Changed`。

### Step 3：加 NativeTextInputAction

至少：

```rust
#[derive(Clone, Debug, Default)]
pub enum NativeTextInputAction {
    #[default]
    None,
    KeyFocus,
    KeyFocusLost,
    Changed(String),
}
```

并提供 `NativeTextInputRef::changed(&Actions) -> Option<String>` helper（形状参考 `TextInputRef::changed` text_input.rs:2527）。

**当前实现进度**：`NativeTextInputAction::{None, KeyFocus, KeyFocusLost, Changed}` 与 `NativeTextInputRef::{changed,key_focus,key_focus_lost}` 已落地。`widgets/src/native_text_input.rs` 单测覆盖 native changed/focus action 进入 widget 后的文本同步与 widget action emit，并验证其它 native id 的 action 不会串到当前 widget。

### Step 4：完成 programmatic guard

- `set_text(..., programmatic: true)` 不触发 `on_change`。
- 平台层 `MacosNativeTextInput::set_text` 已有 `is_programmatic_update` 字段（apple_native_text_input.rs:98），P0B 让 AppKit callback 在分发 `NativeTextInputChanged` 之前先查这位，true 则吞掉。

### Step 5：修 ObjC ownership

- `NSTextField` `alloc/init` 后必须在 `cleanup` / `Drop` 路径 `release`。**这是 P0B 的硬性 gate**，只针对 `MacosNativeTextInput`。
- **retain 计数账（写进代码注释）**：`alloc/init` → 1；`addSubview` → 2；`removeFromSuperview` → 1；`cleanup` 一次 `release` → 0。不要重复 release。
- 顺手 audit `MacosSystemBrowser`（apple_webview.rs）是否有同款缺失：若修复 trivial 则一起补；否则单独开 issue 跟踪，**不阻塞** P0B。

### Step 6：lifecycle 收口

- `visible: false` → `detach`（已实现，保留）。
- `Event::Shutdown` → `close()`。
- **Splash hot-reload / widget drop**：widget `impl Drop` 拿不到 `&mut Cx`。正常热替换时可通过 `Cx::post_action` 走全局 action sender，但 `post_action` 当前会 unwrap 全局 sender 和 send result（`platform/src/action.rs:116`），因此 Drop 路径必须是 best-effort、不可 panic。P0B 应先加一个非 panic helper 或显式 guard，再投递：
  ```rust
  Cx::try_post_action(NativeTextInputCloseRequested { text_input_id: self.native_id().0 });
  ```
  主循环 `handle_action_receiver` 拿到后，由一个全局 listener / app-level handler 把它翻成 `CxOsOp::CloseNativeView` 推给平台层。
  `Event::Shutdown -> close()` 仍是正式 shutdown 清理路径，Drop 投递只负责 hot-reload / widget tree 替换这类运行中清理。
- 实施前先查 `Browser` / `Video` 是怎么处理类似场景的，照搬可用的那条；P0B 不要新设计这条基础设施。
- 旧 native field 不残留。

### Step 7：Studio runtime 验证（按 AGENTS.md:1）

- **不**用 shell `cargo run` 验证 UI（AGENTS.md:7、AGENTS.md:11）。
- 把 example 加入 `makepad.splash` runnable。
- 通过 Studio bridge：`ClearBuild` 旧 build → `RunItem` 新 build → `WidgetTreeDump` / `Screenshot` / `Click` / `TypeText` 验证。
- Studio bridge 启动：`target/release/cargo-makepad studio --studio=127.0.0.1:8001`（AGENTS.md:25）。

**当前实现进度**：Studio release RunItem 已能在本机 macOS 路径启动
`makepad-example-native-text-input-macos-standalone`。用户已手工确认主/副
native input 的按钮链路、`Set Both`、`Set Label` 和基础交互修复后正常。
`makepad-example-native-text-input-macos-standalone-diag` 的诊断运行记录到
`docs/native-textinput-evidence/macos-leak-runtime.md`，`NativeTextInput`
live counter 从 3 回到 0 后退出。仍需按
`docs/native-textinput-evidence/macos-manual-checklist.md` 完成人工视觉/输入
checklist，尤其是 changed event、滚动裁剪和 z-order。

### P0B 验收标准

- `NSTextField` 可见、位置正确（坐标翻转沿用 SystemBrowser 路径）。
- `focus()` 成为 first responder；`blur()` 失焦。
- `set_text("...")` 更新原生文本，**不**触发 `on_change`。
- `select_all()` 触发平台原生全选命令，用于 selection/clipboard smoke test。
- `copy()` / `cut()` / `paste()` 转发给平台原生控件，由系统剪贴板实现具体行为。
- 用户手动输入触发 `on_change(text)`，且 `ui.input.text()` 返回最新输入。
- Rust app 可通过 `NativeTextInputAction::Changed` 订阅。
- 文本选区变化触发 `NativeTextInputAction::SelectionChanged { start, end }`；macOS 侧通过 `NSTextField.currentEditor.selectedRange` / `textViewDidChangeSelection:` 回投，移动端通过各平台原生选区回调回投。
- hide / close / Splash hot-reload 不残留 native view。
- **泄漏验证方法（两选一，任一通过即可）**：
  - **A（首选，可在 Studio release 跑）**：在 `MacosNativeTextInput::new` / `Drop` 各放一个 `static AtomicI32`，用 `cfg(feature = "diag-native-leak-count")` feature gate（不限 debug/release）。由于 Studio `RunItem` JSON 不能临时追加 cargo flags，需要在 `makepad.splash` 增加一个专用 diagnostic runnable（当前为 `makepad-example-native-text-input-macos-standalone-diag`），其 `on_run` cargo args 固定带上 `--features diag-native-leak-count`。用这个 runnable 通过 Studio bridge 跑 release，assert 关闭后计数回到 0。
  - **B（fallback，非 UI）**：对 release Studio 进程跑 `leaks <pid>` 或 Instruments Leaks，确认 `NSTextField` 实例数与活跃 widget 数一致。

---

## P1：抽通用 NativeHostWidget / macOS registry

依赖：P0B done。

目标：不让每个 native widget 复制一套 lifecycle。

- 抽 `NativeHostWidget` 内部 trait（`native_kind / create / update_props / update_layout / command / destroy`）。
- 抽 typed props/commands/events 基础结构（避免通用 JSON）。
- macOS registry 改为统一 `HashMap<LiveId, Box<dyn MacosNativeHost>>`。
- 用 `NativeTextInput` + 一个简单 `NativeLabel`（包 non-editable `NSTextField` 或 static `NSTextView`）验证复用。

**当前实现进度**：代码侧已落地 macOS `MacosNativeHost` trait、统一 `native_hosts: HashMap<LiveId, Box<dyn MacosNativeHost>>` registry，以及最小 `NativeLabel` widget。`NativeTextInput` 与 `NativeLabel` 已共享同一套 macOS attach / update / detach / cleanup 入口。原先按控件拆开的 spawn / layout / props / command / close ops 已替换为 `CreateNativeView`、`UpdateNativeViewLayout`、`UpdateNativeViewProps`、`CommandNativeView`、`DetachNativeView`、`CloseNativeView`，payload 使用 `NativeHostKind` / `NativeHostProps` / `NativeHostPropUpdate` / `NativeHostCommand` typed enum。`cx_api` 单测覆盖 widget-facing `Cx::native_text_input(...)` / `Cx::native_label(...)` API 会经 NativeMountQueue flush 成 typed ops，而不是绕开 transaction 直接写平台 op；TextInput 的 create/layout/text/placeholder/editable/focus/blur/detach 与 Label 的 create/layout/text/close 均有 typed op 覆盖。`widgets` 单测覆盖 `NativeTextInput` event/action 同步、setter state、shutdown lifecycle，以及 `NativeLabel` 基础 state/lifecycle；example 可手动触发 `NativeLabelRef::set_text` 验证 Label props update。按当前策略，P1 代码/静态收敛完成；Studio runtime 验证保留为 checklist，不阻塞继续推进。

---

## P1.5：Hybrid Composition 基础能力

依赖：P0B done（可与 P1 并行）。

目标：解决 P0B example 暂时规避的问题。

- host view + content view 双层结构，参考 `MacosSystemBrowser::ensure_attached`（apple_webview.rs:114）+ `clipped_browser_layout`（apple_webview.rs:223）。
- clip rect / scroll container / visible 同步。
- z-order 策略。
- 多 native view 同屏排序。
- 明确滚动容器内 `NativeTextInput` 的行为。

**当前实现进度**：代码侧已为 `NativeTextInput` / `NativeLabel` 增加 clipped host view，布局使用 `unclipped_rect` + `clipped_rect` 计算 host/content 两层 frame，并有 `apple_native_host` 单元测试覆盖裁剪偏移和空 clip 隐藏。macOS native host 现在通过 `order_native_host_view` 在 layout 更新时按当前 native op 顺序重排 host view，避免同屏多个 native host 永久停留在首次 attach 顺序。滚动容器、z-order、多 native view 同屏的真实视觉行为仍需 Studio runtime 验证；按当前策略保留为 checklist，不阻塞代码推进。

---

## P2：iOS UITextField

依赖：P1 done。目标：复用 P1 契约迁移到 iOS。

验收：focus / blur / 软键盘 / 中文输入 / emoji / 剪贴板 / selection 同步 / `on_change` 工作 / programmatic `set_text` 不回环。

**当前实现进度**：代码侧已新增最小 `IosNativeTextInput`，复用 P1 的 `CreateNativeView` / `UpdateNativeViewLayout` / `UpdateNativeViewProps` / `CommandNativeView` typed op 契约，底层使用 `UITextField` + target/action bridge。已覆盖 text / placeholder / editable / focus / blur / select_all / copy / cut / paste / editing changed / begin / end editing 的编译路径，native event action 构造复用 `NativeTextInputChanged::new` / `NativeTextInputFocusChanged::new`。最小 selection event contract 已新增为 `NativeTextInputSelectionChanged { start, end }`，widget 侧会更新 selection state 并发出 `NativeTextInputAction::SelectionChanged`；macOS 侧通过 `NSTextField.currentEditor.selectedRange` / `textViewDidChangeSelection:` 回投 selection，iOS 侧通过 `textFieldDidChangeSelection:` 读取 `selectedTextRange` 并回投该 action。`NativeTextInputRef::select_all(cx)` / `copy(cx)` / `cut(cx)` / `paste(cx)` 与 Splash 同名方法已接到 native command，example 增加 Select All / Copy / Cut / Paste 按钮。`cargo check -p makepad-example-native-text-input --target aarch64-apple-ios` 已通过。P2 仍缺真机/模拟器 runtime 验证，尤其中文输入、emoji、剪贴板与 selection。

`makepad.splash` 已注册 `makepad-example-native-text-input-ios-sim`
RunItem，使用仓库内 `examples/native_text_input` 通过
`cargo-makepad apple ios run-sim -p makepad-example-native-text-input --release`
路径验证，避免为 P2 另起 ad hoc app。

## P3：Android EditText

依赖：P2 done。目标：JNI registry + `EditText`。

验收：中文输入法 / emoji / 软键盘 / 剪贴板 / focus / blur / `on_change`。

**当前实现进度**：代码侧已在 Android `MakepadActivity` 中新增 `EditText` overlay registry，并在 Rust `android_jni` / `android.rs` 中接入 P1 typed native op。`CreateNativeView(TextInput)` 会创建 `EditText`，layout / visible / text / placeholder / editable / focus / blur / select_all / copy / cut / paste / close 通过 Activity 方法走 UI thread，Java `TextWatcher` / focus listener 回调 `MakepadNative.onNativeTextInputChanged` / `onNativeTextInputFocusChanged`，再由 Rust 投递 `NativeTextInputChanged` / `NativeTextInputFocusChanged`。Android JNI callback 复用 `NativeTextInputId` typed conversion，并拒绝负的 `jlong` id；native event action 构造复用 `cx_api` constructors。最小 selection event contract 已在 Rust/widget/schema 侧落地，Android 侧通过 `NativeTextInputView.onSelectionChanged` 回调 `MakepadNative.onNativeTextInputSelectionChanged`，再由 JNI 投递 `NativeTextInputSelectionChanged`。仓库已有的 `libs/jni-sys` / `libs/android_state` 现在通过 workspace `[patch.crates-io]` 固定为全局单实例依赖，`tools/native_textinput_android_static_check.sh` 已覆盖 Java overlay registry、programmatic guard、changed/focus/selection callbacks、JNI callback posting 与 Rust command dispatch，`cargo check -p makepad-widgets --target aarch64-linux-android` 已通过。普通 `cargo check -p makepad-example-native-text-input --target aarch64-linux-android` 不是正确 Android app gate，因为 `app_main!` 在 Android 上导出 cdylib entrypoint 而不生成 bin `main`；真实 app build 仍需走 `cargo-makepad android ...` / 设备路径。P3 仍缺真实 Android package/deploy/runtime 验证；剪贴板与 selection 同步不能按完整 P3 验收关闭。

`makepad.splash` 已注册 `makepad-example-native-text-input-android`
RunItem，使用仓库内 `examples/native_text_input` 通过
`cargo-makepad android run -p makepad-example-native-text-input --release`
路径部署到设备/模拟器。

## P4：NativeMountQueue / transaction

目标：解决多 native widget 同帧更新的 torn frame 风险。

- 收集 `create / update / layout / destroy` mutation。
- frame boundary 一次性 flush。
- 主线程重入保护（参考 RN `_transactionInFlight`）。
- props snapshot 策略（`Arc<dyn NativeProps>` 或 typed enum），避免每帧复制/手写 diff。

**当前实现进度**：代码侧已新增 `NativeMountQueue` / `NativeMountMutation`，`NativeTextInput` 与 `NativeLabel` 的 create / layout / props / command / detach / close API 不再直接写 `platform_ops`，而是先进入 native mount queue。各平台在 drain `platform_ops` 前统一 `flush_native_mount_queue()`；flush 时按 `platform_ops.pop()` 的消费模型反向压入 op，保证平台实际处理顺序仍是 widget 发出的 create → layout → props → command。queue 已支持基础 coalescing：同帧同 host 的重复 layout、同类 props、重复 detach 会合并，close 会吞掉同帧之前同 id 的 mutation；command 仍保留完整顺序，并且 layout/props 不会跨过同 host command 被合并，避免 `set_text -> copy/cut -> set_text` 这类序列被重排。已加单元测试覆盖 flush 顺序、layout/props coalescing、close 收口、command 顺序保留、command barrier、reentrant flush guard、close 只影响同 id host，以及 `CxNativeTextInput` / `CxNativeLabel` API 到 typed mount ops 的端到端 flush（含 TextInput placeholder/editable/blur/detach 和 Label create/layout/text/close）。按当前策略，P4 代码/静态收敛完成；Studio runtime 下多 native view 同帧更新的 visual 验证保留为 checklist。

## P5：OHOS ArkUI leaf widget 原型

目标：让 P1 typed native op 能在 OpenHarmony 上落到 ArkTS host registry。

- Rust `open_harmony.rs` 消费 `CreateNativeView(TextInput)` / layout / props / command / detach / close。
- ArkTS `ArkGlue` 暴露 native text input host methods。
- `Index.ets` 维护 overlay `TextInput` 列表，按 Makepad rect 同步 position / size / visible。
- ArkTS `TextInput.onChange/onFocus/onBlur` 通过 XComponent context 回投 Rust NAPI callback，再转成 `NativeTextInputChanged` / `NativeTextInputFocusChanged` action。

**当前实现进度**：代码侧已接入最小 ArkTS overlay host。`tools/open_harmony/deveco/entry/src/main/ets/pages/Index.ets` 持有 `nativeTextInputs` state，并实现 create / update / detach / set text / placeholder / editable / focus / blur / select_all / copy / cut / paste / close；`tools/open_harmony/deveco/entry/src/main/ets/makepad/makepad.ets` 为 `ArkGlue` 增加 host 转发方法；`platform/src/os/linux/open_harmony/open_harmony.rs` 将 generic native ops 映射为 ArkTS calls，字符串参数按 UTF-8 bytes + explicit length 创建 JS string，不再因用户文本中包含 NUL 字节而拒绝调用；`oh_callbacks.rs` 增加 native text input changed / focus changed / selection changed callbacks，复用 `NativeTextInputId` 的 decimal string parser，并复用 `cx_api` native event constructors。ArkTS `TextInput` 已挂 `.onTextSelectionChange(...)` 回投 `handleNativeTextInputSelectionChanged(...)`；`selectAllNativeTextInput` 通过 `TextInputController.setTextSelection(...)` 全选并回投 selection，`copyNativeTextInput` / `cutNativeTextInput` / `pasteNativeTextInput` 已接 `@kit.BasicServicesKit` pasteboard 的纯文本读写，cut/paste 会同步 ArkTS state 并回投 changed/selection。`module.json5` 已声明 `ohos.permission.READ_PASTEBOARD`。OpenHarmony target 编译当前受 crates.io 下载阻塞（最近几次 `cargo check -p makepad-platform --target aarch64-unknown-linux-ohos` 阻塞在 OpenHarmony NAPI crates 下载，包含 `napi-ohos 0.1.3` 与 `napi-derive-backend-ohos 0.0.7`，当前环境无法解析 `static.crates.io`），且仍缺 DevEco/设备 runtime 验证。

`makepad.splash` 已注册 `makepad-example-native-text-input-ohos`
RunItem，使用仓库内 `examples/native_text_input` 通过
`cargo-makepad ohos run -p makepad-example-native-text-input --release`
路径做 DevEco/设备验证。

## P6：schema / codegen

目标：生成 props/events/commands 胶水，避免手写多平台重复桥接。

**当前实现进度**：已新增 `platform/src/native_host_schema.rs` 作为最小 Rust schema/codegen 切片，覆盖 `TextInput` 与 `Label` 的 props / prop updates / commands / events，并提供 `generate_native_host_manifest()` 生成稳定 manifest 文本。TextInput commands 当前覆盖 `focus`、`blur`、`select_all`、`copy`、`cut`、`paste`；events 当前覆盖 `changed`、`focus_changed`、`selection_changed`。`cx_api` 单测会把 schema 与当前 `NativeHostKind` / `NativeHostProps` / `NativeHostPropUpdate` / `NativeHostCommand` / event payload 契约做一致性校验，manifest 测试使用精确 golden 字符串，避免手写 enum 与 schema 漂移或输出格式弱漂移。当前阶段仍未把 typed enum 替换为生成产物；后续要在 build/codegen 工具链中决定输出 Rust/ArkTS/Java/ObjC glue 的落点。

## P7：Texture Layer

目标：原生内容作为 texture 进入 Makepad shader 管线。

**当前实现进度**：已新增 `platform/src/native_texture_layer.rs` 的最小资源契约，提供 `NativeTextureLayer`、`NativeTextureLayerFrame` 与 `NativeTextureLayerBridge::alloc_native_texture_layer(host_id)`，底层先复用现有 `TextureFormat::VideoExternal` 分配外部纹理句柄。单测覆盖 host id 绑定、texture slot 的 `VideoExternal` 格式、不同 host 分配不同 texture id，以及 native frame generation 递增同步契约。这个切片只固定资源/同步边界，尚未把任何 native widget 渲染进 texture，也未处理触摸转发、IME、平台纹理上传或 GlassPanel 折射验收。

---

## 优先级原则

先把 P0B 做实，再抽 P1。**不要**在 events / lifecycle / retain-release 没闭环前做 codegen 或跨平台扩展。

example 参考源以仓库内真实例子为准：`examples/native_text_input` 是本 POC 的主 smoke example；事件语义参考 `examples/text_input` / `widgets/src/text_input.rs`；Studio runnable 入口参考 `makepad.splash` 中已有 `RunStudioRelease` / standalone diagnostic RunItem。

具体对齐到这些仓库 example：

- `examples/native_text_input`：本 POC 唯一 canonical smoke app。所有 macOS/iOS/Android/OHOS runtime evidence 都必须从这个 example 的 Studio RunItem 收集。
- `examples/text_input`：输入配置、`returned`/`changed` 风格、`makepad_test` selector 写法的参考。后续若给 `NativeTextInput` 补 UI 自动化测试，测试文件形状应贴近 `examples/text_input/tests/ui.rs`。
- `examples/todo`：真实 app 中 `TextInput` action 进入 app state 的参考，尤其是 `returned(actions)` 后更新状态、清空输入、`redraw` 的模式。
- `examples/uizoo/src/tab_textinput.rs`：控件状态矩阵参考。后续若把 NativeTextInput 放进 uizoo，应只做展示矩阵，不替代 `examples/native_text_input` 的 runtime gate。
- `examples/splash` 与根 `makepad.splash`：只参考 RunItem / Studio release 路径，不用 raw `cargo run` 验 UI。

完成度判断同样以这个 example 为准：运行时验收必须通过
`makepad-example-native-text-input` /
`makepad-example-native-text-input-macos-standalone` /
`makepad-example-native-text-input-macos-standalone-diag`
收集 evidence，并让 `tools/native_textinput_completion_audit.sh` 通过。
该 audit 脚本默认只检查真实 runtime/device evidence marker，不把
`cargo check`、static grep 或 unit test 当成 UI/device 验收。

## 关键文件锚点

- `widgets/src/native_text_input.rs` — widget 骨架；callback / Drop 改动入口。
- `widgets/src/text_input.rs:627` / `:2527` / `:2583` — `TextInputAction` 镜像模板。
- `platform/src/os/apple/apple_native_text_input.rs` — AppKit delegate 子类 / release 修复入口。
- `platform/src/os/apple/macos/macos.rs:1242` — op 消费分支；channel drain 改动入口。
- `platform/src/cx_api.rs:92` / `:442` — op shape；P1 改 `CreateNativeView { kind, props }` 入口。
- `platform/src/os/apple/apple_webview.rs:52` / `:114` / `:223` — `MacosSystemBrowser` 模板（host_view 双层 / clip）。
- `widgets/src/splash.rs` — Splash eval 入口（example smoke 验证用）。
- `AGENTS.md:1` — Studio remote runbook（P0B step 7 强制路径）。

## Verification

### Current non-UI verification snapshot

Latest local verification in this workspace:

- `cargo check -p makepad-widgets` — passed.
- `cargo check -p makepad-example-native-text-input --release` — passed.
- `cargo check -p makepad-example-native-text-input --release --features makepad-widgets/diag-native-leak-count` — passed.
- `cargo check -p makepad-example-native-text-input --target aarch64-apple-ios` — passed, with pre-existing warnings in `apple_game_input.rs` / `web_socket.rs`.
- `cargo check -p makepad-example-native-text-input --target aarch64-apple-ios --release` — passed, with the same pre-existing warnings.
- `cargo check -p makepad-widgets --target aarch64-linux-android` — passed, with pre-existing warnings in `openxr.rs`.
- `cargo test -p makepad-widgets native_ --lib` — passed, 9 tests.
- `cargo test -p makepad-platform native_ --lib` — passed, 26 tests.
- Targeted `rustfmt --check --config skip_children=true` over NativeHost-related Rust files — passed.
- `tools/native_textinput_example_static_check.sh` — passed and is included in
  `tools/native_textinput_poc_verify.sh`; it guards the example package,
  workspace member entry, Studio RunItem/diag RunItem registration, required
  smoke controls/actions, and clear-build guard defaults.
- `tools/native_textinput_studio_gate_static_check.sh` — passed and is included
  in `tools/native_textinput_poc_verify.sh`; it guards normal/diagnostic JSONL
  packets and verifies that unguarded `RunItem` launch is refused.
- `tools/native_textinput_host_queue_static_check.sh` — passed and is included
  in `tools/native_textinput_poc_verify.sh`; it guards typed native host enums,
  `NativeMountQueue` coalescing/close/order/reentrant tests, widget-facing
  `CxNativeTextInput` / `CxNativeLabel` APIs, `NativeLabel` registration,
  macOS/iOS native host registries, clipped host layout, and per-platform
  native queue flush calls.
- `tools/native_textinput_apple_static_check.sh` — passed and is included in
  `tools/native_textinput_poc_verify.sh`; it guards macOS AppKit delegate
  wiring, retain/release cleanup, diagnostic leak feature wiring, iOS
  target/action wiring, programmatic guards, selection callbacks, clipboard
  commands, typed command dispatch, and widget Drop close requests.
- `tools/native_textinput_android_static_check.sh` — passed and is included in
  `tools/native_textinput_poc_verify.sh`; it guards Android Java overlay,
  JNI callback, command dispatch, and local crate patch wiring.
- `tools/native_textinput_ohos_static_check.sh` — passed and is included in
  `tools/native_textinput_poc_verify.sh`; it guards ArkTS controller,
  pasteboard, permission, and callback wiring.
- `tools/native_textinput_schema_texture_static_check.sh` — passed and is
  included in `tools/native_textinput_poc_verify.sh`; it guards NativeHost
  schema exports, manifest golden coverage, schema-vs-typed-contract tests,
  texture layer bridge exports, `VideoExternal` allocation, frame generation
  tests, and the documented P6/P7 seed boundaries.
- `git diff --check` — passed.

Current local runtime/device blockers:

- Full-workspace `cargo fmt --check` is not a usable gate yet because it enters unrelated pre-existing files with formatting issues, including `platform/script/src/shader_output.rs` and `platform/src/os/apple/apple_util.rs`.
- Studio bridge launch still fails in this Codex sandbox with
  `failed to connect to studio websocket at 127.0.0.1:8001/ui: connect failed: Operation not permitted (os error 1)`.
- Android library graph target verification now passes. Android app packaging
  and runtime behavior still require the cargo-makepad Android cdylib/package
  flow and a device/emulator.
- OpenHarmony target verification remains blocked in this sandbox by crates.io DNS access; recent failing downloads include `napi-ohos 0.1.3` and `napi-derive-backend-ohos 0.0.7` from `static.crates.io`, and DevEco/device API validation is still required for the ArkTS `TextInputController` and pasteboard path.

P0B 完成后：

```bash
# 编译门槛
tools/native_textinput_poc_verify.sh

# 最终完成度门槛；在真实 Studio / simulator / device evidence 补齐前应失败
tools/native_textinput_completion_audit.sh

# macOS P0B/P1 smoke evidence helper；仍需先按 AGENTS.md 清掉旧 build
tools/native_textinput_macos_studio_smoke.sh --clear-build-id N
tools/native_textinput_macos_leak_smoke.sh --clear-build-id N

# 等价的手工命令：
cargo check -p makepad-widgets
cargo check -p makepad-example-native-text-input --release
cargo check -p makepad-example-native-text-input --release --features makepad-widgets/diag-native-leak-count
cargo test -p makepad-example-native-text-input --test ui --no-run
cargo check -p makepad-example-native-text-input --target aarch64-apple-ios
cargo check -p makepad-example-native-text-input --target aarch64-apple-ios --release
cargo test -p makepad-widgets native_ --lib
cargo test -p makepad-platform native_ --lib
git diff --check

# UI 验证（按 AGENTS.md）
# 1. 推荐用脚本生成并发送 Studio bridge JSONL；如需 leak counter，加 --diag
tools/native_textinput_studio_gate.sh --list-builds
tools/native_textinput_studio_gate.sh --clear-build-id N
tools/native_textinput_studio_gate.sh --diag --clear-build-id N
# 1b. 常规 macOS smoke 可以直接用 evidence helper 产出 marker 文件
tools/native_textinput_macos_studio_smoke.sh --clear-build-id N
tools/native_textinput_macos_leak_smoke.sh --clear-build-id N

# 也可以手工启动 Studio bridge；如需 leak counter，用 makepad.splash 中带 diag feature 的专用 runnable
target/release/cargo-makepad studio --studio=127.0.0.1:8001
# 2. 通过 JSON Lines:
#    {"ListBuilds":[]}
#    {"ClearBuild":{"build_id":[N]}}  （旧 build）
#    {"RunItem":{"mount":"makepad","name":"makepad-example-native-text-input"}}
#    等待 BuildStarted / AppStarted / RunViewCreated，记录新 build_id
#    {"Screenshot":{"build_id":[M]}}
#    {"WidgetTreeDump":{"build_id":[M]}}
#    {"Click":{"build_id":[M],"x":..,"y":..}}
#    {"TypeText":{"build_id":[M],"text":"hello"}}
# 3. P0B smoke checklist:
#    - screenshot 中能看到两个 NativeTextInput，其中 named native_input 可点击输入
#    - 点击 Focus 后，Status 变为 focus requested，然后收到 focus action
#    - 对 native_input TypeText("hello") 后，Value 变为最新文本，Status 变为 changed action received
#    - 点击 Set Text 后原生文本变为 "hello native"，Value 同步为 "hello native"，且不额外触发用户输入路径的 changed 回环
#    - 点击 Select All 后，Status 先变为 select all requested，随后 selection event 显示非空范围
#    - 点击 Copy / Cut / Paste 后不崩溃；Cut/Paste 后 changed action 和 Value 能反映原生文本变化
#    - 点击 Set Label 后 NativeLabel 文本更新，证明共享 native host registry 的第二个组件可更新 props
#    - 点击 Blur 后，Status 变为 blur requested，然后收到 blur action
#    - 多次 ClearBuild -> RunItem 不残留旧 NSTextField / NativeLabel
#    - 泄漏验证按 P0B 验收 A 或 B（feature-gated atomic counter 或 leaks tool）
```

P1 完成后：

- `NativeLabel` 与 `NativeTextInput` 共享同一套 lifecycle。
- `CxOsOp::CreateNativeView { kind, props }` 替换两套 spawn op 不破任何现有调用。

## Runtime Gate Checklist

这些 gate 不是 `cargo check` 的替代品；它们用于关闭各阶段 roadmap 中明确标为 runtime/device 的验收项。

### macOS Studio gate（P0B / P1 / P1.5）

- 使用 `makepad-example-native-text-input` 或
  `makepad-example-native-text-input-macos-standalone` RunItem，必要时用
  `makepad-example-native-text-input-macos-standalone-diag` 跑 leak counter。
- P0B 使用 `native_input` 做非滚动主 smoke，确认事件、focus、selection、clipboard 与 lifecycle 闭环。
- P1.5 使用同一 example 里的 `clipped_native_input` 做滚动/裁剪场景，确认 native view 不越界、不遮挡、不乱序。
- 验证 lifecycle：连续 `ClearBuild -> RunItem` 至少 5 次，或用 Splash re-eval 替换 View，确认旧 native views 关闭且 leak counter 回到活跃 widget 数。

### iOS device/simulator gate（P2）

- 运行 `makepad-example-native-text-input` 到 iOS simulator 或真机。
- 首选 Studio RunItem `makepad-example-native-text-input-ios-sim`。
- 验证 focus / blur 弹收软键盘。
- 输入英文、中文组合输入、emoji，确认 `Changed` 与 `Value` 同步。
- 点击 Select All / Copy / Cut / Paste，确认 selection status、剪贴板行为与 Value 同步。
- 验证 programmatic Set Text 不产生额外 changed 回环。

### Android device gate（P3）

- 先跑 `cargo check -p makepad-widgets --target aarch64-linux-android`，确认 Android library graph 覆盖 `NativeTextInput` / platform bridge。
- 不使用普通 bin `cargo check -p makepad-example-native-text-input --target aarch64-linux-android` 作为 gate；Android app 由 `cargo-makepad` 以 `--lib --crate-type=cdylib` 构建，`app_main!` 不在 Android bin target 中生成 `main`。
- 再通过 Android RunItem 或 `cargo-makepad android ... run` 的 Studio-defined runnable 路径部署，避免绕过 Studio runbook。
- 首选 Studio RunItem `makepad-example-native-text-input-android`。
- 验证中文输入法、emoji、软键盘、focus / blur、Select All / Copy / Cut / Paste、Changed/Value 同步。
- 验证 activity rotate/resume 后 overlay registry 不残留旧 `EditText`。

### OpenHarmony DevEco/device gate（P5）

- 先在可访问 crates.io 的环境跑 `cargo check -p makepad-platform --target aarch64-unknown-linux-ohos`。
- 首选 Studio RunItem `makepad-example-native-text-input-ohos`。
- 用 DevEco 编译 ArkTS，重点确认 `.onTextSelectionChange(...)`、`focusControl.requestFocus(...)` 与新增 host methods 的 ArkUI API 形状。
- 在设备上验证 create/update/detach/focus/blur/changed/selection_changed。
- `selectAllNativeTextInput` / `copyNativeTextInput` / `cutNativeTextInput` / `pasteNativeTextInput` 已接 ArkUI `TextInputController` + pasteboard API，但关闭 P5 前仍必须用 DevEco/设备确认 API 形状、权限弹窗、selection 回投和 changed 回投行为。
