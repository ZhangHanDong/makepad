spec: task
name: "AI Chat Liquid Glass Step 80 Native Interleave Entrypoints"
tags: [makepad, liquid-glass, apple-native, interleave, renderer]
---

## Intent

Record the concrete macOS renderer entry points that a future
`AppleNativeInterleave` prototype must change, based on the current
`CAMetalLayer`, repaint, draw-pass, and Studio screenshot implementation.

## Decisions

- This step is documentation only; it must not start the renderer split.
- The requirements document must name the exact files and functions inspected.
- The document must keep `AppleNativeInterleave` reserved until a prototype
  proves lower native surface sampling, upper Makepad foreground, input
  ownership, screenshot behavior, resize/DPI alignment, and lifecycle cleanup.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`
- this step spec

### Constraints

- Do not change Rust code in this step.
- Do not claim `AppleNativeInterleave` is implemented.
- Do not remove the existing visual, input, Studio, resize/DPI, or lifecycle gates.

## Acceptance Criteria

### Scenario: requirements name the single-surface creation entry point
Given the current macOS window creates one Metal surface
When the requirements document is inspected
Then it names `MetalWindow::new`, `MetalWindow::new_popup`, and the `CAMetalLayer` assignment to the view layer
Test: `rg "MetalWindow::new|MetalWindow::new_popup|CAMetalLayer|setLayer: ca_layer" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`

### Scenario: requirements name the repaint drawable entry point
Given window repaint currently targets the primary layer drawable
When the requirements document is inspected
Then it names `Cx::handle_repaint`, `metal_window.ca_layer`, `nextDrawable`, and `DrawPassMode::Drawable`
Test: `rg "Cx::handle_repaint|metal_window\\.ca_layer|nextDrawable|DrawPassMode::Drawable" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`

### Scenario: requirements name the offscreen non-evidence path
Given offscreen texture passes alone are not native interleave evidence
When the requirements document is inspected
Then it names `DrawPassMode::Texture` and states that it is not hosted as a native layer behind Apple glass
Test: `rg "DrawPassMode::Texture|not hosted as a native layer behind Apple glass" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`

### Scenario: requirements name the Studio screenshot implication
Given Studio screenshot behavior must be defined for split surfaces
When the requirements document is inspected
Then it names `Cx::draw_pass`, `build_screenshot_struct`, and states that screenshots currently copy from the presented window texture
Test: `rg "Cx::draw_pass|build_screenshot_struct|presented window texture" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`

### Scenario: interleave remains unimplemented
Given this is an entry point audit only
When the requirements document is inspected
Then it still says the document does not implement `AppleNativeInterleave` and that the backend remains reserved
Test: `rg "does not implement `AppleNativeInterleave`|`AppleNativeInterleave` remains reserved" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`

### Scenario: no unrelated files are changed
Given this step only updates renderer requirements
When the worktree status is inspected
Then only the requirements document and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-80-native-interleave-entrypoints.spec examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md "`
