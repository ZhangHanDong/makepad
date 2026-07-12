// Octos agent backend.
//
// Posts to an octos-bus `/chat` endpoint and consumes its SSE reply. Unlike
// the OpenAI-style backends this is NOT an LLM API: octos is an agent host, so
// the app is a thin client. The request is `{message, session_id, thread_id}`
// and the response is a `text/event-stream` of data-only frames (no `event:`
// lines) discriminated by a JSON `"type"`:
//   - `replace` : full snapshot of the assistant text so far (`text` field)
//   - `token`   : incremental delta to append (`text` field)
//   - `done`    : end of turn (carries model/usage)
//   - others (`thinking`, `cost_update`, `response`, ...) are ignored
// See docs from the octos /chat protocol probe.

use std::collections::HashMap;

use makepad_widgets::*;

use crate::backend::{AiBackend, AiEvent, BackendConfig, RequestId};
use crate::types::*;

struct InFlightRequest {
    request_id: RequestId,
    // Current assistant text. `replace` overwrites it; `token` appends.
    text: String,
    saw_done: bool,
    // SSE frames split on `\n\n`; chunks arrive at arbitrary byte boundaries,
    // so keep the trailing partial frame for the next chunk.
    sse_buffer: String,
}

pub struct OctosBackend {
    config: BackendConfig,
    in_flight: HashMap<LiveId, InFlightRequest>,
    // octos requires a thread_id per turn; allocate a stable session and a
    // fresh thread per request.
    session_id: String,
    next_thread: u64,
}

impl OctosBackend {
    pub fn new(config: BackendConfig) -> Self {
        let started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Self {
            config,
            in_flight: HashMap::new(),
            session_id: format!("aichat-{}-{}", started_at, LiveId::unique().0),
            next_thread: 0,
        }
    }

    fn escape_json_string(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
                c => result.push(c),
            }
        }
        result
    }

    fn latest_user_text(request: &AiRequest) -> String {
        // octos owns the conversation; send only the latest user message.
        for msg in request.messages.iter().rev() {
            if matches!(msg.role, MessageRole::User) {
                let mut text = String::new();
                for block in &msg.content {
                    if let ContentBlock::Text { text: t } = block {
                        text.push_str(t);
                    }
                }
                if !text.trim().is_empty() {
                    return text;
                }
            }
        }
        String::new()
    }

    fn build_http_request(&mut self, request: &AiRequest) -> HttpRequest {
        let BackendConfig::Octos {
            base_url,
            auth_token,
        } = &self.config
        else {
            panic!("OctosBackend requires Octos config");
        };

        let thread_id = format!("{}-t{}", self.session_id, self.next_thread);
        self.next_thread += 1;

        let url = if base_url.ends_with("/chat") {
            base_url.clone()
        } else {
            format!("{}/chat", base_url.trim_end_matches('/'))
        };

        let message = Self::latest_user_text(request);
        log!(
            "Octos request: url={} session={} thread={} message_chars={}",
            url,
            self.session_id,
            thread_id,
            message.chars().count()
        );

        let body = format!(
            "{{\"message\":\"{}\",\"session_id\":\"{}\",\"thread_id\":\"{}\"}}",
            Self::escape_json_string(&message),
            Self::escape_json_string(&self.session_id),
            Self::escape_json_string(&thread_id),
        );

        let mut http = HttpRequest::new(url, HttpMethod::POST);
        http.set_is_streaming();
        http.set_header("Content-Type".to_string(), "application/json".to_string());
        if let Some(token) = auth_token {
            if !token.trim().is_empty() {
                http.set_header("Authorization".to_string(), format!("Bearer {}", token));
            }
        }
        http.set_string_body(body);
        http
    }

    // Pull the string value of a top-level JSON key out of one SSE frame.
    // octos frames are flat objects, so a minimal scanner avoids a JSON dep.
    fn json_str_field<'a>(obj: &'a str, key: &str) -> Option<String> {
        let needle = format!("\"{}\"", key);
        let mut idx = obj.find(&needle)? + needle.len();
        let bytes = obj.as_bytes();
        while idx < bytes.len() && bytes[idx] != b':' {
            idx += 1;
        }
        idx += 1;
        while idx < bytes.len() && (bytes[idx] as char).is_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || bytes[idx] != b'"' {
            return None;
        }
        idx += 1;
        let mut out = String::new();
        let chars: Vec<char> = obj[idx..].chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '\\' && i + 1 < chars.len() {
                let n = chars[i + 1];
                match n {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        if i + 5 < chars.len() {
                            let hex: String = chars[i + 2..i + 6].iter().collect();
                            if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                                if let Some(ch) = char::from_u32(cp) {
                                    out.push(ch);
                                }
                            }
                            i += 6;
                            continue;
                        }
                    }
                    other => out.push(other),
                }
                i += 2;
                continue;
            }
            if c == '"' {
                return Some(out);
            }
            out.push(c);
            i += 1;
        }
        None
    }

    fn process_stream_data(
        &mut self,
        request_id: LiveId,
        data: &str,
        is_complete: bool,
    ) -> Vec<AiEvent> {
        let events = vec![];
        let Some(in_flight) = self.in_flight.get_mut(&request_id) else {
            return events;
        };
        in_flight.sse_buffer.push_str(data);

        // Process whole `\n\n`-terminated SSE frames; keep the trailing partial.
        loop {
            let Some(sep) = in_flight.sse_buffer.find("\n\n") else {
                break;
            };
            let frame: String = in_flight.sse_buffer.drain(..sep + 2).collect();
            for line in frame.lines() {
                let line = line.trim();
                let Some(payload) = line.strip_prefix("data:") else {
                    continue;
                };
                let payload = payload.trim();
                if payload.is_empty() {
                    continue;
                }
                let ty = Self::json_str_field(payload, "type").unwrap_or_default();
                match ty.as_str() {
                    "replace" => {
                        if let Some(text) = Self::json_str_field(payload, "text") {
                            in_flight.text = text;
                        }
                    }
                    "token" => {
                        if let Some(text) = Self::json_str_field(payload, "text") {
                            in_flight.text.push_str(&text);
                        }
                    }
                    "done" => {
                        in_flight.saw_done = true;
                    }
                    _ => {}
                }
            }
        }

        if is_complete {
            in_flight.saw_done = true;
        }
        events
    }
}

