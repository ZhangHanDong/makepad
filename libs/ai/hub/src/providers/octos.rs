//! Octos agent-host chat provider.
//!
//! octos is an agent host, not an LLM API: the app is a thin client. One
//! turn is `POST {base}/chat` with `{message, session_id, thread_id}` and a
//! `text/event-stream` reply of data-only frames (no `event:` lines)
//! discriminated by a JSON `"type"`:
//!   - `replace` : full snapshot of the assistant text so far (`text`)
//!   - `token`   : incremental delta to append (`text`)
//!   - `done`    : end of turn (carries model/usage, ignored here)
//!   - others (`thinking`, `cost_update`, `response`, ...) are ignored.
//!
//! The host keeps the conversation, so only the LATEST user message of the
//! turn is sent, with the turn's system block folded in front of it — the
//! octos wire has no system-prompt field. The transport is the platform's
//! dependency-free blocking HTTP client on a detached worker; the bounded
//! SSE body is parsed once it completes and surfaces as one `Delta` and a
//! `Done`, the same shape the Claude API provider uses. Tests replace only
//! the transport.

use crate::chat_wire::{sanitize_public_error, ChatRole, ProviderAvailability, ProviderKind};
use crate::providers::provider::{ChatProvider, ProviderEvent, TurnInput};
use makepad_network::blocking_http::{
    post_json, CancelToken, Error as HttpError, Limits, Request,
};
use makepad_strict_json::{self as json, Value};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::time::Duration;

/// Overrides the octos base URL (`http://host:port`, `/chat` is appended).
pub const OCTOS_URL_ENV: &str = "MAKEPAD_OCTOS_URL";
/// Optional bearer token sent as `Authorization: Bearer <token>`.
pub const OCTOS_TOKEN_ENV: &str = "MAKEPAD_OCTOS_TOKEN";
/// Agent turns can run tools before answering; give them room.
pub const DEFAULT_OCTOS_TIMEOUT: Duration = Duration::from_secs(300);
/// `replace` frames re-send the whole answer each time, so the SSE body
/// grows with the square of the answer length. 8 MB covers a long
/// generated app with headroom and still refuses a runaway stream.
pub const MAX_OCTOS_RESPONSE_BODY: usize = 8 * 1024 * 1024;
/// The request carries the folded system block (a whole scripting manual
/// for the Splash apps) plus the user message.
pub const MAX_OCTOS_REQUEST_BODY: usize = 1024 * 1024;
pub const OCTOS_MODEL_LABEL: &str = "octos";

const MAX_SESSION_ID_BYTES: usize = 128;

/// Where the octos host is and how to talk to it. The base URL is
/// normalized to the `/chat` endpoint at construction.
#[derive(Clone)]
pub struct OctosConfig {
    endpoint: String,
    auth_token: Option<String>,
    request_timeout: Duration,
    session_id: String,
}

impl OctosConfig {
    /// `base_url` is `http://host:port` or `http://host:port/chat`.
    pub fn new(base_url: impl Into<String>) -> Result<Self, String> {
        let base_url: String = base_url.into();
        let trimmed = base_url.trim();
        if trimmed.is_empty() {
            return Err("octos url is empty".to_string());
        }
        if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
            return Err("octos url must start with http:// or https://".to_string());
        }
        let endpoint = if trimmed.ends_with("/chat") {
            trimmed.to_string()
        } else {
            format!("{}/chat", trimmed.trim_end_matches('/'))
        };
        Ok(OctosConfig {
            endpoint,
            auth_token: None,
            request_timeout: DEFAULT_OCTOS_TIMEOUT,
            session_id: fresh_session_id(),
        })
    }

    /// `MAKEPAD_OCTOS_URL` / `MAKEPAD_OCTOS_TOKEN`, falling back to
    /// `default_url` when the URL variable is unset or empty.
    pub fn from_env(default_url: &str) -> Result<Self, String> {
        let url = std::env::var(OCTOS_URL_ENV)
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| default_url.to_string());
        let mut config = OctosConfig::new(url)?;
        if let Some(token) = std::env::var(OCTOS_TOKEN_ENV)
            .ok()
            .filter(|v| !v.trim().is_empty())
        {
            config = config.with_auth_token(Some(token));
        }
        Ok(config)
    }

    pub fn with_auth_token(mut self, token: Option<String>) -> Self {
        self.auth_token = token.filter(|t| !t.trim().is_empty());
        self
    }

    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Pins the octos session id (one conversation on the host). Defaults
    /// to a fresh per-process id.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        let session_id: String = session_id.into();
        if !session_id.is_empty() && session_id.len() <= MAX_SESSION_ID_BYTES {
            self.session_id = session_id;
        }
        self
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

