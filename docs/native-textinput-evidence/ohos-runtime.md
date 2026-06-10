Status: DEFERRED
RunItem: makepad-example-native-text-input-ohos
Example: examples/native_text_input
Verified: install launch module_init xcomponent_surface egl_context
BuildId: local-cli (Studio bridge not used for this bring-up pass)
Device: OpenHarmony local simulator (127.0.0.1:5555, aarch64, API 15, emulator)
Transcript: docs/native-textinput-evidence/ohos-runtime.log
Command: tools/ohos_sim_sign_run.sh (after cargo run -p cargo-makepad -- ohos --deveco-home="$DEVECO_HOME" build -p makepad-example-native-text-input --release)

DevEcoHome: /Applications/DevEco-Studio.app/Contents
JavaHome: /Applications/DevEco-Studio.app/Contents/jbr/Contents/Home
HdcTargets: 127.0.0.1:5555
Hap: target/makepad-open-harmony/makepad_example_native_text_input/entry/build/default/outputs/default/makepad-default-signed.hap 62413599 sha256:734de4a0655a4bfa...
LibMakepad: target/makepad-open-harmony/makepad_example_native_text_input/entry/libs/arm64-v8a/libmakepad.so 13545160 sha256:69809dec5166686f...
Screenshot: docs/native-textinput-evidence/ohos-first-run.jpeg (black Makepad surface; see blocker)

LayerTrace:
  RustOp: DEFERRED (render thread dies before the first widget draw completes;
    no native view ops are emitted)
  OhosPlatformHandler: DEFERRED
  ArkTsHostMethod: DEFERRED
  ArkTsState: DEFERRED
  ArkUiVisual: DEFERRED
  ArkTsCallback: DEFERRED
  RustAction: DEFERRED
  MakepadStatusLabel: DEFERRED
LayoutTrace: none (blocked before layout ops)
CommandTrace: none (blocked before commands)

## First Concrete Blocker

The emulator's GPU stack cannot compile Makepad's GLES3 shaders. The guest
driver reports:

```
GL_VENDOR=ARM GL_RENDERER=Mali-G77 GL_VERSION=OpenGL ES 3.0 (4.1 Metal - 88.1)
GLSL_VERSION=OpenGL ES GLSL ES 3.0
```

The "(4.1 Metal - 88.1)" suffix is the macOS host OpenGL-on-Metal driver: the
emulator translates guest GLES to host desktop GL. Compiling Makepad's vertex
shader (source starts `#version 300 es`, dumped in the transcript) fails with
host-side desktop GLSL errors:

```
ERROR: 0:1: '' :  version '450' is not supported
ERROR: 0:2: '' :  #version required and missing.
ERROR: 0:77: ';' : syntax error: Uniform blocks may not have identifiers in GLSL 140
```

The translation layer emits desktop GLSL 450, but the macOS host OpenGL caps at
4.1 (GLSL 410), so every Makepad shader fails to compile. `opengl.rs:1517`
panics on compile failure, the render thread dies ~170ms after
`entry main_loop`, the widget tree never completes its first draw, and no
native text input ops reach the ArkTS host. This blocks the entire LayerTrace
above. The same failure occurs with and without the `MAKEPAD=ohos_sim` cfg.

Layers proven working before the blocker (all with hilog markers in the
transcript):

1. HAP signing with SDK community material, install, launch (`bm install` ok
   after adding `ohos.permission.READ_PASTEBOARD` to the profile ACL).
2. `libmakepad.so` NAPI module init, `makepad.onCreate(ArkGlue)`.
3. XComponent (libraryname `makepad`) delivers `__NATIVE_XCOMPONENT_OBJ__`;
   Rust registers callbacks; `OnSurfaceCreated` 1260x2503, density 3.25.
4. EGL context + window surface creation, vsync callback registration,
   `entry main_loop`.

## Defects Found and Fixed During This Bring-up

1. `Index.ets` XComponent `libraryname: 'entry'` pointed at a nonexistent
   libentry.so, so the XComponent never delivered the surface to Rust (black
   screen, zero logs). Fixed to `'makepad'`.
2. The OHOS entry path never called `Cx::init_log()` (desktop/Android/wasm do),
   so all Rust-side `crate::log!` output was silently dropped. Fixed in
   `app_main.rs`; this is what made every later layer observable.
3. `bm install` error 9568289: `READ_PASTEBOARD` is a system_basic ACL
   permission and must be listed in the provision profile `acls.allowed-acls`.
   Handled by `tools/ohos_sim_sign_run.sh`.
4. cargo-makepad's `run` requires `makepad-default-signed.hap` but the
   generated project has no signingConfigs. `tools/ohos_sim_sign_run.sh` signs
   with the SDK's OpenHarmony community material (profile rewritten with the
   real bundle name, the simulator UDID, fresh validity, and the ACL above).

## Next Options for Unblocking the Simulator Slice

- Run the same gate on a real OpenHarmony/HarmonyOS device (device GPU compiles
  GLES3 directly; this is already the P5 device gate).
- Investigate emulator GPU/render settings or a newer emulator image whose
  translation accepts GLES3 shaders.
- Out of slice scope: a Makepad GL shader emission variant compatible with the
  emulator's GLSL-450-over-GL4.1 translation, or a Vulkan backend path.
