# Agent Layer Specification

This document details the design, responsibilities, and implementation of the **Agent Layer** (`src-rust/agent/`) in `ruscut`. 

In the Architecture Enforcement System (AES) design pattern, the Agent layer acts as the system's "brain" and "Composition Root." It is situated at Level 5, bridging high-level UI/CLI surfaces (Level 6) with core Capabilities (Level 3) and Infrastructure adapters (Level 4).

---

## Architectural Role and Purpose

The Agent layer has two distinct responsibilities segregated into separate components:

1. **Wiring and Dependency Assembly** (The "Electrician" Role)
2. **Workflow Coordination** (The "Conductor" Role)

By separating wiring from orchestration, the system ensures that domain logic remains stateless, modular, and completely decoupled from technical dependencies.

---

## Layer Components

```
         ┌────────────────────────────────────────────────────┐
         │  SURFACES (L6) — Two Entry Points                 │
         │                                                    │
         │  cli_main_entry.rs  →  CliCommandHandler          │
         │  tui_main_entry.rs  →  TuiCommandHandler          │
         └──────────────────────────┬─────────────────────────┘
                                    │
                                    │ (Instantiates Container & Conductor)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ AGENT LAYER (L5 Brain)                                                 │
│                                                                        │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │  DependencyInjectionContainer (The Electrician)                │   │
│   │  - Instantiates Infrastructure adapters (L4)                    │   │
│   │  - Assembles Capabilities use-cases (L3)                       │   │
│   │  - Exposes resolved use-case protocols                        │   │
│   └──────────────────────────────┬─────────────────────────────────┘   │
│                                  │                                     │
│                                  ▼                                     │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │  BgRemoverOrchestrator (The Conductor)                         │   │
│   │  - Holds no system state                                       │   │
│   │  - Takes a Contract Protocol and triggers execution           │   │
│   └────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
```

### 1. The Dependency Injection Container

Implemented in `dependency_injection_container.rs` as the struct `DependencyInjectionContainer`.

- **Role**: acts as the structural wiring harness.
- **Responsibilities**:
  - Instantiates concrete Infrastructure adapters (e.g., `HuggingfaceModelAdapter` and `OnnxRemoverAdapter`).
  - Assembles and wires up core Capabilities (`RemovalUseCase`).
  - Exposes resolved inbound capabilities wrapped in `std::sync::Arc` pointers conforming to Contract Protocols.
- **Architectural Constraints**:
  - **No Domain Logic**: The container is purely structural. It is prohibited from executing domain rules, evaluating business logic, or invoking filesystem/model I/O directly.
  - **Lazy or Eager Initialization Only**: The constructor only executes component assembly.

### 2. The Orchestrator

Implemented in `bg_remover_orchestrator.rs` as the struct `BgRemoverOrchestrator`.

- **Role**: Acts as the workflow conductor.
- **Responsibilities**:
  - Receives options mapped into a L1 Taxonomy Value Object (`RemovalOptions`).
  - Conducts step-by-step stateless invocation of the business use case through the boundary Contract Protocol (`RemovalUseCaseProtocol`).
- **Architectural Constraints**:
  - **Stateless Execution**: The orchestrator holds zero execution state. It does not store intermediate files, results, or cached state between calls. This ensures safe concurrent and multi-threaded usage.
  - **Single Execution Goal**: Dedicated exclusively to orchestrating background removal.

---

## Dependency Boundaries and Governance Rules

To prevent architectural regression, the following strict rules are enforced at compile and lint time:

- **Surface Layer Decoupling**: Both the CLI (`cli_command_handler.rs`) and TUI (`tui_command_handler.rs`) surface handlers are only permitted to import from `agent`, `taxonomy`, and aggregate `contract` definitions. They have no direct access to capabilities or infrastructure.
- **Dual Binary, Single Architecture**: `cli_main_entry.rs` and `tui_main_entry.rs` are two separate Rust `[[bin]]` targets in `Cargo.toml`. Both share the same Agent, Capabilities, Contract, Infrastructure, and Taxonomy layers — only the Surface handler differs.
- **Technical Isolation**: The orchestrator is forbidden from directly importing infrastructure elements (such as `onnx_remover_adapter.rs`). It must only interact with use-cases via their abstract protocols defined in the `contract` layer.
- **Zero Raw Type Bypasses**: The orchestrator passes structured Value Objects (`RemovalOptions`) rather than unpacking primitives (such as strings or booleans) individually, maintaining high-level domain boundaries.
