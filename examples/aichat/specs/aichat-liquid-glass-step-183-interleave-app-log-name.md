# Step 183 - Interleave App Log Name

Date: 2026-05-12

## Context

Step 182 made `AppleNativeInterleave` directly runnable behind an explicit
guard, but the app-side native success log still reported:

```text
app-substrate=apple-native-underlay
```

That was technically understandable because the platform installer still reuses
the native underlay descriptor path, but it was misleading for the guarded
interleave route.

## Implementation

The app now derives the user-facing native substrate log name from both:

- whether native glass is active
- whether the guarded interleave route and lower-scene pass are enabled

When both are true, the app-side success log uses:

```text
app-substrate=apple-native-interleave
```

The lower platform/widget logs still use `apple-native-underlay` where they are
describing the shared native panel installer.

## Verification

Commands:

```text
cargo test -p makepad-example-aichat --release aichat_native_app_substrate_log_names_interleave_guard
cargo test -p makepad-example-aichat --release aichat_apple_native_interleave
cargo check -p makepad-example-aichat --release
```

Result:

```text
1 passed; 0 failed
4 passed; 0 failed
cargo check finished release profile
```

Studio release build:

```text
build_id=[37]
```

Key logs:

```text
[liquid-glass] AppleNativeInterleave guarded production preview enabled; manual visual validation still required
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

## Verdict

The guarded interleave route now has honest app-facing observability. It still
requires manual visual validation before it can be promoted beyond guarded
preview status.
