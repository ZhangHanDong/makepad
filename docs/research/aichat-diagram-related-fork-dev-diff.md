# Fork Dev Diff: aichat / streaming-markdown-kit / makepad-diagram-kit

Date: 2026-04-27

Baseline:

- Repository: `ZhangHanDong/makepad`
- Branch: `dev`
- Comparison target: `makepad/makepad:dev`
- Local command basis: `git diff upstream/dev..dev`

## Summary

Current fork `dev` is ahead of upstream `dev` by 166 commits.

Diff size:

- Files changed: 75
- Insertions: about 15314
- Deletions: about 268

Only part of this diff belongs to the `aichat` / `streaming-markdown-kit` / `makepad-diagram-kit` feature line. The rest includes Canvas tools, shader music player, pomodoro, Splash experiments, and other support work that should not be mixed into the same upstream PRs.

## Recommended PR Buckets

The relevant changes should be split into these buckets:

- `aichat` application integration
- Markdown, table, math, Mermaid, and diagram-fence rendering
- SVG renderer support for Mermaid output
- AI backend support, including OpenAI-compatible streaming and thinking mode
- macOS borderless window resize support and dependency patching
- Documentation and specs

## 1. aichat Application

These files belong to the aichat example itself.

- `examples/aichat/src/main.rs`
- `examples/aichat/Cargo.toml`
- `examples/aichat/resources/LXGWWenKaiMono-Regular.ttf`
- `examples/aichat/resources/LiberationMono-Regular.ttf`
- `examples/aichat/resources/NotoColorEmoji.ttf`
- `examples/aichat/resources/NotoSans-Regular.ttf`
- `examples/aichat/specs/aichat-claude-code-backend.spec.md`
- `examples/aichat/specs/aichat-liquid-glass-ui.spec.md`

Main scope:

- Streaming chat example
- Inline Mermaid rendering
- Inline `diagram` JSON rendering through `makepad-diagram-kit`
- Liquid-glass UI
- Input controls, backend picker, thinking toggle
- History storage and assistant-message safety handling
- Font stack for mixed Chinese, code, emoji, and Markdown output

Largest file:

- `examples/aichat/src/main.rs`: about `+1922 / -84`

## 2. Markdown / Streaming / Table / Math / Diagram Fence

These files support rich Markdown rendering for aichat and related consumers.

- `widgets/src/markdown.rs`
- `widgets/src/math_view.rs`
- `libs/latex_math/src/parser.rs`
- `widgets/src/text_flow.rs`

Main scope:

- Streaming-safe Markdown rendering
- Mermaid code-block hook
- `diagram` fenced-code handling
- GFM table rendering
- Table cell math rendering
- Link interaction support
- Local and HTTP image handling
- CJK column-count and layout behavior

Largest file:

- `widgets/src/markdown.rs`: about `+2490 / -84`

Notes:

- This bucket is a core dependency of the aichat feature line.
- It should be reviewed separately from the aichat UI because it changes reusable widget behavior.

## 3. Mermaid / SVG Rendering Support

These files improve Makepad SVG support so Mermaid output can render correctly.

- `libs/svg/src/text.rs`
- `libs/svg/src/edge.rs`
- `libs/svg/src/document.rs`
- `libs/svg/src/parse.rs`
- `libs/svg/src/style.rs`
- `libs/svg/src/transform.rs`
- `libs/svg/src/animate.rs`
- `libs/svg/src/lib.rs`
- `libs/svg/tests/text_parse.rs`
- `draw/src/svg/render.rs`
- `draw/src/svg/mod.rs`
- `draw/src/shader/draw_svg_glyph.rs`
- `widgets/src/vector.rs`

Main scope:

- SVG `<text>` parsing and rendering
- Marker-end arrow support
- SVG edge support
- Transform fixes
- Text test coverage
- Draw shader support for SVG glyph output

Notes:

- This bucket is needed for Mermaid visual fidelity.
- It can be proposed as a lower-level Makepad SVG enhancement PR.

## 4. AI Backend / Thinking / Claude Code

These files update the shared AI backend layer used by aichat.

- `libs/makepad_ai/src/agent.rs`
- `libs/makepad_ai/src/backend.rs`
- `libs/makepad_ai/src/backends/claude_code_cli.rs`
- `libs/makepad_ai/src/backends/mod.rs`
- `libs/makepad_ai/src/backends/openai.rs`
- `libs/makepad_ai/src/types.rs`

Main scope:

- OpenAI-compatible streaming backend improvements
- Moonshot/Kimi thinking-mode request support
- Reasoning-delta handling
- Incomplete SSE stream detection
- Optional Claude Code CLI backend

Notes:

- OpenAI-compatible thinking support and Claude Code CLI backend should be split if preparing upstream PRs.
- The thinking UI in aichat depends on this backend layer, but the backend changes are reusable.

## 5. macOS Window / Dependency Integration

These files support running the aichat UI cleanly inside this fork.

- `platform/src/os/apple/macos/macos_window.rs`
- `Cargo.toml`

Main scope:

- Keep borderless macOS windows resizable when requested
- Patch `https://github.com/ZhangHanDong/makepad.git` `makepad-widgets` back to local workspace path during local builds

Notes:

- The macOS resize fix is independently useful and already has a focused commit.
- The root `Cargo.toml` patch is needed because external git kits depend on `ZhangHanDong/makepad:dev`; without it, Cargo pulls a second Makepad crate graph and widget/script types no longer match local workspace types.

## 6. Documentation / Research / Plans

These files document design intent and implementation planning.

- `docs/research/aichat-claude-code-backend.md`
- `docs/research/aichat-ui-polish-notes.md`
- `docs/superpowers/plans/2026-04-24-aichat-claude-code-backend.md`
- `docs/superpowers/plans/2026-04-24-aichat-liquid-glass-ui.md`

Notes:

- These are useful for project continuity.
- They should not be mixed into runtime code PRs unless the PR explicitly includes design documentation.

## 7. Not Core To The aichat / Diagram Feature Line

These paths exist in the fork diff but should not be bundled into the aichat, streaming Markdown, or diagram-kit upstream PRs.

- `tools/canvas/**`
- `examples/shader_music_player/**`
- `tools/pomodoro/**`
- `widgets/src/splash.rs`
- `platform/script/src/heap.rs`
- `platform/script/src/object_heap.rs`
- `studio/desktop/src/app_messages.rs`
- `code_editor/src/code_editor.rs`

Notes:

- Some of these were useful references for the aichat UI, especially Canvas.
- They are still separate product/tooling work and should be reviewed independently.

## Suggested Upstream PR Split

Suggested order:

1. macOS borderless resize fix
2. SVG text / marker / Mermaid rendering support
3. Markdown widget support for Mermaid, tables, math, images, and diagram fences
4. OpenAI-compatible backend streaming/thinking improvements
5. aichat example integration
6. Optional Claude Code backend
7. Optional docs/spec PR

Rationale:

- Lower-level rendering support should land before the aichat example depends on it.
- Backend changes should be reviewable without UI noise.
- The aichat example should be the final integration PR after dependencies are understandable.
