// OpenHarmony network backend.
//
// OHOS ships no stable C HTTP client API, so HTTP runs through the ArkTS
// bridge: http_start forwards the request to the Makepad main loop (thread
// safe mpsc), the main loop calls ArkGlue.httpStart over the typed-args NAPI
// bridge, ArkTS drives @ohos.net.http requestInStream (system TLS, streaming
// dataReceive suits SSE), and the response flows back through the #[napi]
// callbacks below into the per-request EventSink.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use makepad_network::backend::ohos::register_platform_backend;
use makepad_network::{
    EventSink, HttpError, HttpRequest, HttpResponse, NetworkBackend, NetworkError, NetworkResponse,
    WsSend,
};
use napi_derive_ohos::napi;

use super::oh_callbacks::{send_from_ohos_message, FromOhosMessage};
use crate::makepad_live_id::LiveId;

struct LiveHttpRequest {
    sink: EventSink,
    metadata_id: LiveId,
    is_streaming: bool,
    status_code: u16,
    headers: BTreeMap<String, Vec<String>>,
    body_acc: Vec<u8>,
}

fn live_http_requests() -> &'static Mutex<HashMap<u64, LiveHttpRequest>> {
    static SLOT: OnceLock<Mutex<HashMap<u64, LiveHttpRequest>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(HashMap::new()))
}

// Headers cross the bridge as newline-delimited key/value pairs
// (k1\nv1\nk2\nv2...) to avoid a JSON dependency on either side.
fn parse_flat_headers(flat: &str) -> BTreeMap<String, Vec<String>> {
    let mut headers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut lines = flat.split('\n');
    while let (Some(key), Some(value)) = (lines.next(), lines.next()) {
        if key.is_empty() {
            continue;
        }
        headers
            .entry(key.to_ascii_lowercase())
            .or_default()
            .push(value.to_string());
    }
    headers
}

fn flatten_headers(headers: &BTreeMap<String, Vec<String>>) -> String {
    let mut flat = String::new();
    for (key, values) in headers {
        for value in values {
            flat.push_str(key);
            flat.push('\n');
            flat.push_str(value);
            flat.push('\n');
        }
    }
    flat
}

pub struct OhosNetworkBackend;

impl NetworkBackend for OhosNetworkBackend {
    fn http_start(
        &self,
        request_id: LiveId,
        request: HttpRequest,
        sink: EventSink,
    ) -> Result<(), NetworkError> {
        let Ok(mut live) = live_http_requests().lock() else {
            return Err(NetworkError::backend("ohos http registry lock poisoned"));
        };
        live.insert(
            request_id.0,
            LiveHttpRequest {
                sink,
                metadata_id: request.metadata_id,
                is_streaming: request.is_streaming,
                status_code: 0,
                headers: BTreeMap::new(),
                body_acc: Vec::new(),
            },
        );
        drop(live);
        send_from_ohos_message(FromOhosMessage::HttpRequestStart {
            request_id,
            method: request.method.as_str().to_string(),
            url: request.url,
            headers_flat: flatten_headers(&request.headers),
            body: request.body.unwrap_or_default(),
        });
        Ok(())
    }

    fn http_cancel(&self, request_id: LiveId) -> Result<(), NetworkError> {
        // The registry entry is removed before the ArkTS-side cancel lands,
        // so chunks/completions racing the cancel silently no-op (their
        // handlers find no entry). Intentional: the caller asked to cancel,
        // so no further events — including a cancel-ack — are emitted.
        if let Ok(mut live) = live_http_requests().lock() {
            live.remove(&request_id.0);
        }
        send_from_ohos_message(FromOhosMessage::HttpRequestCancel { request_id });
        Ok(())
    }

    fn ws_open(
        &self,
        _socket_id: LiveId,
        _request: HttpRequest,
        _sink: EventSink,
    ) -> Result<(), NetworkError> {
        Err(NetworkError::Unsupported(
            "OpenHarmony WebSocket backend not implemented yet (HTTP/SSE only)",
        ))
    }

