# Goals and Boundaries

The core goal of Agent2App is to define how an Agent hands a structured app view to a Host for rendering, and how user operations in that view flow back to the Host or Agent.

This version covers only `agent2view`.

## Goals

`agent2view` must answer four questions:

1. How does an Agent declare the app view it wants rendered?
2. How does the Host validate app type, state, template, and action?
3. How does the Host preserve state so authoritative state is not lost when widgets are destroyed?
4. How do user actions return to the Host or Agent while keeping the shared-truth boundary clear?

## Two Tracks

```mermaid
flowchart TB
    U[User request or Matrix event] --> A{Agent2App track}
    A -->|inline view| V[agent2view]
    A -->|generated project| P[agent2project]

    V --> VH[Host renders app view]
    V --> VS[Host owns state/action boundary]

    P --> PF[File tree]
    P --> PC[Cargo.toml / src/main.rs]
    P --> PR[Studio runnable / package metadata]
```

## Agent2View

`agent2view` means the Agent output becomes a live view inside the Host. It is not an independent project and does not own a full process, file tree, or packaging lifecycle.

It is suitable for:

- aichat inline `runsplash` demos.
- Robrix2 weather or news cards.
- Robrix2 mission room control surfaces.

## Agent2Project

`agent2project` means the Agent generates an independent Makepad project or application artifact.

It must define at least:

- file tree;
- `Cargo.toml`;
- `src/main.rs`;
- resource files;
- state modules;
- event/action handlers;
- Studio runnable item;
- export and packaging metadata.

Agent2Project must not reuse the Agent2View runtime capability manifest. The former is a build artifact contract. The latter is a host runtime capability boundary.

## Out of Scope

This version does not define:

- code-generation project protocol;
- plugin marketplace;
- execution protocol for LLM-generated templates in the Robrix2 production path;
- multi-Agent resolver/template-author division of labor;
- automatic repair loops.
