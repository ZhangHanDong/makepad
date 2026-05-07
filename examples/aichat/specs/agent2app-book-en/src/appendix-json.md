# Appendix: JSON Examples

## Mission Room Event

```json
{
  "body": "Mission update: plan is waiting for approval.",
  "msgtype": "m.text",
  "org.octos.app": {
    "type": "mission_room",
    "version": 1,
    "scope": "room",
    "app_id": "mission.main",
    "initial_state": {
      "goal": {
        "title": "Ship room-scoped agent2view runtime",
        "status": "planning"
      },
      "phase": "planning",
      "tasks": [],
      "agents": [],
      "pending_human_actions": [],
      "decisions": [],
      "blockers": []
    }
  },
  "org.octos.actions": [
    {
      "id": "approve_plan",
      "label": "Approve plan",
      "style": "primary"
    },
    {
      "id": "request_plan_changes",
      "label": "Request changes",
      "style": "secondary"
    }
  ]
}
```

## Action Response

```json
{
  "org.octos.action_response": {
    "action_id": "approve_plan",
    "source_event_id": "$mission123",
    "app": {
      "type": "mission_room",
      "version": 1,
      "scope": "room",
      "app_id": "mission.main"
    }
  }
}
```

## Account Dashboard Event

```json
{
  "body": "Mission dashboard update.",
  "msgtype": "m.text",
  "org.octos.app": {
    "type": "mission_dashboard",
    "version": 1,
    "scope": "account",
    "app_id": "missions.global",
    "initial_state": {}
  }
}
```
