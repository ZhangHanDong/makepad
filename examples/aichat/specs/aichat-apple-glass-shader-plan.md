# aichat Apple Glass Shader 改造分析

本文记录如何参考 `examples/splash` 中的 Apple Glass / Gauss shader 效果，改造当前 aichat 的 shader glass 视觉。本文只描述分析和实施计划，不包含代码变更。

## 1. 参考来源

主要参考文件：

- `examples/splash/src/main.rs`
  - `TabGlass`
  - `gauss_popup`
  - `apple_glass_popup`
  - `glass_lens_button_popup`
  - `gradient_blur_popup`
- `widgets/src/gauss_view.rs`
  - `GaussRoundedView`
  - `AppleGlassRoundedView`
  - `GaussGradientRoundedView`
  - `request_window_gauss`
- `widgets/src/glass_panel.rs`
  - 当前 aichat 使用的 `GlassPanel`
- `examples/aichat/src/main.rs`
  - `ToolbarGlass`
  - `GlassSlider`
  - `app_shell`
  - `sidebar`
  - `main_area`
  - `composer`
  - `apply_glass_opacity`

## 2. 当前 aichat GlassPanel 的性质

aichat 当前使用 `GlassPanel`，它是轻量 shader glass：

- 半透明 `tint_color + tint_alpha`。
- SDF 圆角边框。
- 顶部 highlight band。
- 少量动态 noise。
- 可选 halo。
- 不采样窗口背后的 scene texture。
- 不做真实 blur、refraction、edge lensing 或 RGB diffraction。

因此它更像 “liquid-tinted translucent panel”，不是 Splash example 中那种真正依赖 backdrop snapshot 的 Apple Glass。

当前优点：

- 成本低。
- 可以用于大面积容器，例如整窗 shell、sidebar、main area。
- 不依赖 overlay capture。
- slider 只调 `tint_alpha` 也能稳定工作。

当前不足：

- 背景不会被模糊。
- 面板边缘缺少折射和色散。
- 玻璃厚度感主要靠 tint/highlight/noise，接近但不够真实。

## 3. Splash Example 的 Apple Glass 机制

Splash example 的 glass 效果来自 `GaussRoundedView` 系列。

### 3.1 GaussRoundedView

`GaussRoundedView` 的核心机制：

```text
draw_walk
  -> request_window_gauss(cx)
  -> bind scene_texture + mip blur textures
  -> shader sample_blur(level, uv)
  -> tint + specular + border + shadow
```

它不是普通透明面板，而是采样当前窗口已经绘制好的 scene snapshot，再根据 `blur_level` 从不同 mip texture 混合出模糊背景。

关键参数：

```text
blur_level
tint_color
tint_alpha
border_alpha
specular_strength
noise_strength
fallback_color
shadow_radius
shadow_offset
```

### 3.2 AppleGlassRoundedView

`AppleGlassRoundedView` 在 `GaussRoundedView` 基础上增加：

- rounded edge normal。
- edge lensing。
- RGB channel offset。
- diffraction。
- press/ripple 形变。
- 更强边缘 specular。

它的观感更像 Apple-style glass，因为边缘会折射背景，并带轻微色散。

Splash example 中 popup 参考参数：

```text
blur_level: 5.2
lensing_effect: 0.75
corner_radius: 18.0
tint_color: #b8b8b8
tint_alpha: 0.08
surface_alpha: 0.76
border_alpha: 0.56
specular_strength: 0.14
fallback_color: #8c8c8c
shadow_color: #000c
shadow_radius: 46.0
shadow_offset: vec2(0.0, 20.0)
diffraction_strength: 2.4
```

### 3.3 GaussGradientRoundedView

`GaussGradientRoundedView` 使用渐变 blur：

- 中心区域 blur 更强。
- 边缘区域 blur 更浅。
- lensing 默认关闭。
- 更适合大面积面板，视觉更稳，不会过度色散。

Splash example 中 popup 参考参数：

