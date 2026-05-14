# Step 197 - Current Completion Audit

Date: 2026-05-15

## Objective

The active objective is full Apple native Liquid Glass API support.

For this branch, that means the implementation needs more than "State 4" or a
single macOS visual pass. The concrete completion criteria are:

1. macOS native glass surfaces use public AppKit API semantics for style, tint,
   radius, container spacing, and contentView parenting.
2. aichat can run a macOS native Liquid Glass production route with native
   panels installed, readable foreground content, and visible interior material.
3. native glass controls have a clear ownership model and working activation
   paths on macOS and iOS.
4. UIKit native glass surfaces and controls are runtime-validated on iOS 26,
   not only compiled against older local SDKs.
5. Advanced behavior gates are either implemented and validated or explicitly
   scoped as unsupported for the current release: runtime switching,
   fullscreen, Stage Manager/split view, multi-display, inactive window,
   animated morphing, popup/modal, focus/accessibility.
6. The manual visual checklist has system-composited evidence or explicit
   human verdicts for the relevant macOS and iOS targets.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| macOS style raw values | `NativeGlassStyle::Regular.macos_raw_value() == 0`; `Clear == 1`; Studio logs through build `[69]` show `style_raw=1` for clear; above-metal regular build `[68]` shows `style_raw=0` | Covered for macOS runtime path |
| macOS public container API semantics | Step 196 promotes `NSGlassEffectContainerView.contentView` parenting by default; build `[69]` logs `role=container-created-contentView class=NSView` | Covered structurally; visual effect still needs manual verdict |
| macOS native batch install | build `[69]` logs `containers=1 panels_installed=4 panels_failed=0` and `app-substrate=apple-native-interleave state=Installed` | Covered for production interleave route |
| macOS visual Liquid Glass material | user reports have varied; recent diagnostic reports included flat grid/no liquid material; build `[69]` needs a fresh manual verdict after contentView default | Incomplete |
| Studio visual run policy | run through Studio remote release RunItems; latest production route build `[69]`, above-metal probe build `[68]` | Covered |
| Screenshot evidence | Studio screenshots omit AppKit native layer; system `screencapture` remains black on this machine | Blocked locally; requires manual visual verdict or external screenshot |
| macOS native controls | macOS native button activation and accessibility probes are documented in completion audit through Step 165 | Partially covered; broader focus/accessibility behavior remains incomplete |
| iOS native surfaces | UIKit dynamic installer skeleton exists; iOS target compile checks pass on SDK 18.5 | Incomplete: no iOS 26 runtime validation |
| iOS native controls | UIKit control descriptors/selector preflight/static checks exist | Incomplete: no iOS 26 runtime activation/visual/focus validation |
| Popup/modal | macOS popup probes exist; interleave popup create-path fixed in Step 191 | Incomplete: UIKit transient and modal native glass unproven |
| Fullscreen | fallback/restore probes exist for interleave | Not full native fullscreen support |
| Stage Manager / split view | same-display geometry probe coverage exists | Incomplete: no real Stage Manager/split-view validation |
| Multi-display | frame snapshot probe exists | Incomplete: current machine has single-display environment |
| Animated morphing / spacing | spacing update/install stability probes exist | Incomplete: visual morph quality/drift remains unproven |
| Reduce transparency/readability | policies and foreground token handling exist | Incomplete: bright/dark wallpaper and accessibility visual checks remain open |

## Latest Verification

Commands run after Step 196:

```text
cargo check -p makepad-platform --release
cargo test -p makepad-platform --release native_glass_container_content_view_env_defaults_to_content_view_parenting
cargo test -p makepad-platform --release native_glass
cargo check -p makepad-example-aichat --release
git diff --check
```

Studio release evidence:

```text
build_id=[69]
target=makepad-example-aichat-apple-native-interleave-experimental
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] native-container-panel-parent container=0000000000000010 requested=contentView role=container-created-contentView class=NSView
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

## Audit Verdict

The branch is not complete for the full objective.

The macOS production interleave path is now stronger because it follows the
public AppKit container contentView model and reaches State 4 with four panels.
However, this does not prove full Apple Liquid Glass support. The remaining
completion blockers are:

1. fresh macOS manual visual verdict after Step 196
2. iOS 26 runtime validation for UIKit native glass surfaces and controls
3. advanced behavior validation for fullscreen, Stage Manager/split view,
   multi-display, morphing, inactive-window visual behavior, popup/modal, and
   accessibility/focus
4. system-composited evidence source that is not Studio framebuffer and not the
   currently black `screencapture` output