fn fresh_session_id() -> String {
    let started_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("aichat-{}-{}", started_at, std::process::id())
}

/// HTTP seam for deterministic tests. Implementations must not include the
/// token, request body, or response body in errors.
pub trait OctosTransport: Send + Sync + 'static {
    fn post_chat(
        &self,
        url: &str,
        auth_token: Option<&str>,
        body: &[u8],
        cancel: &CancelToken,
        timeout: Duration,
    ) -> Result<OctosRawHttp, String>;
}

pub struct OctosRawHttp {
    pub status: u16,
    pub body: Vec<u8>,
}

pub struct BlockingOctosTransport;

impl OctosTransport for BlockingOctosTransport {
    fn post_chat(
        &self,
        url: &str,
        auth_token: Option<&str>,
        body: &[u8],
        cancel: &CancelToken,
        timeout: Duration,
    ) -> Result<OctosRawHttp, String> {
        let request = Request::post(url)
            .header("accept", "text/event-stream")
            .map_err(safe_http_err)?;
        let request = match auth_token {
            Some(token) => request.bearer(token).map_err(safe_http_err)?,
            None => request,
        };
        let request = request
            .json_body(body.to_vec())
            .map_err(safe_http_err)?
            .cancel_token(cancel.clone())
            .limits(Limits {
                max_body_bytes: MAX_OCTOS_RESPONSE_BODY,
                total_timeout: timeout,
                ..Limits::default()
            });
        match post_json(request) {
            Ok(response) => Ok(OctosRawHttp {
                status: response.status,
                body: response.body,
            }),
            Err(error) => Err(safe_http_err(error)),
        }
    }
}

fn safe_http_err(error: HttpError) -> String {
    error.to_string()
}

// ------------------------------------------------------------------ parsing

/// What one completed octos SSE body said.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParsedOctosStream {
    /// The assistant text after the last `replace`/`token` frame.
    pub text: String,
    /// Whether a `done` frame arrived. Its absence is logged by callers as
    /// a truncated stream; the text is still delivered.
    pub saw_done: bool,
}

/// Parses a complete `text/event-stream` body of octos frames. Frames are
/// `\n\n`-separated (CRLF tolerated); only `data:` lines count. A frame
/// that is not JSON is skipped rather than failing the turn, matching the
/// lenient client octos was built against.
pub fn parse_octos_stream(bytes: &[u8]) -> ParsedOctosStream {
    let body = String::from_utf8_lossy(bytes);
    let mut parsed = ParsedOctosStream::default();
    for frame in body.replace("\r\n", "\n").split("\n\n") {
        for line in frame.lines() {
            let Some(payload) = line.trim().strip_prefix("data:") else {
                continue;
            };
            let payload = payload.trim();
            if payload.is_empty() {
                continue;
            }
            let Ok(value) = json::parse(payload.as_bytes()) else {
                continue;
            };
            match value.get("type").and_then(Value::as_str) {
                Some("replace") => {
                    if let Some(text) = value.get("text").and_then(Value::as_str) {
                        parsed.text = text.to_string();
                    }
                }
                Some("token") => {
                    if let Some(text) = value.get("text").and_then(Value::as_str) {
                        parsed.text.push_str(text);
                    }
                }
                Some("done") => parsed.saw_done = true,
                _ => {}
            }
        }
    }
    parsed
}

/// The single message a turn sends: the latest user text, headed by the
/// system block (octos has nowhere else to put it). `None` when the turn
/// carries no user message.
pub fn fold_turn_message(input: &TurnInput) -> Option<String> {
    let user = input
        .messages
        .iter()
        .rev()
        .find(|m| m.role == ChatRole::User && !m.text.trim().is_empty())
        .map(|m| m.text.as_str())?;
    let system = input.system_with_dynamic();
    if system.trim().is_empty() {
        Some(user.to_string())
    } else {
        Some(format!("{}\n\nUSER REQUEST:\n{}", system, user))
    }
}