```text
blur_level: 4.35
gradient_blur_edge: 1.45
gradient_blur_edge_width: 0.20
gradient_blur_power: 0.75
tint_alpha: 0.045
border_alpha: 0.44
specular_strength: 0.08
shadow_radius: 44.0
shadow_offset: vec2(0.0, 18.0)
```

## 4. 重要限制

`GaussRoundedView` 依赖 `request_window_gauss(cx)`。该函数只在合适的 overlay drawing 阶段返回 snapshot。

这解释了为什么 Splash example 里 glass popup 效果成立：

```text
Window
  -> regular UI / dock
  -> overlay layer
      -> PopupNotification
          -> GaussRoundedView / AppleGlassRoundedView
```

Popup 是 overlay 内容，可以请求和采样窗口截图。

aichat 当前主 UI 是：

```text
Window
  -> app_shell
      -> sidebar
      -> main_area
          -> top_bar
          -> chat_shell
          -> composer
```

如果把所有 `GlassPanel` 直接替换成 `AppleGlassRoundedView`，有两个风险：

- 大面积 blur sampling 成本高。
- 若不在正确 overlay capture 路径，shader 可能只能使用 `fallback_color`，看不到真实 backdrop blur。

所以不建议全量替换。

## 5. 推荐视觉分层

推荐采用分层迁移，而不是替换全局 `GlassPanel`。

| aichat 区域 | 推荐组件 | 原因 |
| --- | --- | --- |
| `app_shell` | 继续使用 `GlassPanel`，或低强度 `GaussGradientRoundedView` | 面积最大，必须控制成本 |
| `sidebar` | `GaussGradientRoundedView` | 需要稳重、可读，不宜强色散 |
| `main_area` | `GaussGradientRoundedView` | 大面积内容区，适合中心 blur + 边缘浅 blur |
| `ToolbarGlass` | 低强度 `AppleGlassRoundedView` | 小面积，能体现 glass edge |
| `composer` | `AppleGlassRoundedView` | 最适合做 Apple Glass，浮层感强 |
| `SendButton` | 保留当前自定义 shader | 现有按钮已有明确质感，不必强行替换 |

## 6. 推荐参数范围

### 6.1 Composer

```text
blur_level: 4.8 - 5.2
lensing_effect: 0.45 - 0.65
lensing_strength: 10.0 - 14.0
lensing_width: 18.0 - 24.0
diffraction_strength: 1.2 - 2.0
tint_alpha: 0.045 - 0.075
border_alpha: 0.42 - 0.62
specular_strength: 0.12 - 0.18
shadow_radius: 34.0 - 46.0
shadow_offset: vec2(0.0, 12.0 - 20.0)
```

Composer 是最推荐优先改造的位置，因为它在视觉上天然是浮层，而且面积比整窗小。

### 6.2 ToolbarGlass

```text
blur_level: 3.5 - 4.5
lensing_effect: 0.20 - 0.35
lensing_strength: 8.0 - 12.0
diffraction_strength: 0.8 - 1.2
tint_alpha: 0.045 - 0.07
border_alpha: 0.32 - 0.46
specular_strength: 0.08 - 0.12
shadow_radius: 14.0 - 24.0
```

Toolbar 面积小，但内部有 DropDown 和 Label。色散不能太强，否则文字边缘会显脏。

### 6.3 Sidebar / Main Area

```text
component: GaussGradientRoundedView
blur_level: 4.0 - 4.8
gradient_blur_edge: 1.2 - 1.6
gradient_blur_edge_width: 0.16 - 0.24
gradient_blur_power: 0.70 - 1.0
tint_alpha: 0.04 - 0.08
border_alpha: 0.18 - 0.36
specular_strength: 0.05 - 0.10
lensing_effect: 0.0
```

大面板建议关闭 lensing。主要靠 backdrop blur、轻 tint、轻 specular 建立质感。

## 7. Slider 行为调整

aichat 当前 `apply_glass_opacity` 只更新：

