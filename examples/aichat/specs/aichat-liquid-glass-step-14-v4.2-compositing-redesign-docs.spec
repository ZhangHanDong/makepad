spec: task
name: "AI Chat Liquid Glass Step 14 v4.2 Compositing Redesign Docs"
tags: [makepad, aichat, liquid-glass, macos-native, compositing, planning]
---

## Intent

Create the v4.2 compositing redesign document and implementation plan after the
Phase 12/13 proof showed that v4.1 native underlay is visible but not complete
Liquid Glass.

## Decisions

- v4.2 starts with prototypes, not aichat production UI changes.
- The central question is whether Apple native glass can visibly sample
  Makepad-rendered Metal content in a viable hierarchy.
- The design must include a decision gate between native interleave,
  ShaderBackdrop interiors, and native-underlay-only.
- v4.1 release notes must link the redesign as the next phase.

## Constraints

- Must not claim that v4.1 is complete Liquid Glass.
- Must not implement renderer changes in this documentation step.
- Must keep Studio remote release validation as the required UI verification
  path for future prototype tasks.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-14-v4.2-compositing-redesign-docs.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.spec.md`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.plan.md`
- `examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`

### Forbidden

- `examples/aichat/src/main.rs`
- `platform/**`
- `widgets/**`
- `makepad.splash`

### Out of Scope

- Above-Metal probe implementation.
- Two-layer interleave probe implementation.
- ShaderBackdrop implementation.

## Acceptance Criteria

Scenario: v4.2 design states the compositing problem
Test: `rg "v4.1 Finding|under-Metal|Decision Gate|AppleNativeInterleave|ShaderBackdropInterior" examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.spec.md`
Given the v4.1 proof result
When the v4.2 design is read
Then it explains why under-Metal native panels are insufficient and defines route choices

Scenario: v4.2 plan starts with prototypes
Test: `rg "Above-Metal Sampling Probe|Two-Layer Interleave Probe|Input And Studio Probe|Route Decision" examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.plan.md`
Given v4.2 is not ready for production integration
When the plan is read
Then it starts with isolated probes and ends in an explicit route decision

Scenario: v4.1 release notes point to v4.2
Test: `rg "v4.2 compositing redesign|aichat-liquid-glass-v4.2-compositing-redesign" examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`
Given v4.1 is documented as an underlay proof
When release notes are read
Then they link the v4.2 compositing redesign as the next phase

Scenario: documentation step does not edit renderer files
Test: `git diff --name-only -- examples/aichat/src/main.rs platform widgets makepad.splash`
Given this is a documentation-only step
When the diff is inspected
Then no renderer, widget, platform, or Studio runnable source file is changed

Scenario: v4.2 design rejects premature production integration
Level: static documentation guard
Test: `rg "Do not implement v4.2 directly inside aichat|Do not claim full native Liquid Glass|not complete Liquid Glass" examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.spec.md examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`
Given the native sampling route is not proven
When the design handles the failure path
Then it forbids premature aichat production integration and avoids full-glass claims
