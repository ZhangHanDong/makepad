spec: task
name: "Capability manifest with CFP permission levels"
tags: [makepad, agent2app, aichat, capability, cfp]
depends: [agent2app-p0-isolated-vm-agent-registration, agent2app-p1-multi-instance-regression]
estimate: 2d
---

## Intent

aichat's `handle_splash_event` currently trusts every known event_id equally;
the only policy is "unknown → log and ignore". Introduce an explicit
capability manifest that assigns each event a permission level, using the CFP
v0.3 L4 vocabulary, so generated apps get a declared, enforced permission
policy before agent2app grows beyond demo events. This is vocabulary
adoption only — no CFP runtime, claims, or review chains.

## Decisions

- Permission enum in `examples/aichat-agent2app/src/main.rs`:
  `ReadOnly`, `RequiresConfirmation`, `AutoExecutable`, `Forbidden`
  (names verbatim from CFP v0.3 §9.2).
- Manifest is a static table mapping event patterns to levels:
  `inc|dec|reset|timer.*` → AutoExecutable, `ask_ai` →
  RequiresConfirmation, unknown events → treated as Forbidden-by-default
  (log and ignore, current behavior preserved).
- `handle_splash_event` consults the manifest before dispatching; a
  Forbidden verdict never executes the event handler, regardless of payload.
- RequiresConfirmation flow: the event is parked in a pending slot and a
  confirmation card is appended to the chat; the card's Approve/Deny buttons
  are themselves `agent.notify("host.confirm", {})` /
  `agent.notify("host.deny", {})` calls; `host.confirm`/`host.deny` are
  AutoExecutable manifest entries. Only one pending confirmation at a time;
  a newer RequiresConfirmation event replaces the parked one with a log line.
- Manifest lookup and the pending-confirmation state machine are plain
  functions/structs testable without Cx; tests live in a `#[cfg(test)]`
  module in main.rs.

## Boundaries

### Allowed Changes
- examples/aichat-agent2app/src/main.rs
- examples/aichat-agent2app/specs/AGENT-2-APP-DESIGIN.md

### Forbidden
- Do not modify widgets/** or platform/**
- Do not add new cargo dependencies
- Do not persist pending confirmations to disk

## Completion Criteria

Scenario: permission enum matches the CFP vocabulary
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_permission_levels_match_cfp_vocabulary
  Given the permission enum in examples/aichat-agent2app/src/main.rs
  When its variants are enumerated
  Then they are exactly ReadOnly, RequiresConfirmation, AutoExecutable, Forbidden

Scenario: auto-executable event dispatches immediately (critical)
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_auto_executable_event_dispatches
  Given the manifest maps "inc" to AutoExecutable
  When handle_splash_event receives event "inc"
  Then the counter state mutation runs in the same call
  And no confirmation card is created

Scenario: forbidden event never executes
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_forbidden_event_is_refused
  Given the manifest maps event "fs.write" to Forbidden
  When handle_splash_event receives event "fs.write" with any payload
  Then no state mutation and no agent prompt occurs
  And a log line records the refusal with the event id

Scenario: unknown event defaults to forbidden handling
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_unknown_event_treated_as_forbidden
  Given an event id absent from the manifest
  When handle_splash_event receives it
  Then it is logged and ignored exactly like a Forbidden event

Scenario: ask_ai waits for confirmation
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_requires_confirmation_parks_event
  Given the manifest maps "ask_ai" to RequiresConfirmation
  When handle_splash_event receives event "ask_ai"
  Then no LLM request is sent yet
  And the event is parked as the pending confirmation

<!-- lint-ack: verification-metadata-suggestion — the LLM dispatch is replaced by a recording stub (declared in the Test Double line); no real network I/O is exercised -->
Scenario: approval releases the parked event
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_host_confirm_releases_pending
  Test Double: LLM request path replaced by a recording stub; no network call
  Given a parked "ask_ai" pending confirmation
  When event "host.confirm" arrives
  Then the parked event dispatches (LLM request path is invoked)
  And the pending slot is empty afterwards

Scenario: denial discards the parked event
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_host_deny_discards_pending
  Given a parked "ask_ai" pending confirmation
  When event "host.deny" arrives
  Then the parked event is discarded without dispatch
  And the pending slot is empty afterwards

Scenario: newer confirmation request replaces the older one
  Test:
    Package: makepad-example-aichat-agent2app
    Filter: test_second_pending_replaces_first
  Given a parked "ask_ai" pending confirmation
  When another RequiresConfirmation event arrives
  Then the older parked event is discarded with a log line
  And the newer event occupies the pending slot

## Out of Scope

- CFP Claim/Commitment objects, ReviewChain, event sourcing
- Signature or identity verification of events
- tools/canvas manifest enforcement (follow-up after P2 stabilizes)
- AppBundle metadata persistence (P4, spec deferred)
