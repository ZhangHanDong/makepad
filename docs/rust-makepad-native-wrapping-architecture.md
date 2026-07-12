# Rust + Makepad 跨平台原生组件包壳架构设计

> **作者**：基于刘振华《Rust 作为 OHOS 主打开发语言的可行性论证》的延伸技术设计
> **日期**：2026-05-17
> **修订说明**：本版以 `~/Work/Projects/fw/fork-makepad` 与 `~/Work/Projects/fw/react-native` 的本地源码为准，收紧原文中过强的能力判断，明确区分“已有特例”“可复用模式”和“未来需要新增的通用抽象”。
> **核心命题**：借鉴 React Native 的原生组件包壳模式，让 Makepad 在保留 shader-first 自绘能力的同时，把 macOS 作为第一验证平台，先跑通平台原生 leaf widget 的 create/update/layout/command/event 闭环，再推广到 iOS / Android / ArkUI，并由 Splash DSL 作为上层跨平台应用语言驱动。

---

## 目录

- [一、三层架构的原意校准](#一三层架构的原意校准)
- [二、React Native 包壳的核心抽象](#二react-native-包壳的核心抽象)
- [三、Makepad 当前实际能力](#三makepad-当前实际能力)
- [四、迁移 RN 包壳模式到 Makepad + Rust](#四迁移-rn-包壳模式到-makepad--rust)
- [五、Splash 作为上层 DSL 的边界](#五splash-作为上层-dsl-的边界)
- [六、最小可行落地路线](#六最小可行落地路线)
- [七、对原论证文档的修正建议](#七对原论证文档的修正建议)
- [附录 A：关键源码锚点](#附录-a关键源码锚点)
- [附录 B：参考文献](#附录-b参考文献)

---

## 一、三层架构的原意校准

原论证文档中“Rust + Makepad + Splash”的真实含义不是只做自绘，也不是简单复制 React Native，而是三层叠加：

```text
┌─────────────────────────────────────────────────────────┐
│  Splash DSL                                             │
│  上层应用逻辑：状态、回调、跨端页面编排                  │
├─────────────────────────────────────────────────────────┤
│  Makepad widget 层                                      │
│  - Makepad 自绘组件：GlassPanel / PortalList / TextFlow │
│  - 平台原生 leaf widget：NSView / UIView / Android View │
│    / ArkUI                                              │
├─────────────────────────────────────────────────────────┤
│  Rust 平台层                                            │
│  ObjC runtime / JNI / NAPI / XComponent / EGL / Metal   │
└─────────────────────────────────────────────────────────┘
```

这条路线的最大杠杆是把两种跨端路线合并：

| 框架 | 自绘 | 原生组件包壳 | 状态 |
|---|---|---|---|
| React Native | 否 | 是 | 原生组件优先，跨端视觉一致性弱 |
| Flutter | 是 | 有，但 PlatformView 成本高 | 自绘优先，原生嵌入是补丁能力 |
| Makepad 当前 | 是 | 有特例，无通用抽象 | `WKWebView`、camera preview 等局部模式存在 |
| Makepad 目标 | 是 | 是 | 自绘组件与平台 leaf widget 统一由 Makepad widget tree 管理 |

因此，原文档中“Rust + Makepad 同时具备 Flutter 自绘 + RN 包壳”的表述需要修正为：**Makepad 已具备实现这条路线的底层条件，但通用包壳能力还需要新增 Host Widget 抽象层。**

本文的工程落点也需要明确：**第一阶段不是直接做 iOS/Android/OHOS 三端，也不是先做完整抽象层，而是先以 macOS 为 POC 平台。**原因是 macOS 后端已有 `WKWebView` 叠层实现、Metal window/native view 坐标换算路径、camera preview attach/update/detach 平台 op，能用最短路径验证 Host Widget 的关键链路。

---

## 二、React Native 包壳的核心抽象

React Native 新架构的价值，不在于“JS 能调用原生 API”，而在于它把原生组件封装为稳定契约：

| RN 抽象 | 作用 | Makepad/Rust 对应设计 |
|---|---|---|
| `codegenNativeComponent` / Native Component spec | 上层声明组件名、props、events、commands | Splash/Rust spec 声明 Host Widget 契约 |
| `ViewManager` / `RCTComponentViewProtocol` | 平台原生 view 的工厂、属性 setter、命令入口 | `NativeHostWidget` / platform host view |
| `ConcreteShadowNode<Props, EventEmitter>` | 类型化 props、layout、event emitter 的 C++ 节点 | Makepad `WidgetRef` + `#[derive(Script)]` props + `WidgetUid` |
| Mounting transaction / mutation list | 把树变更提交为 create/update/remove 操作 | P0 先 enqueue `CxOsOp` 并在同帧 main-thread drain；P4 后新增 `NativeMountOp` transaction |
| `EventEmitter` / `dispatchCommand` | 原生事件和命令的双向通道 | `WidgetAction` / `post_action` / `script_call` |

### 2.1 RN 的关键路径

本地 RN 源码中，典型路径是：

```text
JS spec:
  codegenNativeComponent<NativeProps>("RNTMyNativeView")
  codegenNativeCommands(...)

Codegen:
  Props / EventEmitters / ShadowNodes / ComponentDescriptors

iOS:
  RCTViewComponentView
  updateProps(...)
  handleCommand(...)

Android:
  SimpleViewManager
  @ReactProp setter
  ViewManagerDelegate
```

RN 对 Makepad 最值得借鉴的是这三点：

- **组件契约先行**：组件名、props、events、commands 必须稳定。
- **平台实现隔离**：同一个组件名在 iOS/Android/OHOS 分别有平台实现。
- **事件与命令分离**：props 是声明式状态，commands 是命令式操作，events 是原生回调。

macOS POC 不需要照搬 RN 的 codegen 体系，但要保留这三条边界。第一版可以手写 `NativeTextInput`，只要它的 API 已经按 props/events/commands 切分，后续迁移到 iOS `UITextField`、Android `EditText`、OHOS ArkUI `TextInput` 时就不会推翻上层 Splash 契约。

### 2.2 RN 源码校验结论

本地 RN 源码验证了上面的抽象判断：

| 文档判断 | RN 源码证据 | 对 Makepad 的结论 |
|---|---|---|
| 组件契约先行 | `MyNativeViewNativeComponent.js` 用 `codegenNativeComponent<NativeProps>` 声明 props/events，用 `codegenNativeCommands` 声明 commands | Makepad 应先固定 `NativeTextInput` 的 props/events/commands，再考虑宏和 codegen |
| iOS 平台隔离 | `RNTMyNativeViewComponentView.mm` 实现 `updateProps(...)`、`handleCommand(...)`、`EventEmitter` 回调 | macOS/iOS 应在平台 registry 中处理真实 native view，Rust widget 只发 op |
| Android 平台隔离 | `MyNativeViewManager.kt` 用 `SimpleViewManager`、`@ReactProp`、`ViewManagerDelegate`、`receiveCommand` | Android 后续应走 JNI registry + typed command，而不是让 Splash 直接调 Java |
| commands 独立于 props | `codegenNativeCommands.js` 把 `ref + command + args` 转成 `dispatchCommand` | `script_call` 与 `command_native` 分离是正确边界 |
| mutation transaction 不应 P0 先做 | `MountingTransaction` 包装 `ShadowViewMutationList`，且源码注释强调复制 mutation list 成本高 | Makepad P0/P1 先直接 enqueue `CxOsOp`，但必须同帧 main-thread drain；P4 后再做 `NativeMountQueue` |

RN 的实现还提示了一个输入框专属风险：`TextInput` 不是普通静态 native view。iOS `RCTTextInputComponentView` 和 Android `ReactTextInputManager` 都把 `focus`、`blur`、`setTextAndSelection` 作为 commands，并用 `eventCount` / `_comingFromJS` / `_ignoreNextTextInputCall` 这类状态避免“用户输入 -> 回调到 JS -> JS 回写 text -> 再触发输入事件”的回环。Makepad 的 macOS POC 可以先不实现完整 selection/eventCount，但必须至少有一个 `is_programmatic_update` guard，保证 `ui.input.set_text(...)` 不会反向触发 `on_change`。

### 2.3 不必完整复制 Fabric

Makepad 不需要一开始复制 RN 的完整 shadow tree / mounting coordinator。

原因是 Makepad 的上层脚本、widget tree、渲染循环都在 Rust 进程内，少了 JS -> JSI -> C++ -> 平台 view 这一整段边界。P0/P1 阶段应优先复用现有机制，但这里的“直接”只表示不引入完整 Fabric 式 transaction，不表示 widget 可以绕过平台 op 队列直接操作原生 view：

- `WidgetUid` 作为 native view id。
- `Area` 作为布局同步依据。
- `CxOsOp` 作为平台 op 队列。
- `script_call` 作为 Splash -> widget 命令入口。
- `WidgetAction` / `post_action` 作为 widget -> 应用事件入口。

等到滚动容器、z-order、批量更新、复杂 clip 进入范围后，再引入 RN 式 mutation transaction。

---

## 三、Makepad 当前实际能力

基于 `~/Work/Projects/fw/fork-makepad` 源码，Makepad 当前不是“完全没有原生嵌入”，而是**已有特例，没有通用 PlatformView/HostWidget 抽象**。

| 能力 | 当前状态 | 源码锚点 |
|---|---|---|
| iOS/macOS ObjC 调用 | 已有 | `platform/src/os/apple/apple_webview.rs` 使用 `makepad_objc_sys::{class, msg_send}` 创建 `WKWebView` |
| Android JNI | 已有 | `platform/src/os/linux/android/android_jni.rs` 等平台层文件 |
| OHOS NAPI + XComponent | 已有 | `platform/src/os/linux/open_harmony/open_harmony.rs`、`oh_callbacks.rs` |
| 原生 browser 嵌入 | 已有特例 | `widgets/src/browser.rs` -> `CxSystemBrowser` -> `WKWebView` |
| 原生 camera preview | 已有特例 | `widgets/src/video.rs` 使用 `Attach/Update/DetachCameraNativePreview` |
| Splash runtime UI | 已有 | `widgets/src/splash.rs` 使用 `eval_with_append_source` 生成/替换内部 `View` |
| Splash 调 widget 方法 | 已有 | `widgets/src/widget_async.rs` 注册 `ui` handle 并分发到 `script_call` |
| 通用 PlatformView 抽象 | 缺失 | 尚无统一 `create/update/destroy native leaf view` API |
| ArkUI leaf widget 包壳 | 缺失 | 当前 OHOS 是 ArkTS `XComponent` 托管 Makepad surface，不是 Rust 嵌入 ArkUI 子组件 |

### 3.1 已有模式：Browser

`Browser` 是最接近 Host Widget 的现有实现：

```text
Browser widget
  -> 计算 Makepad Area
  -> cx.system_browser(id).spawn/update/detach/set_url
  -> iOS/macOS 平台层创建 WKWebView
  -> setFrame / setHidden / addSubview
```

这个模式说明 Makepad 已经能做“Makepad widget 占位 + 平台原生 view 叠层”。缺的是把 `Browser` 的专用逻辑抽象成所有 native leaf widget 都能复用的机制。

对 macOS POC 来说，`Browser` 的价值尤其直接：`MacosSystemBrowser::update(...)` 已经处理了 Makepad `Area` 到 Cocoa view frame 的坐标翻转、clipped rect、`addSubview`、visible 同步。这些代码路径可以作为 `MacosNativeTextInput` 的实现模板。

### 3.2 已有模式：Video native preview

`Video` 的 camera preview 证明了第二个关键能力：

```text
Video widget
  -> draw_bg.area()
  -> attach_camera_native_preview(video_id, area)
  -> update_camera_native_preview(video_id, area, visible)
  -> detach_camera_native_preview(video_id)
```

这比 `Browser` 更接近通用 Host Widget，因为它已经把生命周期抽成了 attach/update/detach 三类平台 op。

### 3.3 OHOS 当前边界

OHOS 后端当前链路是：

```text
ArkTS EntryAbility
  -> makepad.onCreate(ArkGlue)
  -> ArkTS XComponent(type: SURFACE)
  -> Rust NAPI 注册 XComponent callbacks
  -> Rust EGL surface 渲染 Makepad
```

这说明 Makepad 已能在 OHOS 上作为一个 native surface 运行。但这不是 ArkUI 组件包壳。要实现 ArkUI leaf widget，还需要一套反向机制：

```text
Makepad NativeHostWidget
  -> CxOsOp::CreateArkUiNode / UpdateArkUiNode / DestroyArkUiNode
  -> NAPI 调 ArkTS host registry
  -> ArkTS 创建/更新/销毁 ArkUI Component
```

---

## 四、迁移 RN 包壳模式到 Makepad + Rust

### 4.1 第一版抽象：`NativeHostWidget`

建议先不要设计过重的 `MakepadHostWidget: WidgetMatchEvent`。P0 更适合一个小而稳定的内部 trait：

```rust
pub trait NativeHostWidget {
    fn native_kind(&self) -> LiveId;
    fn native_id(&self) -> LiveId;

    fn create_native(&mut self, cx: &mut Cx);
    fn update_native_props(&mut self, cx: &mut Cx);
    fn update_native_layout(&mut self, cx: &mut Cx, area: Area, visible: bool);
    fn command_native(&mut self, cx: &mut Cx, command: LiveId, args: ScriptValue);
    fn destroy_native(&mut self, cx: &mut Cx);
}
```

这里的 `script_call` 与 `command_native` 不是重复抽象，而是同一条链路的两端：

```text
Splash: ui.input.focus()
  -> Widget::script_call("focus", ...)
  -> NativeHostWidget::command_native(id!(focus), ...)
  -> CxOsOp::CommandNativeView
  -> macOS/iOS/Android/OHOS 平台实现
```

因此，`script_call` 保持 Splash -> Widget 的入口职责，`command_native` 保持 Widget -> Platform 的出口职责。这个边界可以避免每个 Host Widget 都手写平台 op 推送逻辑。

对应平台 op：

```rust
pub enum CxOsOp {
    CreateNativeView {
        id: LiveId,
        kind: LiveId,
    },
    UpdateNativeViewProps {
        id: LiveId,
        kind: LiveId,
        props: Box<dyn Any + Send>,
    },
    UpdateNativeViewLayout {
        id: LiveId,
        area: Area,
        visible: bool,
    },
    CommandNativeView {
        id: LiveId,
        kind: LiveId,
        command: LiveId,
        args: Box<dyn Any + Send>,
    },
    DestroyNativeView {
        id: LiveId,
    },
}
```

这个设计刻意沿用现有 `SystemBrowser` 与 camera preview 的做法：widget 只负责把 Makepad 世界里的 `Area/props/command` 变成平台 op，平台层负责真实 view。

`kind` 可以直接放在 `UpdateNativeViewProps` / `CommandNativeView` op 中，也可以在平台 registry 创建时随 `id` 一起保存。P0 建议先显式放进 op，减少跨文件追踪成本；等 registry 结构稳定后再决定是否去重。

这里不建议用 `props_json: String` 或 `args_json: String` 作为第一版接口。JSON 会把 RN 旧 Bridge 的序列化成本带回来，而 Makepad 的 widget 层与平台 op 层仍在 Rust 进程内，可以先用类型化 payload。每个 native widget kind 知道自己的 props 类型，平台层按 `kind` downcast 回具体 struct：

```rust
#[derive(Clone, Debug)]
pub struct NativeTextInputProps {
    pub text: String,
    pub placeholder: String,
    pub editable: bool,
}

match kind {
    id!(NativeTextInput) => {
        let props = props.downcast::<NativeTextInputProps>()?;
        // macOS/iOS: apply to NSTextField / UITextField
    }
    _ => {}
}
```

只有当 payload 必须跨 ArkTS/NAPI 边界、跨进程、持久化或异步线程长期保存时，才需要降级为可序列化格式。即便如此，也应优先考虑类型化 enum 或结构化 `LiveValue`，而不是把通用内部接口设计成 JSON 字符串。

P0 的 `Box<dyn Any + Send>` 仍然只是最小验证方案。RN 的 `ConcreteShadowNode<Props, EventEmitter>` 使用 `shared_ptr<const PropsT>`，天然具备不可变 props snapshot、引用共享和 O(1) 指针级 diff。Makepad P0 不需要马上复制这套结构，但当 native leaf widget 数量上升到几十到上百个时，每帧复制 props、downcast、手写字段 diff 会形成内存 churn。P4 引入 `NativeMountQueue` 前，应重新评估两种更稳的 props 表达：

- `Arc<dyn NativeProps + Send + Sync>` 或具体 `Arc<NativeTextInputProps>`，形成不可变 snapshot。
- 组件专用 typed enum，例如 `NativeViewProps::TextInput(Arc<NativeTextInputProps>)`，避免通用 `Any` 的运行时 downcast 扩散。

### 4.1.1 线程契约

`CxOsOp` 的消费必须发生在平台 UI/main thread。这不是 Makepad 自己的偏好，而是 AppKit / UIKit / Android View / ArkUI 的共同硬约束。

具体契约如下：

- `NativeHostWidget` 可以在 widget draw/event 阶段 enqueue `CxOsOp`，但不得直接调用 `NSView` / `UIView` / Android `View` / ArkUI node 方法。
- 平台层处理 `CxOsOp::CreateNativeView`、`UpdateNativeViewLayout`、`CommandNativeView`、`DestroyNativeView` 的循环必须运行在 UI/main thread。
- macOS POC 应复用 Makepad 现有 `handle_platform_ops` drain 机制；`MacosSystemBrowser` 已经在这个路径里处理 `WKWebView` 的 attach/update/detach。
- 从 render thread 或 worker thread 直接调用 `setFrame:`、`becomeFirstResponder`、`requestFocus` 这类平台 view 方法属于未定义行为级别的错误，表现可能是随机崩溃、焦点错乱或布局状态不同步。

RN 的对应做法是：`RCTMountingManager` 在 `scheduleTransaction` / `dispatchCommand` 中切回 main queue，`initiateTransaction` 用 `_transactionInFlight` 防止重入，并在 `RCTPerformMountInstructions(...)` 中应用整批 mutation。

### 4.2 平台实现边界

每个平台都需要一个 native view registry：

| 平台 | registry 持有物 | create/update/destroy |
|---|---|---|
| macOS | `HashMap<LiveId, ObjcId>` | `NSView` / `NSTextField` / `WKWebView` |
| iOS | `HashMap<LiveId, ObjcId>` | `UIView` / `UITextField` / `WKWebView` / `MKMapView` |
| Android | `HashMap<LiveId, GlobalRef>` | `View` / `EditText` / `TextureView` / `MapView` |
| OHOS | `HashMap<LiveId, ArkTsObjRef 或 NAPI ref>` | ArkTS host registry 创建 ArkUI component |

P0 应先只做 macOS，因为 macOS 本地开发、调试和验证成本最低，并且当前源码中的 `MacosSystemBrowser` 与 `MacosNativeCameraPreview` 已经证明 `addSubview + setFrame + setHidden`、坐标换算、visible 同步、detach 清理等关键路径可行。iOS 仍是下一步自然迁移目标，但不应作为第一验证平台。

### 4.3 两种合成模式

| 模式 | 原理 | 适用 | 优先级 |
|---|---|---|---|
| Hybrid Composition | 原生 view 作为真实平台 view 叠在 Makepad surface 上 | 输入框、Map、WebView、相机预览 | P0/P1 |
| Texture Layer | 原生内容渲染到 texture，Makepad 当普通 texture 采样 | 视频、可离屏渲染内容、未来 GlassPanel 折射 | P7 |

第一阶段应明确只做 Hybrid Composition。它的价值是快速验证：布局、focus、键盘、剪贴板、生命周期、visible、z-order。macOS POC 不应承诺完整移动端 IME 行为，移动端中文输入法、软键盘与组合输入应放到 iOS/Android 阶段验证。

Texture Layer 是差异化能力，但不适合 P0：它会立刻引入离屏渲染、触摸转发、IME 缺失、平台纹理同步等问题。

### 4.4 Mutation transaction 的位置

RN Fabric 的 `ShadowViewMutation` 很重要，但 Makepad 第一版不应先造完整 transaction。

建议分两层：

```text
P0/P1:
  Widget draw/layout 阶段只 enqueue update_native_layout(...)
  props setter 只 enqueue update_native_props(...)
  平台层在同一帧边界、同一 main-thread drain 中统一消费 CxOsOp

P4 以后:
  NativeMountQueue 收集 Create/Update/Destroy
  在平台 UI 队列统一 flush
```

### 4.4.1 直接 enqueue 的撕裂帧风险

P0/P1 不做完整 transaction 的代价是原子性较弱。同一帧内如果多个 Host Widget 同时更新，平台层若边产生边 apply，就可能出现 torn frame：部分 native view 已经应用新布局，部分仍停留在旧布局，用户会看到一帧不一致状态。

P0 的最低缓解要求是：

- Host Widget 只 enqueue `CxOsOp`，不直接 apply。
- 同一帧产生的 native ops 在 main thread 一次性 drain。
- drain 点应位于 Makepad 已有 frame boundary 上，例如事件处理或绘制完成后的平台 op 消费阶段，而不是每个 widget 自己立即触发平台 view 更新。

P4 引入 `NativeMountQueue` 后，这个问题应升级为 RN 风格 transaction：create/update/layout/destroy 先形成一批 mutation，再在平台 UI 队列按顺序提交，并加上重入保护。

后续可演进为：

```rust
pub enum NativeMountOp {
    Create { id: LiveId, kind: LiveId },
    UpdateProps { id: LiveId, kind: LiveId, props: Box<dyn Any + Send> },
    UpdateLayout { id: LiveId, area: Area, visible: bool, z_index: i32 },
    Destroy { id: LiveId },
}
```

### 4.5 Codegen 的实际时机

`#[makepad_host_widget]` proc-macro 是合理终态，但不应放在 P0。

更稳妥的顺序是：

1. 手写 `NativeTextInput`，打通 macOS `NSTextField`。
2. 把通用生命周期抽到 `NativeHostWidget`。
3. 迁移到 iOS `UITextField`。
4. 复制到 Android `EditText`。
5. 抽出 props/events/commands schema。
6. 最后再做 proc-macro。

否则宏会过早固化错误抽象。

---

## 五、Splash 作为上层 DSL 的边界

`~/Work/Projects/fw/fork-makepad/splash.md` 明确把 Splash 定义为 Makepad 的 UI scripting language，并且是面向 AI 生成的 terse reference。它给出的能力边界比“静态 UI DSL”更强：Splash 支持本地状态、函数、回调、局部重渲染和 widget method 调用。

对 Host Widget 方案来说，关键能力是：

- `Splash` 可以运行时 eval 代码，并把结果转成 Makepad `View`。
- `View` 能从脚本对象创建和更新 `WidgetRef` children。
- `widget_async.rs` 允许 Splash 通过 `ui.foo.method(...)` 找到 widget 并调用 `script_call`。
- widget 可以通过 `ScriptFnRef` 把事件回调到 Splash。
- `splash.md` 明确要求交互控件必须接上真实业务逻辑，例如 `on_click`、`on_return`、`on_change`。
- 动态列表可以用 `on_render` 重建，再通过 `ui.<list_id>.render()` 刷新。

因此，Host Widget 对 Splash 应该是透明的：

```splash
View{
    width: Fill height: Fit flow: Down spacing: 8

    input := NativeTextInput{
        width: Fill height: Fit
        placeholder: "输入中文"
        on_change: |text| ui.preview.set_text(text)
    }

    preview := Label{
        text: ""
    }
}
```

注意：上面示例中的 `input` 和 `preview` 都必须用 `:=`，因为 `splash.md` 规定只有 `:=` 声明的 named/dynamic child 才能被 `ui.<id>` 访问或被外部 override。普通 `label:` 是 static property，不适合回调里引用。

同理，`NativeTextInput` 如果要被其他回调读取或更新，也应写成：

```splash
input := NativeTextInput{
    width: Fill height: Fit
    on_return: |text| ui.preview.set_text(text)
}
```

这意味着 Host Widget 的 Splash API 设计必须遵守现有 Splash 语法，而不是另造一套 RN 风格 JSX 心智模型：

- 不使用 `Root{}` / `Window{}`，这些是宿主层 wrapper。
- 不使用分号或逗号分隔属性。
- 需要被 `ui.<id>` 访问的 child 必须用 `:=`。
- 容器默认应显式写 `height: Fit`，避免在 Fit parent 内变成 0 高度。
- 交互控件必须暴露可被 Splash 回调消费的 method，例如 `text()`、`set_text(...)`、`focus()`、`blur()`。

### 5.1 需要收紧的说法

不建议说“Splash 比 JS 更高级”或“无独立 GC”。更准确的表述是：

| 维度 | RN + JS/Hermes | Splash + Makepad |
|---|---|---|
| 生态与 AI 训练数据 | 极强 | 弱，需要工程化文档补足 |
| 与 UI 框架距离 | JS -> JSI -> C++ -> native | Splash VM -> Rust widget |
| 动态能力 | 成熟 | 已可用，但生态小 |
| 性能边界 | 有 JS runtime 与 bridge/JSI 边界 | 少一层 JS 边界，但仍有脚本 VM 成本 |

Splash 的定位应是：

- 适合页面编排、状态机、事件绑定、AI 生成 UI。
- 不适合性能关键逻辑、重计算、复杂平台 FFI。
- 平台能力必须通过 Rust widget 暴露，而不是让 Splash 直接调 ObjC/JNI/NAPI。

---

## 六、最小可行落地路线

### 6.1 推荐路线

| 阶段 | 产出 | 验收标准 |
|---|---|---|
| P0：macOS `NativeTextInput` | 手写一个 Makepad widget 包 `NSTextField` | 能显示、布局同步、focus/blur、`set_text`、原生输入回调 `on_change`、detach/close 不泄漏 |
| P1：抽出 `NativeHostWidget` + macOS registry | 不再为每个组件复制 create/update/destroy | `NativeTextInput` 和一个只读 `NativeLabel` 共用 lifecycle |
| P2：iOS `UITextField` | 复用 P1 契约迁移到 iOS | focus、软键盘、中文输入、选择/剪贴板、`on_change` 工作 |
| P3：Android `EditText` | Android 走 JNI 创建/更新原生输入框 | 中文输入法、emoji、剪贴板、软键盘工作 |
| P4：Hybrid Composition 基础设施 | z-order、visible、clip、滚动同步 | 在 Makepad 滚动容器中嵌入原生输入框或 MapView |
| P5：OHOS ArkUI leaf widget 原型 | NAPI 调 ArkTS host registry 创建 ArkUI TextInput/Button | ArkUI 组件可作为 Makepad leaf widget 布局和更新 |
| P6：schema/codegen | props/events/commands 生成胶水 | 一份 Rust spec 生成 macOS/iOS/Android/OHOS 基础代码 |
| P7：Texture Layer | 原生内容作为 texture 进入 Makepad shader 管线 | 视频/Map snapshot 可被 GlassPanel 折射或二次加工 |

### 6.2 macOS POC 范围

P0 的目标不是做通用平台视图系统，而是验证最小闭环：

```text
Splash
  input := NativeTextInput{ on_change: |text| ... }
      ↓
Widget::script_call("focus" / "set_text" / "blur")
      ↓
NativeTextInput widget
      ↓
CxOsOp::Create/UpdateProps/UpdateLayout/Command/Destroy
      ↓
macOS platform registry
      ↓
NSTextField addSubview / setFrame / becomeFirstResponder / delegate callback
      ↓
WidgetAction 或 widget_to_script_call 回到 Splash
```

P0 必须验证的点：

- **创建**：`NativeTextInput` 第一次 draw/layout 时能创建 `NSTextField`。
- **布局**：Makepad `Area` 能稳定同步到 Cocoa `NSRect`，包含 y 轴翻转与 visible。
- **坐标翻转**：Makepad `Area` 是 top-left origin，Cocoa `NSRect` 默认按 bottom-left origin 理解；P0 必须复用或等价实现 `MacosSystemBrowser::update(...)` 已验证的 y 轴换算。
- **props**：`text`、`placeholder`、`editable` 至少能从 Rust 同步到原生 view。
- **commands**：Splash 通过 `ui.input.focus()`、`ui.input.blur()`、`ui.input.set_text(...)` 能触发平台命令。
- **events**：用户输入能触发 `on_change`，回调参数是最新文本；程序化 `set_text` 必须被标记为 internal update，不能形成 `set_text -> on_change -> set_text` 回环。
- **AppKit 输入协议**：`NSTextField` 默认处理 `NSTextInputClient` 和候选词窗口；P0 只验证默认行为。若后续需要自定义 marked text、candidate window 或富文本输入，应单独作为后续议题。
- **Responder chain**：`focus/blur` 必须通过 AppKit responder chain 验证，不能只检查 Rust 状态位。
- **生命周期**：widget 不再可见或销毁时能 detach/close，不能留下悬空 native view。
- **范围外**：不处理滚动容器裁剪、多窗口拖拽、Texture Layer、iOS/Android IME、OHOS ArkUI node。

### 6.3 为什么 P0 选输入框

输入框比 MapView 更适合作为第一目标：

- 它能立即补 Makepad 纯自绘输入法的现实短板。
- 它最终能覆盖最难绕开的系统能力：IME、focus、键盘、剪贴板、选择区、组合输入。macOS POC 先验证桌面输入、focus 和事件回传；移动端 IME 在 iOS/Android 阶段单独验收。
- 它的渲染区域简单，便于先验证 Hybrid Composition。
- 它能迫使抽象认真处理事件回传，而不是只做静态 view。

macOS POC 不能完整证明移动端输入法能力，但能先验证 `NativeHostWidget` 的结构是否成立。等 macOS 路径稳定后，再把同一契约迁移到 iOS/Android，移动端特有问题单独验收。

### 6.4 P4/P7 的差异化

Hybrid Composition 是实用能力，Texture Layer 才是 Makepad 的差异化能力。

一旦原生内容能作为 texture 进入 Makepad：

- 原生视频可以被 Makepad shader 二次处理。
- Map 或相机画面可以成为 GlassPanel 的 backdrop。
- ArkUI 组件可以被“视觉整合”进 Makepad 的 shader-first UI。

但这不是第一阶段目标。第一阶段先把真实平台 view 放准、更新准、销毁准。

---

## 七、对原论证文档的修正建议

原论证文档应做三处修正。

### 7.1 修正 Robius 能力表述

原文说“Robius 框架已封装 Notifications / Location / Camera / Storage 等原生 SDK”。按本地 `fork-makepad` 源码，这不能直接作为 Makepad 主仓能力引用。

建议改成：

> Robius 生态正在探索跨平台系统能力封装；在当前 `fork-makepad` 中，可见的直接集成以 `robius-open` 这类能力为主。Notifications / Location / Camera / Storage 等不应表述为 Makepad 主仓既有能力。

### 7.2 修正 Makepad 原生包壳能力表述

建议把：

> Rust + Makepad 同时具备 Flutter 自绘和 RN 包壳能力。

改成：

> Rust + Makepad 已具备实现“自绘 + 原生包壳”并集架构的基础条件：自绘、shader、Splash、ObjC/JNI/NAPI、以及 `WKWebView`/camera preview 等特例已经存在；但通用原生组件包壳仍需新增 `NativeHostWidget` 抽象层。

### 7.3 修正 OHOS 包壳表述

当前 OHOS 后端是 ArkTS `XComponent` 承载 Makepad surface，不是 Rust 包 ArkUI node。

建议改成：

> OHOS 侧当前已跑通 Rust + NAPI + XComponent + EGL 的自渲染路径。ArkUI leaf widget 包壳是下一阶段目标，需要新增 ArkTS host registry 与 Rust 侧 native view ops。

---

## 附录 A：关键源码锚点

### A.1 Makepad（`~/Work/Projects/fw/fork-makepad`）

| 路径 | 用途 |
|---|---|
| `widgets/src/splash.rs` | `Splash` widget，运行时 eval 源码并生成内部 `View` |
| `widgets/src/view.rs` | `View` 从脚本对象创建/更新 `WidgetRef` children |
| `widgets/src/widget.rs` | `WidgetUid`、`WidgetNode`、`Widget::script_call` 等核心 trait |
| `widgets/src/widget_async.rs` | Splash `ui` handle、script -> widget、widget -> script 调用队列 |
| `widgets/src/browser.rs` | `Browser` widget，已有 native browser 特例 |
| `platform/src/cx_api.rs` | `CxSystemBrowser` 与 camera native preview 平台 op |
| `platform/src/os/apple/apple_webview.rs` | macOS/iOS `WKWebView` 创建、attach、update、detach |
| `platform/src/os/apple/macos/macos.rs` | macOS `handle_platform_ops` main-loop drain、`MacosSystemBrowser` / camera preview op 消费 |
| `widgets/src/video.rs` | camera native preview 的 attach/update/detach 调用点 |
| `platform/src/os/linux/open_harmony/open_harmony.rs` | OHOS NAPI entry、EGL surface、平台 op 处理 |
| `platform/src/os/linux/open_harmony/oh_callbacks.rs` | XComponent surface/touch/vsync callbacks |
| `tools/open_harmony/deveco/entry/src/main/ets/pages/Index.ets` | ArkTS `XComponent(type: SURFACE)` 承载 Makepad |
| `tools/open_harmony/deveco/entry/src/main/ets/makepad/makepad.ets` | ArkTS `ArkGlue`，键盘与环境信息桥接 |

### A.2 React Native（`~/Work/Projects/fw/react-native`）

| 路径 | 用途 |
|---|---|
| `packages/rn-tester/NativeComponentExample/js/MyNativeViewNativeComponent.js` | `codegenNativeComponent` 与 `codegenNativeCommands` 示例 |
| `packages/rn-tester/android/app/src/main/java/com/facebook/react/uiapp/component/MyNativeViewManager.kt` | Android `SimpleViewManager` 示例 |
| `packages/rn-tester/NativeComponentExample/ios/RNTMyNativeViewComponentView.mm` | iOS Fabric `RCTViewComponentView` 示例 |
| `packages/react-native/Libraries/NativeComponent/NativeComponentRegistry.js` | Native component 注册与 view config 获取 |
| `packages/react-native/Libraries/Utilities/codegenNativeComponent.js` | codegen fallback 到 `requireNativeComponent` |
| `packages/react-native/Libraries/Utilities/codegenNativeCommands.js` | commands -> `dispatchCommand` |
| `packages/react-native-codegen/src/generators/components/` | Fabric component codegen 目录 |
| `packages/react-native/ReactCommon/react/renderer/mounting/ShadowViewMutation.h` | Fabric mutation 类型 |
| `packages/react-native/ReactCommon/react/renderer/mounting/MountingTransaction.h` | Fabric mounting transaction |
| `packages/react-native/React/Fabric/Mounting/RCTMountingManager.mm` | iOS Fabric main-thread mounting、`_transactionInFlight`、`RCTPerformMountInstructions` |
| `packages/react-native/React/Fabric/Mounting/ComponentViews/TextInput/RCTTextInputNativeCommands.h` | iOS TextInput commands：`focus` / `blur` / `setTextAndSelection` |
| `packages/react-native/React/Fabric/Mounting/ComponentViews/TextInput/RCTTextInputComponentView.mm` | iOS TextInput 回调、`eventCount`、`_comingFromJS` 回环抑制 |
| `packages/react-native/ReactAndroid/src/main/java/com/facebook/react/views/textinput/ReactTextInputManager.kt` | Android TextInput `receiveCommand` 与 `focus/blur/setTextAndSelection` |
| `packages/react-native/ReactCommon/react/renderer/components/textinput/TextInputEventEmitter.cpp` | TextInput `onChange` / `onFocus` / `onBlur` payload 构造 |

---

## 附录 B：参考文献

### React Native 架构

- [React Native Architecture Overview](https://reactnative.dev/architecture/overview)
- [React Native Fabric Renderer](https://reactnative.dev/architecture/fabric-renderer)
- [React Native Render Pipeline](https://reactnative.dev/architecture/render-pipeline)
- [React Native Threading Model](https://reactnative.dev/architecture/threading-model)
- [Building Custom Fabric Components](https://reactnative.dev/docs/the-new-architecture/pillars-fabric-components)

### Flutter PlatformView

- [Flutter PlatformView Architecture](https://docs.flutter.dev/platform-integration/android/platform-views)
- [Hybrid Composition vs Texture Layer](https://docs.flutter.dev/platform-integration/ios/platform-views)

### Makepad 生态

- [Makepad GitHub](https://github.com/makepad/makepad)
- [Robrix - Matrix Client in Rust + Makepad](https://github.com/project-robius/robrix)
- [Project Robius 2024 Retrospective](https://robius.rs/blog/robius-retrospective-2024/)
- [Moly AI](https://github.com/moly-ai)

### 原论证文档

- 刘振华《Rust 作为 OHOS 主打开发语言的可行性论证》v1.0（2026-05-11）

---

## 结语

把 RN 的包壳模式迁移到 Makepad，核心不是复刻 Fabric，而是提炼出适合 Makepad 的 Host Widget 抽象：

- Makepad 自绘组件继续承担 shader-first UI。
- 原生组件作为 leaf widget 嵌入 Makepad widget tree。
- Splash 只面对统一 widget API，不关心底层是自绘还是原生。
- Rust 平台层用 ObjC runtime / JNI / NAPI 实现真实 view。

当前源码已经证明局部路径可行：`Browser` 证明了原生 view 叠层，`Video` 证明了 attach/update/detach 平台 op，OHOS 后端证明了 NAPI + XComponent + EGL surface。下一步不是继续论证，而是把这些特例收敛成 `NativeHostWidget`，先从原生输入框做一个可验证的最小闭环。