    fn ws_send(&self, _socket_id: LiveId, _message: WsSend) -> Result<(), NetworkError> {
        Err(NetworkError::Unsupported(
            "OpenHarmony WebSocket backend not implemented yet (HTTP/SSE only)",
        ))
    }

    fn ws_close(&self, _socket_id: LiveId) -> Result<(), NetworkError> {
        Err(NetworkError::Unsupported(
            "OpenHarmony WebSocket backend not implemented yet (HTTP/SSE only)",
        ))
    }
}

pub fn register_ohos_network_backend() {
    register_platform_backend(Arc::new(OhosNetworkBackend));
}

fn parse_request_id(request_id: &str) -> Option<u64> {
    request_id.parse::<u64>().ok()
}

#[napi]
pub fn handle_http_headers(
    request_id: String,
    status_code: u32,
    headers_flat: String,
) -> napi_ohos::Result<()> {
    let Some(id) = parse_request_id(&request_id) else {
        return Ok(());
    };
    if let Ok(mut live) = live_http_requests().lock() {
        if let Some(req) = live.get_mut(&id) {
            // ArkTS sends headers twice: early from headersReceive with
            // status 0 (the OS reveals the code only in its final callback),
            // then again with the real code. Never downgrade either field.
            if status_code != 0 {
                req.status_code = status_code as u16;
            }
            if !headers_flat.is_empty() {
                req.headers = parse_flat_headers(&headers_flat);
            }
        }
    }
    Ok(())
}

#[napi]
pub fn handle_http_chunk(
    request_id: String,
    data: napi_ohos::JsArrayBuffer,
) -> napi_ohos::Result<()> {
    let Some(id) = parse_request_id(&request_id) else {
        return Ok(());
    };
    let data = data.into_value()?;
    let bytes: &[u8] = &data;
    if let Ok(mut live) = live_http_requests().lock() {
        if let Some(req) = live.get_mut(&id) {
            if req.is_streaming {
                // Mirrors the Linux backend: chunks carry the status code and
                // empty headers; the full header map arrives with complete.
                let _ = req.sink.emit(NetworkResponse::HttpStreamChunk {
                    request_id: LiveId(id),
                    response: HttpResponse {
                        metadata_id: req.metadata_id,
                        status_code: req.status_code,
                        headers: Default::default(),
                        body: Some(bytes.to_vec()),
                    },
                });
            } else {
                req.body_acc.extend_from_slice(bytes);
            }
        }
    }
    Ok(())
}

#[napi]
pub fn handle_http_complete(request_id: String) -> napi_ohos::Result<()> {
    let Some(id) = parse_request_id(&request_id) else {
        return Ok(());
    };
    let Some(req) = live_http_requests()
        .lock()
        .ok()
        .and_then(|mut l| l.remove(&id))
    else {
        return Ok(());
    };
    let response = HttpResponse {
        metadata_id: req.metadata_id,
        status_code: req.status_code,
        headers: req.headers,
        body: if req.is_streaming {
            None
        } else {
            Some(req.body_acc)
        },
    };
    let event = if req.is_streaming {
        NetworkResponse::HttpStreamComplete {
            request_id: LiveId(id),
            response,
        }
    } else {
        NetworkResponse::HttpResponse {
            request_id: LiveId(id),
            response,
        }
    };
    let _ = req.sink.emit(event);
    Ok(())
}

#[napi]
pub fn handle_http_error(request_id: String, message: String) -> napi_ohos::Result<()> {
    let Some(id) = parse_request_id(&request_id) else {
        return Ok(());
    };
    let Some(req) = live_http_requests()
        .lock()
        .ok()
        .and_then(|mut l| l.remove(&id))
    else {
        return Ok(());
    };
    let _ = req.sink.emit(NetworkResponse::HttpError {
        request_id: LiveId(id),
        error: HttpError {
            message,
            metadata_id: req.metadata_id,
        },
    });
    Ok(())
}
