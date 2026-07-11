use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotifyEnvelope {
    Send(String),
    Skip { log: String },
}

#[derive(Serialize)]
struct ClickEnvelope<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    name: &'a str,
}

#[derive(Serialize)]
struct NotifyEnvelopeBody<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    event: &'a str,
    payload: Value,
}

pub fn click_envelope(name: &str) -> String {
    serde_json::to_string(&ClickEnvelope {
        kind: "click",
        name,
    })
    .unwrap_or_default()
}

pub fn notify_envelope(event_id: &str, payload: &str) -> NotifyEnvelope {
    if event_id.is_empty() {
        return NotifyEnvelope::Skip {
            log: "[canvas] rejected empty agent.notify event id".to_string(),
        };
    }

    let payload = serde_json::from_str(payload).unwrap_or_else(|_| Value::String(payload.into()));
    let body = NotifyEnvelopeBody {
        kind: "notify",
        event: event_id,
        payload,
    };

    NotifyEnvelope::Send(serde_json::to_string(&body).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_action_produces_notify_envelope() {
        let NotifyEnvelope::Send(line) = notify_envelope("inc", "{}") else {
            panic!("notify event should produce an envelope");
        };
        let parsed: Value = serde_json::from_str(&line).expect("envelope should be valid JSON");

        assert_eq!(parsed["type"], "notify");
        assert_eq!(parsed["event"], "inc");
        assert_eq!(parsed["payload"], serde_json::json!({}));
    }

    #[test]
    fn test_button_click_produces_click_envelope() {
        let line = click_envelope("start");

        assert_eq!(line, r#"{"type":"click","name":"start"}"#);
        assert_ne!(line, "start");
    }

    #[test]
    fn test_invalid_payload_is_wrapped_as_string() {
        let raw = "{not valid json";
        let NotifyEnvelope::Send(line) = notify_envelope("inc", raw) else {
            panic!("malformed payload should still produce an envelope");
        };
        let parsed: Value = serde_json::from_str(&line).expect("envelope should be valid JSON");

        assert_eq!(parsed["type"], "notify");
        assert_eq!(parsed["event"], "inc");
        assert_eq!(parsed["payload"], Value::String(raw.to_string()));
    }

    #[test]
    fn test_empty_event_id_is_logged_and_skipped() {
        let NotifyEnvelope::Skip { log } = notify_envelope("", "{}") else {
            panic!("empty event id should be skipped");
        };

        assert!(log.contains("rejected empty agent.notify event id"));
    }

    #[test]
    fn test_canvas_skill_documents_notify_envelope() {
        let skill = include_str!("../../skills/app/SKILL.md");

        assert!(skill.contains("agent.notify"));
        assert!(skill.contains(r#"{"type":"notify""#));
    }
}
