Status: PASS
Scope: agent2app — OHOS aichat (app) talks to a local octos agent host (plan B)
Example: examples/aichat with BackendType::Octos
Device: OpenHarmony local simulator (127.0.0.1:5555, aarch64, API 15, emulator)
Transcript: docs/native-textinput-evidence/ohos-aichat-octos.log (scrubbed)

## Topology

```
OHOS aichat (emulator)
  -> OctosBackend (cx.http_request -> OHOS network backend)
  -> http://10.0.2.2:19401/chat        (tools/ohos_tcp_forward.py)
  -> host 127.0.0.1:9401  octos-bus /chat
  -> octos agent -> Moonshot (kimi)
  -> octos SSE (replace/token/done) streamed back the same path
  -> OctosBackend parses deltas -> aichat UI turn
```

This is the real agent2app shape: the OHOS app is a thin client of an octos
agent host, not a direct LLM caller. octos owns the session, model, and
tools; aichat just sends `{message, session_id, thread_id}` and renders the
streamed reply.

## Verified Chain (hilog markers)

```
aichat: backends after re-detect: [Octos]
aichat: smoke prompt sending: "Reply with exactly the three words: octos on harmony"
Octos request: url=http://10.0.2.2:19401/chat session=aichat-1 thread=aichat-1-t0 message_chars=253
[MakepadNet] httpStart id=5 method=POST url=http://10.0.2.2:19401/chat
[MakepadNet] dataReceive id=5 len=...   (multiple octos SSE frames)
aichat UI text delta chars=3 / 3 / 13   (token deltas applied to the UI)
[MakepadNet] dataEnd -> response code=200 -> complete
Octos stream complete: content_chars=16 saw_done=true leftover=0
aichat UI turn complete content_chars=58
```

`content_chars=16` is the assistant answer ("octos on harmony" = 16 chars),
matching the prompt. (`turn complete content_chars=58` counts the octos
"replace" preamble snapshot `Synthesizing via moonshot...` plus the answer;
the final assistant text is the 16-char reply.)

## What this proves

1. The new `OctosBackend` (makepad_ai) correctly speaks the octos-bus `/chat`
   protocol: data-only SSE frames keyed by JSON `type`, with `replace`
   snapshots emitting only the new suffix and `token` frames appended.
2. It rides the OHOS network backend transparently (OctosBackend uses
   `cx.http_request` just like OpenAiBackend).
3. aichat's `BackendType::Octos` selection, rawfile config
   (`OCTOS_BASE_URL`), and the host TCP forwarder all work together so the
   emulator reaches a 127.0.0.1-bound octos.

## Notes

- octos on this instance has no auth (synthetic api channel, empty settings);
  `OCTOS_AUTH_TOKEN` is wired but unused here.
- `tools/ohos_tcp_forward.py 19401 9401` bridges the emulator (10.0.2.2) to
  octos (127.0.0.1:9401); start it on the host before running.
- UI is blank on this emulator (GLES3-on-Metal limitation); verified via logs.
  Real-device / desktop shows the visible chat.
