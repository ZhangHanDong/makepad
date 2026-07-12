# CFP-lite Core S2b Aichat Integration Implementation Plan

> **For agent execution:** Use `superpowers:subagent-driven-development` task by task. Every implementation task requires RED → GREEN → REFACTOR evidence, a task report, and an independent read-only review before the next task.

**Goal:** Replace `examples/aichat-agent2app`'s local capability gate and pending-confirmation reducer with the reviewed `cfp-lite-core` S2a API while preserving the existing visible aichat behavior and raw `ask_ai` payload bytes.

**Architecture:** A once-parsed `GateProfile` derives both the manifest and registry from core's embedded `AICHAT_GATE_V1_JSON`. A single `SplashGateAdapter` owns the authoritative core `ReviewChain` and a non-authoritative raw-payload cache keyed by active review ID. Every notify becomes an immutable `ExecutionIntent`, passes `evaluate_gate`, and uses `ReviewChain::apply` for all proposal/approval/rejection state changes. The existing dispatcher remains unchanged and receives a core intent plus the exact legacy raw JSON only after a valid Dispatch or bound approval.

**Toolchain:** Rust stable, existing Makepad workspace, `cfp-lite-core` git dependency pinned to `4f798f00c34a3ec1416ae41e549983c226b4cb38`, serde_json 1, syn 2 for test-only AST inspection, agent-spec 1.0.0.

**Contract:** `specs/cfp-lite-core-s2b-aichat-integration.spec` — 18 authored scenarios.

**S2b base:** `9e0658a1` (approved contract commit). The implementation base becomes the plan commit created from this file.

**Known baseline:** Before source changes, `cargo test -p makepad-example-aichat-agent2app` passes 43/43 tests and emits one pre-existing ambiguous glob re-export warning. This task must not broaden scope to fix that warning. There are no visual changes, so direct package tests are the primary verification; do not launch the Makepad UI.

---

### Task 1: Exact core dependency and single-source GateProfile

**Files:**
- Modify: `examples/aichat-agent2app/Cargo.toml`
- Modify: `examples/aichat-agent2app/src/main.rs`

**Interfaces:**
- `struct GateProfile { manifest: CapabilityManifest, action_names: Vec<String>, registry: ActionRegistry }`
- `fn parse_gate_profile(source: &str) -> Result<GateProfile, String>`
- `fn runtime_gate_profile() -> &'static Result<GateProfile, String>` backed by `OnceLock`
- Fixed helpers for `Principal { id: "transport:aichat" }` and `Scope { app_id: "app:aichat-agent2app" }`

**Normative implementation shape:**

```rust
static RUNTIME_GATE_PROFILE: OnceLock<Result<GateProfile, String>> = OnceLock::new();

fn runtime_gate_profile() -> &'static Result<GateProfile, String> {
    RUNTIME_GATE_PROFILE.get_or_init(|| {
        parse_gate_profile(cfp_lite_core::fixtures::AICHAT_GATE_V1_JSON)
    })
}

fn aichat_principal() -> Principal {
    Principal { id: "transport:aichat".into() }
}

fn aichat_scope() -> Scope {
    Scope { app_id: "app:aichat-agent2app".into() }
}
```

`parse_gate_profile` must parse `schema`, `manifest`, `registry`, `principal`, and `scope` from the supplied bytes; reject anything except schema `aichat-gate-v1` and the two fixed authority values; reject empty/duplicate registry actions; deserialize the eight manifest entries into core `PermissionLevel`; then build both `action_names` and `ActionRegistry` from the same parsed registry vector. The runtime function above is the only production `OnceLock`; no test profile may be installed into it.

- [ ] **Step 1: Write failing dependency/profile tests**

Add exact selectors:
- `test_cfp_core_git_dependency_is_pinned`
- `test_permission_levels_match_cfp_vocabulary`
- `test_aichat_profile_comes_from_shared_fixture`
- `test_invalid_profile_and_payload_fail_closed_preserving_review` (profile-parsing portion first)

The pin test reads the package `Cargo.toml` and rejects branch/tag/path/version alternatives. Profile tests parse `cfp_lite_core::fixtures::AICHAT_GATE_V1_JSON`, assert schema/principal/scope, the exact 13 registry actions, and the exact eight manifest patterns. RED must fail because the dependency and parser do not exist; record a nonzero test count whenever compilation reaches the runner.