fn build_request_body(message: &str, session_id: &str, thread_id: &str) -> Vec<u8> {
    json::obj(vec![
        ("message", json::s(message)),
        ("session_id", json::s(session_id)),
        ("thread_id", json::s(thread_id)),
    ])
    .to_json()
    .into_bytes()
}

fn http_status_error(status: u16, body: &[u8]) -> String {
    let snippet = String::from_utf8_lossy(&body[..body.len().min(160)]);
    let snippet = snippet.trim();
    if snippet.is_empty() {
        format!("octos returned HTTP {status}")
    } else {
        sanitize_public_error(&format!("octos returned HTTP {status}: {snippet}"))
    }
}

fn endpoint_detail(url: &str) -> String {
    url.trim_end_matches("/chat").to_string()
}

// ------------------------------------------------------------------ provider

enum WorkerOut {
    Ok(ParsedOctosStream),
    Err(String),
}

pub struct OctosChatProvider<T: OctosTransport = BlockingOctosTransport> {
    config: OctosConfig,
    transport: Arc<T>,
    cancel: CancelToken,
    active: Option<Receiver<WorkerOut>>,
    next_thread: u64,
}

impl<T: OctosTransport> OctosChatProvider<T> {
    pub fn with_transport(config: OctosConfig, transport: T) -> Self {
        OctosChatProvider {
            config,
            transport: Arc::new(transport),
            cancel: CancelToken::new(),
            active: None,
            next_thread: 0,
        }
    }

    pub fn config(&self) -> &OctosConfig {
        &self.config
    }
}

impl OctosChatProvider<BlockingOctosTransport> {
    pub fn new(config: OctosConfig) -> Self {
        OctosChatProvider::with_transport(config, BlockingOctosTransport)
    }

    /// Env-configured provider; see [`OctosConfig::from_env`].
    pub fn from_env(default_url: &str) -> Result<Self, String> {
        Ok(OctosChatProvider::new(OctosConfig::from_env(default_url)?))
    }
}