```text
draw_bg.tint_alpha
```

迁移到 Gauss/Apple glass 后，slider 不应只控制 tint。建议把 slider 映射为 material density：

```text
slider
  -> tint_alpha
  -> border_alpha
  -> specular_strength
  -> optional blur_level
```

推荐保守映射：

```text
low slider:
  tint_alpha lower
  border_alpha lower
  specular_strength lower
  blur_level unchanged or slightly lower

high slider:
  tint_alpha higher
  border_alpha higher
  specular_strength higher
  blur_level unchanged or slightly higher
```

不要把 slider 直接映射到非常高的 blur，因为 blur 成本和可读性都会受影响。

注意：`GaussRoundedView` 中声明了 `surface_alpha`，但当前 shader `pixel` 没有直接使用它来控制 output alpha。已有 `set_opacity` 会写 `surface_alpha` 和 `tint_alpha`，但实际视觉主要来自 `tint_alpha`。如果后续要让 opacity 更语义化，需要同步修 `gauss_view.rs` shader。

## 8. 实施顺序

建议分四步做，每一步都用 Studio screenshot 比对。

### Step 1：定义 aichat 专用 alias

不要直接修改全局 `GlassPanel`。新增 aichat 本地 alias，例如：

```text
AichatAppleGlass
AichatGradientGlass
```

这样可以在 aichat 内渐进替换，同时不影响其他 widgets 或 examples。

### Step 2：只替换 composer

优先把 `composer := GlassPanel` 改为 `AichatAppleGlass`。

验收点：

- 输入框仍可聚焦。
- 中文输入不失焦。
- 背后内容可被轻微模糊。
- 文字可读。
- 截图中 composer 有明显玻璃边缘，但不过度发亮。

### Step 3：替换 ToolbarGlass

将 `ToolbarGlass` 从 `GlassPanel` 迁移到低强度 Apple Glass。

验收点：

- DropDown 能正常展开。
- backend label 不糊。
- glass slider 操作仍更新数值。
- toolbar 不抢视觉中心。

### Step 4：评估 sidebar/main_area

如果前两步稳定，再考虑：

- `sidebar` 改为 `AichatGradientGlass`。
- `main_area` 改为 `AichatGradientGlass`。
- `app_shell` 继续保留 `GlassPanel`，或只做很弱的 gradient blur。

## 9. 风险与回退

### 9.1 性能风险

全屏多层 Gauss blur 可能显著增加 GPU 成本。应避免：

- app shell、main、sidebar、composer、toolbar 全部使用强 Apple Glass。
- 大面积色散和 ripple。
- 每帧频繁更新所有 glass uniforms。

### 9.2 可读性风险

Apple Glass 的 edge diffraction 很适合 popup 或按钮，但不适合承载大量文字的主区域。

大文本区域应优先使用：

```text
GaussGradientRoundedView + low tint + low specular + no lensing
```

### 9.3 Capture 路径风险

如果某个位置拿不到 gauss snapshot，视觉会退化到 `fallback_color`。因此每一步都必须截图验证，而不能只看编译通过。

### 9.4 App Store / macOS Native Glass 风险

这次分析只讨论 Makepad shader glass，不依赖 macOS native backdrop。它不解决此前 `WindowBackdrop.Blur` 的平台层问题，也不需要重新开启 `window.backdrop`。

## 10. 建议验收方式

每一步改造后运行：

```text
makepad-example-aichat
```

并在 Studio 中抓取：

- 初始空状态截图。
- 输入框 focus 截图。
- 输入中文时截图。
- backend dropdown 展开截图。
- appgen workspace 截图。

对比基线：

- 当前 aichat shader glass。
- `examples/splash` 中 `Apple Glass` popup。
- `examples/splash` 中 `Gradient Blur` popup。

最终目标不是让 aichat 完全等同 Splash popup，而是让 aichat 拥有更真实的玻璃层次，同时保持 chat 工具的稳定、可读和低干扰。