Start from these test assertions so the parser contract is not inferred later:

```rust
#[test]
fn test_cfp_core_git_dependency_is_pinned() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains(
        "cfp-lite-core = { git = \"https://github.com/ZhangHanDong/a2app-harness.git\", rev = \"4f798f00c34a3ec1416ae41e549983c226b4cb38\", package = \"cfp-lite-core\" }"
    ));
    // Parse/check the dependency line has exactly git+rev+package and no path/branch/tag/version.
}

#[test]
fn test_aichat_profile_comes_from_shared_fixture() {
    let profile = parse_gate_profile(cfp_lite_core::fixtures::AICHAT_GATE_V1_JSON).unwrap();
    assert_eq!(profile.action_names.len(), 13);
    assert_eq!(profile.manifest.actions.len(), 8);
    assert!(parse_gate_profile("{}").is_err());
}
```

- [ ] **Step 2: Add minimal dependencies and fail-closed parser**

Add exactly:

```toml
cfp-lite-core = { git = "https://github.com/ZhangHanDong/a2app-harness.git", rev = "4f798f00c34a3ec1416ae41e549983c226b4cb38", package = "cfp-lite-core" }
serde_json = "1"

[dev-dependencies]
syn = { version = "2", features = ["full", "visit"] }
```

Parse fixture JSON into a small private serde_json view, deserialize permissions explicitly into core `PermissionLevel`, construct `CapabilityManifest`, and derive one `Vec<String>` used to construct the private `ActionRegistry`. Reject malformed/missing/duplicate data and mismatched fixed authority fields. Cache `Result`, never an unchecked fallback.

- [ ] **Step 3: Verify and self-review**

Run each new selector plus:

```bash
cargo test -p makepad-example-aichat-agent2app test_aichat_profile_comes_from_shared_fixture
cargo fmt -p makepad-example-aichat-agent2app -- --check
git diff --check
```

Commit: `feat(aichat): load capability profile from CFP core`

---

### Task 2: Core-gated Splash adapter decisions

**Files:**
- Modify: `examples/aichat-agent2app/src/main.rs`

**Interfaces:**
- `struct SplashGateAdapter { review_chain: ReviewChain, raw_payloads: BTreeMap<String, String> }`
- `enum SplashAdapterDecision` with the exact four contract variants
- `SplashGateAdapter::process(action: &str, raw_payload: &str, profile: Result<&GateProfile, &String>) -> SplashAdapterDecision`
- Read-only helpers for active record/card rendering; test-only fault-injection helpers remain under `#[cfg(test)]`

**Normative decision and gate mapping:**

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
enum SplashAdapterDecision {
    Dispatch { intent: ExecutionIntent, raw_payload: String },
    Park { review_id: String, replaced_review_id: Option<String> },
    Discard { review_id: Option<String> },
    Refuse { code: ProtocolErrorCode, action: String, log: String },
}

