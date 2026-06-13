Status: PASS
Scope: aichat on OHOS simulator with a real streaming LLM (agent2app, plan A)
Example: examples/aichat
Device: OpenHarmony local simulator (127.0.0.1:5555, aarch64, API 15, emulator)
Command: MAKEPAD=ohos_sim cargo run -p cargo-makepad -- ohos --deveco-home="$DEVECO_HOME" build -p makepad-example-aichat --release && tools/ohos_sim_sign_run.sh -p makepad-example-aichat
Transcript: docs/native-textinput-evidence/ohos-aichat-llm.log (scrubbed of key material)

## Verified Chain (hilog markers)

```
aichat: loaded 1 secrets from rawfile
aichat: backends after re-detect: [Moonshot]
aichat: smoke prompt sending: "Reply with exactly the three words: makepad on harmony"
[MakepadNet] httpStart id=4 method=POST url=https://api.moonshot.ai/v1/chat/completions body_len=11876
[MakepadNet] headersReceive -> dataReceive len=1488 -> dataEnd -> response code=200 -> complete
OpenAI stream content delta chars=3 / chars=8 ... finish_reason=stop ... [DONE]
OpenAI stream complete: content_chars=18 finish_reason=Some("stop") saw_done=true
aichat UI text delta chars=... (per token)
aichat UI turn complete content_chars=18
```

The model returned exactly the requested reply (18 chars), proving the full
stack end to end on a screen-less OHOS target:

1. OHOS aichat runtime boots and stays alive (GL shaders skipped under
   `ohos_sim` tolerant mode; UI is blank but logic runs).
2. Bundled-config delivery: secrets ride in a HAP `resources/rawfile/`
   `aichat_secrets.env` (the only app-readable path — the sandbox cannot see
   env vars, CWD, hdc-pushed `/data/local/tmp`, and root is unavailable). The
   new `Cx::read_ohos_rawfile` reads it; aichat populates an in-memory key
   map and re-detects backends → Moonshot.
3. The OpenHarmony network backend (the prior slice) carries a real POST to
   `api.moonshot.ai/v1/chat/completions` over system TLS, streaming the SSE
   response back.
4. makepad_ai `OpenAiBackend` (which uses `cx.http_request`) parses the SSE
   token deltas; aichat applies them as UI text deltas and completes the turn.

## OHOS Sandbox Config-Delivery Findings

The app sandbox is fully isolated. Each was tested and ruled out:

- `/data/storage/el2/base/haps/makepad/files` (cx.get_data_dir): exists but
  not writable from hdc (`no such file or directory` for the sandbox path;
  `permission denied` for the real `/data/app/...` path owned by the app uid).
- `/data/local/tmp`: hdc can write it, but the app sandbox cannot see it
  (`No such file or directory` from inside the app).
- Rooting the simulator: blocked (`Cannot set root run mode in undebuggable
  version`).
- Therefore: bundle config into the HAP `resources/rawfile/`, read via
  `Cx::read_ohos_rawfile`. This is also the real-device answer for shipping
  app config to an OHOS makepad app.

## Security Note

`aichat_secrets.env` (with the real key) was written only under
`target/makepad-open-harmony/.../resources/rawfile/`, which is gitignored and
never committed. The saved hilog transcript was scrubbed of all key material
(`sk-`, Authorization) before being added. For production, the key would be
provisioned per device, not baked into a shared build.

## Remaining

- Plan B: a native Octos backend in aichat (POST /api/chat + SSE) so the app
  talks to the local octos agent instead of an upstream LLM directly — the
  true agent2app topology (octos = agent host, aichat = OHOS app).
- Real-device run for the visible aichat UI (Makepad self-render), IME for
  the prompt input, and clipboard.
