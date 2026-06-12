use std::sync::{Arc, Mutex, OnceLock};

use makepad_live_id::LiveId;

use super::{EventSink, NetworkBackend};
use crate::types::{HttpRequest, NetworkError, WsSend};

// OpenHarmony has no stable C HTTP client API, so the real backend lives in
// makepad-platform where the ArkTS bridge (@ohos.net.http requestInStream)
// and the NAPI callback plumbing are available. makepad-platform registers it
// here at startup; until then every call reports a clear error instead of
// silently dropping traffic. This mirrors the Android platform-backend slot.
fn backend_slot() -> &'static Mutex<Option<Arc<dyn NetworkBackend>>> {
    static SLOT: OnceLock<Mutex<Option<Arc<dyn NetworkBackend>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

pub fn register_platform_backend(backend: Arc<dyn NetworkBackend>) {
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = Some(backend);
    }
}

pub fn clear_platform_backend() {
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = None;
    }
}

fn platform_backend() -> Result<Arc<dyn NetworkBackend>, NetworkError> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(Arc::clone))
        .ok_or(NetworkError::Unsupported(
            "OpenHarmony network backend not registered by makepad-platform",
        ))
}

struct OhosDelegatingBackend;

impl NetworkBackend for OhosDelegatingBackend {
    fn http_start(
        &self,
        request_id: LiveId,
        request: HttpRequest,
        sink: EventSink,
    ) -> Result<(), NetworkError> {
        platform_backend()?.http_start(request_id, request, sink)
    }

    fn http_cancel(&self, request_id: LiveId) -> Result<(), NetworkError> {
        platform_backend()?.http_cancel(request_id)
    }

    fn ws_open(
        &self,
        socket_id: LiveId,
        request: HttpRequest,
        sink: EventSink,
    ) -> Result<(), NetworkError> {
        platform_backend()?.ws_open(socket_id, request, sink)
    }

    fn ws_send(&self, socket_id: LiveId, message: WsSend) -> Result<(), NetworkError> {
        platform_backend()?.ws_send(socket_id, message)
    }

    fn ws_close(&self, socket_id: LiveId) -> Result<(), NetworkError> {
        platform_backend()?.ws_close(socket_id)
    }
}

pub(crate) fn create_backend() -> Arc<dyn NetworkBackend> {
    Arc::new(OhosDelegatingBackend)
}