// Ordinary notify path (host.confirm/host.deny are handled by Task 3):
let profile = match profile {
    Ok(profile) => profile,
    Err(_) => return refuse(ProtocolErrorCode::InvalidManifest, action),
};
let intent = match ExecutionIntent::from_json_str(action, raw_payload) {
    Ok(intent) => intent,
    Err(_) => return refuse(ProtocolErrorCode::InvalidField, action),
};
let gate = match evaluate_gate(
    intent,
    aichat_principal(),
    aichat_scope(),
    &profile.registry,
    &profile.manifest,
) {
    Ok(gate) => gate,
    Err(error) => return refuse(error.code, action),
};
```

Map `GateDecision::Dispatch { intent, .. }` directly to `Dispatch` with `raw_payload.to_owned()`. Map `GateDecision::Refuse { code, action }` without replacing the core code. For `GateDecision::Park(park)`, call `self.review_chain.apply(ReviewEvent::Propose { park })` first; only after success commit the returned chain and update the cache from `ReviewOutcome::Proposed`. A profile parse failure maps to `InvalidManifest`; an invalid payload maps to `InvalidField`. Refusal logs may contain the action/code but never the raw payload.

- [ ] **Step 1: Write failing basic gate tests**

Add exact selectors:
- `test_auto_executable_event_dispatches`
- `test_all_aichat_auto_actions_dispatch_via_core`
- `test_forbidden_and_unknown_events_are_refused_by_core`
- `test_requires_confirmation_parks_event`
- `test_transport_identity_is_fixed_and_payload_cannot_override_authority`

Tests must prove the nine legacy automatic actions preserve the exact original raw string, unknown/forbidden codes come from core policy, and `ask_ai` creates a core Pending record without dispatch. Include a spoofed authority payload and inspect the resulting `ReviewRecord` principal/scope.

Use a table-driven test rather than production branching:

```rust
for action in [
    "inc", "dec", "reset", "timer.start", "timer.pause", "timer.toggle",
    "timer.reset", "timer.add_minute", "timer.subtract_minute",
] {
    let raw = format!(r#"{{"action_echo":"{action}","spacing": [ 1, 2 ]}}"#);
    let decision = adapter.process(action, &raw, Ok(runtime_profile));
    assert!(matches!(decision,
        SplashAdapterDecision::Dispatch { ref raw_payload, .. } if raw_payload == &raw));
}
```

- [ ] **Step 2: Implement the minimal adapter gate path**

For ordinary actions, parse raw JSON to `ExecutionIntent`, invoke `evaluate_gate`, and map the result. On Park, call `ReviewChain::apply(ReviewEvent::Propose)`, install the returned chain, remove any replaced cache entry, and insert the exact raw bytes under the new review ID. Profile and payload errors map to the contract's fail-closed codes and redact payload content from logs. Do not special-case the nine action names in production.

- [ ] **Step 3: Verify invariants and self-review**

Assert after every normal event that cache keys are exactly the zero-or-one active review ID set. Run all five selectors, then:

```bash
cargo test -p makepad-example-aichat-agent2app test_auto_executable_event_dispatches
cargo fmt -p makepad-example-aichat-agent2app -- --check
git diff --check
```

Commit: `feat(aichat): route splash events through CFP gate`

---

### Task 3: Bound approval, rejection, replacement, and raw-byte compatibility

**Files:**
- Modify: `examples/aichat-agent2app/src/main.rs`

- [ ] **Step 1: Write failing review/cache tests**

Add exact selectors:
- `test_host_confirm_releases_pending`
- `test_host_confirm_preserves_original_payload_bytes`
- `test_confirm_refuses_missing_or_mismatched_payload_cache`
- `test_host_deny_discards_pending`
- `test_confirm_and_deny_without_pending_preserve_legacy_behavior`
- `test_second_pending_replaces_first`
- `test_gate_outcomes_preserve_active_review`
- Complete the payload/runtime portions of `test_invalid_profile_and_payload_fail_closed_preserving_review`

The fault tests snapshot serialized/structural chain state and cache before each operation, then prove refusal is transactional. Cover all three cache faults explicitly:

```rust
// Arrange one Pending review, then inject each fault independently.
// Snapshot AFTER fault injection and BEFORE host.confirm.
let chain_before = adapter.review_chain.clone();
let chain_bytes_before = serde_json::to_vec(&adapter.review_chain).unwrap();
let cache_before = adapter.raw_payloads.clone();
let decision = adapter.process("host.confirm", "{}", Ok(profile));
assert_eq!(adapter.review_chain, chain_before);
assert_eq!(serde_json::to_vec(&adapter.review_chain).unwrap(), chain_bytes_before);
assert_eq!(adapter.raw_payloads, cache_before);
```

Run this skeleton for (a) the active key removed from the cache → `InvalidField`; (b) the active key mapped to syntactically invalid JSON such as `"{"` → `InvalidField`; and (c) the active key mapped to valid JSON with different semantics → `ReviewBindingMismatch`. Replacement covers Pending/Deferred × same/different intent. Non-review outcomes compare full history, active ID, and cache before/after.

Define exactly one test-only read-only profile here and reuse this same helper in Task 5:

```rust
#[cfg(test)]
fn synthetic_readonly_profile() -> GateProfile {
    let action_names = vec!["agent.status".to_string()];
    GateProfile {
        manifest: CapabilityManifest {
            manifest_version: "1".into(),
            actions: BTreeMap::from([(
                "agent.status".into(),
                CapabilityRule { permission: PermissionLevel::ReadOnly },
            )]),
        },
        registry: ActionRegistry::new(action_names.clone()),
        action_names,
    }
}
```

The concrete manifest value must use the core manifest type rather than a local permission wrapper. This helper is `#[cfg(test)]`, is the only `synthetic_readonly` constructor, and must never be referenced by `runtime_gate_profile` or its `OnceLock`.

- [ ] **Step 2: Implement confirm/deny through core transitions only**

`host.confirm` must locate active record and raw bytes, reconstruct an intent only to validate the cached JSON and compare its hash to the record, then apply `Approve`. The `ExecutionIntent` placed in `Dispatch` must come from `ReviewOutcome::Approved`, never from the reconstructed cache intent:

```rust
let review_id = match self.review_chain.active_review_id() {
    Some(review_id) => review_id.to_owned(),
    None => return refuse(ProtocolErrorCode::ReviewNotFound, "host.confirm"),
};
let record = self.review_chain.records().iter()
    .find(|record| record.review_id() == review_id).unwrap();
let expected_hash = record.intent_hash().to_owned();
let action = record.intent().action().to_owned();
let raw = match self.raw_payloads.get(&review_id).cloned() {
    Some(raw) => raw,
    None => return refuse(ProtocolErrorCode::InvalidField, "host.confirm"),
};
let reconstructed = match ExecutionIntent::from_json_str(&action, &raw) {
    Ok(intent) => intent,
    Err(_) => return refuse(ProtocolErrorCode::InvalidField, "host.confirm"),
};
if reconstructed.intent_hash() != expected_hash {
    return refuse(ProtocolErrorCode::ReviewBindingMismatch, "host.confirm");
}
let (next_chain, outcome) = match self.review_chain.apply(ReviewEvent::Approve {
    review_id: review_id.clone(),
    intent_hash: expected_hash,
}) {
    Ok(applied) => applied,
    Err(error) => return refuse(error.code, "host.confirm"),
};
let approved_intent = match outcome {
    ReviewOutcome::Approved { intent, .. } => intent,
    _ => unreachable!("Approve must return Approved"),
};
self.review_chain = next_chain;
self.raw_payloads.remove(&review_id);
SplashAdapterDecision::Dispatch { intent: approved_intent, raw_payload: raw }
```

All error returns happen before committing `next_chain` or mutating the cache. Missing or syntactically invalid cache returns `InvalidField`; semantic mismatch returns `ReviewBindingMismatch`; no active record returns `ReviewNotFound`. `host.deny` with an active review applies `Reject`, verifies `ReviewOutcome::Rejected`, then commits the returned chain, removes the cache, and returns `Discard { review_id: Some(id) }`; without active review it returns `Discard { review_id: None }`.

- [ ] **Step 3: Verify the complete adapter matrix**

Run all eight selectors and the package tests. Confirm logs contain action names but not sensitive payload content, and only `#[cfg(test)]` code can violate the raw-cache invariant.

Commit: `feat(aichat): bind confirmations to CFP review chain`

---

### Task 4: Replace App-local pending state and remove duplicate governance

**Files:**
- Modify: `examples/aichat-agent2app/src/main.rs`

- [ ] **Step 1: Write the failing AST/source discipline test**

Add exact selector:
- `test_local_gate_review_definitions_are_removed`

Use syn AST/token inspection of production items to reject the local `PermissionLevel`, `PendingConfirmation`, `capability_permission`, `CAPABILITY_MANIFEST`, or a local review status/reducer. Assert `App` has one `SplashGateAdapter` field and the adapter's authoritative state is exactly the core chain plus raw cache. The audit must not pass by comments/string renaming alone.

- [ ] **Step 2: Migrate the existing App handler**

Replace `pending_confirmation` with one adapter. Route the existing Splash notification handler through `SplashAdapterDecision` using this behavior:

```rust
match self.splash_gate.process(event_id, payload, runtime_profile_result) {
    SplashAdapterDecision::Dispatch { intent, raw_payload } => {
        self.dispatch_splash_event(cx, intent.action(), &raw_payload);
    }
    SplashAdapterDecision::Park { .. } => {
        // Read the active core ReviewRecord and append the same confirmation card shape.
        self.append_confirmation_card(cx, self.splash_gate.active_record().unwrap());
    }
    SplashAdapterDecision::Discard { review_id } => {
        // Log denied/no-pending exactly as baseline. Do not dispatch and do not edit/remove
        // the already persisted confirmation-card ChatMessage.
        log_discard(review_id);
    }
    SplashAdapterDecision::Refuse { log: line, .. } => log!("{}", line),
}
```

`Discard` transitions only adapter review/cache state; it **must not clear, rewrite, or remove the persisted confirmation card**, preserving the baseline visible message. Remove local gate/review definitions and table. Do not alter prompts, DSL, storage, agent setup, or dispatcher branches such as the raw `ask_ai` prompt formatting.

- [ ] **Step 3: Verify behavior and boundaries**

Run the AST selector, all prior gate/review selectors, and:

```bash
cargo test -p makepad-example-aichat-agent2app
cargo fmt -p makepad-example-aichat-agent2app -- --check
git diff --check
```

Commit: `refactor(aichat): replace local confirmation state with CFP adapter`

---

### Task 5: Execute the shared 14-case fixture generically

**Files:**
- Modify: `examples/aichat-agent2app/src/main.rs`

- [ ] **Step 1: Write the failing shared-fixture test**

Add exact selector:
- `test_aichat_uses_shared_gate_fixture`

The runner reads `cfp_lite_core::fixtures::AICHAT_GATE_V1_JSON`, iterates every case and step, interprets `notify`, `approve_active`, `reject_active`, and the two adapter-without-pending operations generically, then compares actual decisions, records, active projection, dispatch, and adapter expectation to each case's `expected`. Do not copy case IDs, action/permission tables, step sequences, expected hashes, or projection rows into Rust.

Use the adapter itself for every step and expose only this exact test-only trace/projection seam:

```rust
// Private production observer event used only to avoid duplicating gate/reducer semantics.
enum AdapterCoreEvent<'a> {
    Gate(&'a GateDecision),
    Review(&'a ReviewOutcome),
}

// process() supplies a no-op observer; process_traced() supplies the collector below.
fn process_impl(
    &mut self,
    action: &str,
    raw_payload: &str,
    profile: Result<&GateProfile, &String>,
    observe: &mut dyn FnMut(AdapterCoreEvent<'_>),
) -> SplashAdapterDecision;

#[cfg(test)]
#[derive(Clone, Debug)]
enum FixtureTraceEntry {
    Gate(GateDecision),
    Review(ReviewOutcome),
}

#[cfg(test)]
#[derive(Default)]
struct FixtureTrace {
    entries: Vec<FixtureTraceEntry>,
    dispatch: Option<ExecutionIntent>,
    adapter_decision: Option<SplashAdapterDecision>,
}

#[cfg(test)]
fn process_traced(
    adapter: &mut SplashGateAdapter,
    action: &str,
    raw_payload: &str,
    profile: Result<&GateProfile, &String>,
    trace: &mut FixtureTrace,
) -> SplashAdapterDecision;

#[cfg(test)]
fn fixture_projection(adapter: &SplashGateAdapter, trace: &FixtureTrace) -> serde_json::Value;
```

`process_impl` emits `Gate` immediately after every successful `evaluate_gate` and emits `Review` immediately after every successful `ReviewChain::apply`, preserving interleaving. `process_traced` clones only those core values into `entries` and returns the actual adapter decision. `run_fixture_step` sets `trace.dispatch` from any returned `Dispatch`; it sets `trace.adapter_decision` **only** for the fixture operation `adapter_notify`, leaving it `None` for `notify`, `approve_active`, and `reject_active`. This is what keeps core-runner cases' `expected.adapter` equal to `null` without consulting case IDs.

`fixture_projection` is a pure serializer with these exact sources:

- `decisions`: ordered `FixtureTraceEntry` values. A gate Dispatch projects `{kind, permission, action}` using the core permission and intent; Park/Refuse use the core variant; review entries project the fields of the core `ReviewOutcome`.
- `records`: map each core record getter to `{proposal_seq, review_id, intent_hash, principal, scope, intent: {action, payload}, status}`. Do not serialize `ExecutionIntent` wholesale because the fixture's nested intent intentionally omits its two derived hash fields.
- `active_review_id`: `review_chain.active_review_id()`; `active_record`: the matching index in `review_chain.records()` or null.
- `dispatch`: `trace.dispatch` projected as `{action, payload}` or null.
- `adapter`: `trace.adapter_decision` projected to `{kind, success, discarded}` or null.

It must not call `permission_for`, `evaluate_gate`, `ReviewChain::apply`, or contain an action/permission/expected-case lookup table.

- [ ] **Step 2: Implement only the generic fixture interpreter**

Use serde_json values for fixture traversal and the single `fixture_projection` serializer above. For `profile == "aichat"`, call `parse_gate_profile` on the embedded fixture bytes; for `profile == "synthetic_readonly"`, call the one `#[cfg(test)] synthetic_readonly_profile()` defined in Task 3. No second synthetic profile or inline permission mapping is allowed, and the synthetic profile must not enter the runtime `OnceLock`. Assert all 14 cases execute exactly once and embedded fixture bytes remain the source used by runtime profile parsing.

The generic loop shape is fixed as follows; only operation-name dispatch is local logic:

```rust
let root: serde_json::Value = serde_json::from_str(AICHAT_GATE_V1_JSON).unwrap();
let cases = root["cases"].as_array().unwrap();
assert_eq!(cases.len(), 14);
for case in cases {
    let mut adapter = SplashGateAdapter::default();
    let mut trace = FixtureTrace::default();
    let profile = profile_named(case["profile"].as_str().unwrap());
    for step in case["steps"].as_array().unwrap() {
        run_fixture_step(&mut adapter, &mut trace, &profile, step);
    }
    assert_eq!(fixture_projection(&adapter, &trace), case["expected"]);
}
```

- [ ] **Step 3: Verify fixture and package**

Run:

```bash
cargo test -p makepad-example-aichat-agent2app test_aichat_uses_shared_gate_fixture
cargo test -p makepad-example-aichat-agent2app
cargo fmt -p makepad-example-aichat-agent2app -- --check
git diff --check
```

Commit: `test(aichat): execute shared CFP gate fixture`

---

### Task 6: S2b lifecycle acceptance, independent review, and branch publication

**Files:**
- Modify only if verification exposes an in-boundary defect.

- [ ] **Step 1: Fresh controller verification**

Run:

```bash
agent-spec --version
agent-spec parse specs/cfp-lite-core-s2b-aichat-integration.spec
agent-spec lint specs/cfp-lite-core-s2b-aichat-integration.spec --min-score 0.7
cargo test -p makepad-example-aichat-agent2app
cargo fmt -p makepad-example-aichat-agent2app -- --check
git diff --check
```

Confirm the package test count is nonzero, all 18 named selectors execute, and the only remaining warning is the known pre-existing ambiguous re-export warning.

- [ ] **Step 2: Non-empty boundary lifecycle**

Derive actual implementation paths from the plan commit to final product HEAD. Exclude only the plan/spec docs, assert the path list is nonempty and equals the intended Cargo/main paths, then pass each path as repeatable `--change`:

```bash
agent-spec lifecycle specs/cfp-lite-core-s2b-aichat-integration.spec --code . \
  --change examples/aichat-agent2app/Cargo.toml \
  --change examples/aichat-agent2app/src/main.rs \
  --format json --run-log-dir /private/tmp/makepad-agent-spec-runs
agent-spec guard --spec-dir specs --code . \
  --change examples/aichat-agent2app/Cargo.toml \
  --change examples/aichat-agent2app/src/main.rs --min-score 0.7
agent-spec explain specs/cfp-lite-core-s2b-aichat-integration.spec --code . --format markdown
agent-spec stamp specs/cfp-lite-core-s2b-aichat-integration.spec --code . --dry-run
```

Expected: 18 authored scenarios + 1 synthetic boundary scenario = 19/19 lifecycle; explain/stamp = 18/18; zero failed/skipped/uncertain/pending.

- [ ] **Step 3: Independent whole-S2b review**

Require a reviewer to build an 18-scenario spec→code→assertion matrix, inspect the generic fixture runner instead of trusting its name, verify raw-cache non-authority and transactional failures, and confirm no forbidden Makepad/prompt/UI/dispatcher changes.

- [ ] **Step 4: Controller publication and umbrella handoff**

After fresh verification and a clean review, push `cfp-lite-core-s2-integration` to the makepad fork remote. Record the exact S2b commit and remote ref in the a2app SDD ledger. Only then mark umbrella S2 complete and begin S3 in the a2app worktree; do not merge either branch automatically.
