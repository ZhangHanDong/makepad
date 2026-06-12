Status: PASS
Scope: OpenHarmony network backend (HTTP/HTTPS, streaming-capable)
Example: examples/native_text_input (startup HTTPS smoke)
Device: OpenHarmony local simulator (127.0.0.1:5555, aarch64, API 15, emulator)
Command: MAKEPAD=ohos_sim cargo run -p cargo-makepad -- ohos --deveco-home="$DEVECO_HOME" build -p makepad-example-native-text-input --release && tools/ohos_sim_sign_run.sh

## Verified Chain (hilog markers)

```
cx.http_request(live_id!(ohos_http_smoke), GET https://example.com/)
  -> OhosNetworkBackend::http_start (caller thread)
  -> send_from_ohos_message(HttpRequestStart) -> Makepad main loop
  -> ArkTS bridge httpStart(id, method, url, headers, body: ArrayBuffer)
  -> @ohos.net.http requestInStream (system TLS)
  -> [MakepadNet] headersReceive -> dataReceive len=559 -> dataEnd -> code=200
  -> NAPI handle_http_headers/chunk/complete -> EventSink
  -> dispatch_network_runtime_events (main loop)
  -> example: "http response status=200 body_len=559"
```

## Defects Found and Fixed During Bring-up

1. `send_from_ohos_message` used a thread_local-only sender; the first caller
   from a non-initialized thread (the network backend) hit
   `Option::unwrap()` on None and aborted the render thread. Fixed with a
   global sender fallback (thread_local stays the fast path) and the unwraps
   replaced with non-panicking error logs.
2. The OHOS main loop never drained the network runtime channel; responses
   were emitted but never dispatched. Added
   `dispatch_network_runtime_events()` to `handle_other_events`, matching the
   other platforms.
3. Observed @ohos.net.http ordering: headersReceive -> dataReceive -> dataEnd
   -> response-code callback (last). Completion now waits for both dataEnd
   and the status code, otherwise the request finished with status 0 and the
   late headers were dropped (first run recorded exactly that:
   "status=0 body_len=559").

## Streaming (SSE) Verification

`tools/ohos_sse_test_server.py` on the host (reachable from the emulator at
10.0.2.2:8765) serves five SSE events one second apart. The example's
streaming smoke (`is_streaming = true`) received each chunk in real time —
hilog timestamps match the server's 1s cadence (.856/.714/.720/.713/.720),
proving per-chunk delivery rather than end-of-request buffering:

```
sse chunk status=0 len=24 text="data: ohos sse event 0..4"  (5 chunks, 1s apart)
sse stream complete
```

The concurrent aggregate HTTPS smoke completed with status=200 in the same
run. Chunks carry status 0 because @ohos.net.http delivers the response code
callback last (see ordering note above); SSE consumers read the body, and
this matches the Linux backend chunk semantics.

## Notes

- Streaming chunks may carry status_code 0 when they arrive before the
  response-code callback; SSE consumers read the body so this matches the
  Linux backend behavior (chunks: status + empty headers; complete: full
  headers).
- WebSocket is intentionally `NetworkError::Unsupported` in this slice; the
  NDK `OH_WebSocketClient` C API is the planned wss path, `ws://` can use the
  existing PlainWebSocket.
- `ohos.permission.INTERNET` added to module.json5 (normal level, no ACL).
- This unblocks aichat/agent2app on OHOS: the example crate compiles for the
  OHOS target and the LLM/SSE network path now works end to end on the
  simulator.