impl<T: OctosTransport> ChatProvider for OctosChatProvider<T> {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Octos
    }

    fn availability(&mut self) -> ProviderAvailability {
        // The host is only known to be up once a turn succeeds; a live
        // probe per send would double every turn's round trips on a LAN
        // box, so availability reports configuration, like the API-key
        // providers do, and a dead host surfaces as a turn error.
        ProviderAvailability::Available {
            model: OCTOS_MODEL_LABEL.to_string(),
            detail: endpoint_detail(&self.config.endpoint),
        }
    }

    fn begin_turn(&mut self, input: &TurnInput) -> Result<(), String> {
        if self.active.is_some() {
            return Err("a turn is already in flight".to_string());
        }
        let message = fold_turn_message(input).ok_or_else(|| "no user message".to_string())?;
        let thread_id = format!("{}-t{}", self.config.session_id, self.next_thread);
        self.next_thread += 1;
        let body = build_request_body(&message, &self.config.session_id, &thread_id);
        if body.len() > MAX_OCTOS_REQUEST_BODY {
            return Err("request body too large".to_string());
        }
        let endpoint = self.config.endpoint.clone();
        let auth_token = self.config.auth_token.clone();
        let timeout = self.config.request_timeout;
        let cancel = self.cancel.clone();
        let transport = self.transport.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("content-chat-octos".into())
            .spawn(move || {
                let response =
                    transport.post_chat(&endpoint, auth_token.as_deref(), &body, &cancel, timeout);
                if cancel.is_cancelled() {
                    return;
                }
                let result = match response {
                    Err(error) => {
                        let public = match &auth_token {
                            Some(token) if error.contains(token.as_str()) => {
                                "provider error".to_string()
                            }
                            _ => sanitize_public_error(&error),
                        };
                        WorkerOut::Err(public)
                    }
                    Ok(raw) if raw.body.len() > MAX_OCTOS_RESPONSE_BODY => {
                        WorkerOut::Err("response body too large".to_string())
                    }
                    Ok(raw) if !(200..300).contains(&raw.status) => {
                        WorkerOut::Err(http_status_error(raw.status, &raw.body))
                    }
                    Ok(raw) => WorkerOut::Ok(parse_octos_stream(&raw.body)),
                };
                if !cancel.is_cancelled() {
                    let _ = tx.send(result);
                }
            })
            .map_err(|_| "failed to start provider worker".to_string())?;
        self.active = Some(rx);
        Ok(())
    }

    fn poll(&mut self) -> Vec<ProviderEvent> {
        let Some(rx) = &self.active else {
            return Vec::new();
        };
        let out = match rx.try_recv() {
            Ok(out) => out,
            Err(mpsc::TryRecvError::Empty) => return Vec::new(),
            Err(mpsc::TryRecvError::Disconnected) => {
                WorkerOut::Err("provider worker ended".to_string())
            }
        };
        self.active = None;
        match out {
            WorkerOut::Ok(parsed) => {
                let mut events = Vec::with_capacity(2);
                if !parsed.text.is_empty() {
                    events.push(ProviderEvent::Delta(parsed.text.clone()));
                }
                events.push(ProviderEvent::Done { text: parsed.text });
                events
            }
            WorkerOut::Err(error) => vec![ProviderEvent::Error(error)],
        }
    }

    fn cancel(&mut self) {
        self.cancel.cancel();
        self.cancel = CancelToken::new();
        self.active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_wire::ChatMessage;
    use std::sync::Mutex;
    use std::time::Instant;

    const STREAM: &str = "data: {\"type\":\"thinking\",\"text\":\"hmm\"}\n\n\
data: {\"type\":\"replace\",\"text\":\"Hel\"}\n\n\
data: {\"type\":\"token\",\"text\":\"lo \"}\n\n\
data: not json\n\n\
data: {\"type\":\"replace\",\"text\":\"Hello w\"}\n\n\
data: {\"type\":\"token\",\"text\":\"orld\"}\n\n\
data: {\"type\":\"done\",\"model\":\"moonshot\"}\n\n";

    #[test]
    fn parses_replace_token_done_and_skips_the_rest() {
        let parsed = parse_octos_stream(STREAM.as_bytes());
        assert_eq!(parsed.text, "Hello world");
        assert!(parsed.saw_done);
    }

    #[test]
    fn tolerates_crlf_and_a_missing_done() {
        let body = "data: {\"type\":\"token\",\"text\":\"a\\nb\"}\r\n\r\ndata: {\"type\":\"token\",\"text\":\"c\"}";
        let parsed = parse_octos_stream(body.as_bytes());
        assert_eq!(parsed.text, "a\nbc");
        assert!(!parsed.saw_done);
    }

    #[test]
    fn config_normalizes_the_chat_endpoint() {
        assert_eq!(
            OctosConfig::new("http://10.0.0.5:19401/").unwrap().endpoint(),
            "http://10.0.0.5:19401/chat"
        );
        assert_eq!(
            OctosConfig::new("http://10.0.0.5:19401/chat").unwrap().endpoint(),
            "http://10.0.0.5:19401/chat"
        );
        assert!(OctosConfig::new("").is_err());
        assert!(OctosConfig::new("10.0.0.5:19401").is_err());
    }

    #[test]
    fn folds_system_in_front_of_the_latest_user_message() {
        let mut input = TurnInput::new(
            "SYSTEM",
            vec![
                ChatMessage::new(ChatRole::User, "first"),
                ChatMessage::new(ChatRole::Assistant, "reply"),
                ChatMessage::new(ChatRole::User, "second"),
            ],
        );
        assert_eq!(
            fold_turn_message(&input).as_deref(),
            Some("SYSTEM\n\nUSER REQUEST:\nsecond")
        );
        input.system.clear();
        assert_eq!(fold_turn_message(&input).as_deref(), Some("second"));
        input.messages.clear();
        assert!(fold_turn_message(&input).is_none());
    }

    struct FakeTransport {
        reply: Result<(u16, &'static str), String>,
        seen: Mutex<Vec<(String, Option<String>, String)>>,
    }

    impl OctosTransport for FakeTransport {
        fn post_chat(
            &self,
            url: &str,
            auth_token: Option<&str>,
            body: &[u8],
            _cancel: &CancelToken,
            _timeout: Duration,
        ) -> Result<OctosRawHttp, String> {
            self.seen.lock().unwrap().push((
                url.to_string(),
                auth_token.map(str::to_string),
                String::from_utf8_lossy(body).to_string(),
            ));
            match &self.reply {
                Ok((status, body)) => Ok(OctosRawHttp {
                    status: *status,
                    body: body.as_bytes().to_vec(),
                }),
                Err(e) => Err(e.clone()),
            }
        }
    }

    fn drain(p: &mut OctosChatProvider<FakeTransport>) -> Vec<ProviderEvent> {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut got = Vec::new();
        while Instant::now() < deadline {
            got.extend(p.poll());
            if got
                .iter()
                .any(|e| matches!(e, ProviderEvent::Done { .. } | ProviderEvent::Error(_)))
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        got
    }

    fn provider(reply: Result<(u16, &'static str), String>) -> OctosChatProvider<FakeTransport> {
        let config = OctosConfig::new("http://octos.test:19401")
            .unwrap()
            .with_auth_token(Some("secret-token".into()))
            .with_session_id("sess");
        OctosChatProvider::with_transport(
            config,
            FakeTransport {
                reply,
                seen: Mutex::new(Vec::new()),
            },
        )
    }

    #[test]
    fn a_turn_posts_one_folded_message_and_yields_delta_then_done() {
        let mut p = provider(Ok((200, STREAM)));
        let input = TurnInput::new("SYS", vec![ChatMessage::new(ChatRole::User, "hi \"there\"")]);
        p.begin_turn(&input).unwrap();
        assert!(p.begin_turn(&input).is_err(), "one turn in flight at a time");
        let events = drain(&mut p);
        assert_eq!(
            events,
            vec![
                ProviderEvent::Delta("Hello world".into()),
                ProviderEvent::Done {
                    text: "Hello world".into()
                }
            ]
        );
        let seen = p.transport.seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        let (url, token, body) = &seen[0];
        assert_eq!(url, "http://octos.test:19401/chat");
        assert_eq!(token.as_deref(), Some("secret-token"));
        let value = json::parse(body.as_bytes()).expect("request body is json");
        assert_eq!(
            value.get("message").and_then(Value::as_str),
            Some("SYS\n\nUSER REQUEST:\nhi \"there\"")
        );
        assert_eq!(value.get("session_id").and_then(Value::as_str), Some("sess"));
        assert_eq!(value.get("thread_id").and_then(Value::as_str), Some("sess-t0"));
        drop(seen);
        // The provider is idle again and the next turn gets a fresh thread.
        p.begin_turn(&input).unwrap();
        let _ = drain(&mut p);
        let seen = p.transport.seen.lock().unwrap();
        assert_eq!(seen[1].2.contains("\"sess-t1\""), true);
    }

    #[test]
    fn http_failures_surface_as_one_error_event() {
        let mut p = provider(Ok((502, "bad gateway")));
        p.begin_turn(&TurnInput::new("", vec![ChatMessage::new(ChatRole::User, "x")]))
            .unwrap();
        let events = drain(&mut p);
        assert!(matches!(&events[..], [ProviderEvent::Error(e)] if e.contains("502")), "{events:?}");
        assert!(p.active.is_none(), "provider is idle after an error");

        let mut p = provider(Err("connect refused for secret-token".into()));
        p.begin_turn(&TurnInput::new("", vec![ChatMessage::new(ChatRole::User, "x")]))
            .unwrap();
        let events = drain(&mut p);
        match &events[..] {
            [ProviderEvent::Error(e)] => assert_eq!(e, "provider error", "token must not leak"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_turn_without_a_user_message_is_refused() {
        let mut p = provider(Ok((200, STREAM)));
        let err = p
            .begin_turn(&TurnInput::new("SYS", vec![ChatMessage::new(ChatRole::Assistant, "a")]))
            .unwrap_err();
        assert!(err.contains("no user message"));
    }

    #[test]
    fn kind_and_availability_report_the_host() {
        let mut p = provider(Ok((200, STREAM)));
        assert_eq!(p.kind(), ProviderKind::Octos);
        match p.availability() {
            ProviderAvailability::Available { model, detail } => {
                assert_eq!(model, OCTOS_MODEL_LABEL);
                assert_eq!(detail, "http://octos.test:19401");
            }
            other => panic!("{other:?}"),
        }
    }
}
