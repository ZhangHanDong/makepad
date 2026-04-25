# DeepSeek V4 Attention — Canvas Splash Animations

6 animated scenes explaining DeepSeek V4's attention mechanics, based on the
[vllm blog post](https://vllm.ai/blog/deepseek-v4).

## Scenes

| # | File | Topic |
|---|------|-------|
| 1 | `scene1-long-context-pressure.splash` | KV cache grows linearly, attention compute grows O(N²) |
| 2 | `scene2-shared-kv-inverse-rope.splash` | Shared K/V saves 2x memory; inverse RoPE cancels absolute rotation |
| 3 | `scene3-c4a-overlapped-compression.splash` | C4A uses 8-token windows with stride 4, producing 3 overlapping compressed entries |
| 4 | `scene4-query-two-memory-paths.splash` | Query attends sliding window (local) + top-k compressed (long-range) |
| 5 | `scene5-causal-boundary.splash` | q attends C_i iff q >= 4i+7; walk q through boundaries |
| 6 | `scene6-c4a-vs-c128a.splash` | c4a (250K entries, top-512) vs c128a (8K entries, top-8192) |

## Usage

Post a scene to a running Canvas instance:

```bash
PORT=$(cat /tmp/makepad-canvas.port)
curl -sf -X POST "http://127.0.0.1:$PORT/splash" \
    --data-binary @scene1-long-context-pressure.splash
```

## Splash patterns used

- `fn tick()` drives animations once per second
- `ui.<name>.set_text("...")` updates labels (colors baked in at declaration)
- Dual-label toggle (green-hidden + red-shown, swap via `""` text) for
  color-change effects that `set_text` cannot express
- Monospace column alignment via `theme.font_code`
- Stage-gated animations (`if stage == N { ... }`) to sequence related motion