impl AiBackend for OctosBackend {
    fn send_request(&mut self, cx: &mut Cx, request: AiRequest) -> RequestId {
        let request_id = RequestId::new();
        let http = self.build_http_request(&request);
        self.in_flight.insert(
            request_id.0,
            InFlightRequest {
                request_id,
                text: String::new(),
                saw_done: false,
                sse_buffer: String::new(),
            },
        );
        cx.http_request(request_id.0, http);
        request_id
    }

    fn cancel_request(&mut self, cx: &mut Cx, request_id: RequestId) {
        if self.in_flight.remove(&request_id.0).is_some() {
            cx.cancel_http_request(request_id.0);
        }
    }

    fn handle_event(&mut self, _cx: &mut Cx, event: &Event) -> Vec<AiEvent> {
        let mut ai_events = vec![];
        let Event::NetworkResponses(responses) = event else {
            return ai_events;
        };
        for response in responses {
            let request_id = match response {
                NetworkResponse::HttpResponse { request_id, .. }
                | NetworkResponse::HttpStreamChunk { request_id, .. }
                | NetworkResponse::HttpStreamComplete { request_id, .. }
                | NetworkResponse::HttpError { request_id, .. }
                | NetworkResponse::HttpProgress { request_id, .. } => *request_id,
                NetworkResponse::WsOpened { .. }
                | NetworkResponse::WsMessage { .. }
                | NetworkResponse::WsClosed { .. }
                | NetworkResponse::WsError { .. } => continue,
            };
            if !self.in_flight.contains_key(&request_id) {
                continue;
            }
            match response {
                NetworkResponse::HttpStreamChunk { response: res, .. } => {
                    if let Some(data) = res.get_string_body() {
                        ai_events.extend(self.process_stream_data(request_id, &data, false));
                    }
                }
                // Non-streaming fallback: some paths deliver the whole body at once.
                NetworkResponse::HttpResponse { response: res, .. } => {
                    if let Some(data) = res.get_string_body() {
                        ai_events.extend(self.process_stream_data(request_id, &data, true));
                    }
                }
                NetworkResponse::HttpStreamComplete { .. } => {
                    ai_events.extend(self.process_stream_data(request_id, "", true));
                    if let Some(in_flight) = self.in_flight.remove(&request_id) {
                        log!(
                            "Octos stream complete: content_chars={} saw_done={} leftover={}",
                            in_flight.text.chars().count(),
                            in_flight.saw_done,
                            in_flight.sse_buffer.chars().count()
                        );
                        let content_blocks = if in_flight.text.is_empty() {
                            vec![]
                        } else {
                            ai_events.push(AiEvent::StreamDelta {
                                request_id: in_flight.request_id,
                                delta: StreamDelta::TextDelta {
                                    text: in_flight.text.clone(),
                                },
                            });
                            vec![ContentBlock::Text {
                                text: in_flight.text,
                            }]
                        };
                        ai_events.push(AiEvent::Complete {
                            request_id: in_flight.request_id,
                            response: AiResponse {
                                message: Message {
                                    role: MessageRole::Assistant,
                                    content: content_blocks,
                                },
                                stop_reason: StopReason::EndTurn,
                                usage: Usage::default(),
                            },
                        });
                    }
                }
                NetworkResponse::HttpError { error: err, .. } => {
                    if let Some(in_flight) = self.in_flight.remove(&request_id) {
                        ai_events.push(AiEvent::Error {
                            request_id: in_flight.request_id,
                            error: err.message.clone(),
                        });
                    }
                }
                NetworkResponse::HttpProgress { .. }
                | NetworkResponse::WsOpened { .. }
                | NetworkResponse::WsMessage { .. }
                | NetworkResponse::WsClosed { .. }
                | NetworkResponse::WsError { .. } => {}
            }
        }
        ai_events
    }

    fn config(&self) -> &BackendConfig {
        &self.config
    }
}
