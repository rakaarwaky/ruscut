# Ruscut Project Context & Instructions

This document provides essential context and instructions for AI agents and developers working on the `ruscut` project.

## Project Overview
`ruscut` is a high-performance, AI-powered background remover featuring both a CLI and an interactive TUI. It is written in Rust (2024 edition) and leverages ONNX Runtime for local AI inference using the **BRIA RMBG-2.0** model.

### Key Technologies
- **Language:** Rust (Edition 2024)
- **AI Runtime:** ONNX Runtime (`ort` crate)
- **Image Processing:** `image`, `fast_image_resize`, `ndarray`
- **CLI/TUI:** `clap` (CLI), `ratatui` & `crossterm` (TUI), `dialoguer` (Interactive Wizard)
- **Inference Model:** BRIA RMBG-2.0 (1.02 GB high-precision model)

---

## Architectural Principles (AES)
The project strictly adheres to the **Architecture Enforcement System (AES)**, a clean architecture variant with 6 distinct layers.

### 1. Layer Definitions & Naming Conventions
Files (excluding `main.rs`, `lib.rs`, `mod.rs`) must follow a **three-word snake_case** naming pattern with specific suffixes:

| Layer | Path | Allowed Suffixes | Role |
|---|---|---|---|
| **Surfaces** | `src-rust/surfaces/` | `_handler`, `_controller`, `_page`, `_view`, `_store` | Entry points (CLI/TUI), UI logic. |
| **Agent** | `src-rust/agent/` | `_container`, `_orchestrator`, `_manager`, `_registry` | Composition root, DI wiring, workflow coordination. |
| **Capabilities** | `src-rust/capabilities/` | `_executor`, `_resolver`, `_analyzer`, `_processor` | Core business use cases and domain logic. |
| **Contract** | `src-rust/contract/` | `_port`, `_protocol`, `_aggregate` | Abstractions, interface definitions, data boundaries. |
| **Infrastructure** | `src-rust/infrastructure/` | `_adapter`, `_provider`, `_client`, `_loader` | Concrete tech implementations (ORT, HTTP, Filesystem). |
| **Taxonomy** | `src-rust/taxonomy/` | `_vo`, `_entity`, `_error`, `_event` | Domain types, Value Objects, constants. |

### 2. Dependency Flow
- **Inward Only:** Surfaces → Agent → Capabilities → Contract → Taxonomy.
- **Taxonomy** is the foundation and must have **zero** internal dependencies on other layers.
- **Contract** defines the shapes; **Infrastructure** implements **Ports**; **Capabilities** implement **Protocols**.
- **Agent** conducts the flow using Contract interfaces, never concrete Infrastructure.

### 3. Coding Standards
- **No Panics:** Avoid `unwrap()` and `panic!`. Use `Result`, `Option`, and the `?` operator.
- **No Warning Suppression:** `#[allow(...)]` is forbidden. Fix the underlying issue.
- **Stateless Orchestrators:** Orchestrators in the Agent layer must be stateless to ensure thread safety.
- **DI Container:** All infrastructure and capabilities must be wired in the `DependencyInjectionContainer`.

---

## Development Workflows

### Building
- **Standard Build:** `cargo build --release`

### Running
- **CLI Binary:** `./target/release/ruscut <INPUT> [OUTPUT]`
- **TUI Binary:** `./target/release/ruscut-tui`
- **Quick Install:** `./install.sh` (builds and installs both binaries to `~/.cargo/bin` or `~/.local/bin`).

### Testing & Validation
- **Run Tests:** `cargo test --workspace --all-targets --all-features`
- **Linting:** `cargo clippy --workspace --all-targets --all-features` (Warnings are treated as errors in CI).
- **Formatting:** `cargo fmt --all`

---

## Technical Details

### Model Management
- Models are downloaded from Hugging Face on first run or via `--force-download`.
- **Cache Location:**
    - Linux: `~/.cache/ruscut/`
    - macOS: `~/Library/Caches/ruscut/`
    - Windows: `%LOCALAPPDATA%\ruscut\`

### Hardware Acceleration
- Supports standard CPU execution via ONNX Runtime, and hardware-accelerated direct GPU execution on AMD Radeon RDNA2 architectures (such as RX 6800 XT) via a custom direct Vulkan Compute backend (`ash` crate), bypassing ROCm entirely.
- Direct Vulkan execution is triggered automatically if the `RUSCUT_DIRECT_GPU` or `RUSCUT_VULKAN` environment variable is detected.

### Multi-Binary Structure
The project produces two separate binaries defined in `Cargo.toml`:
1. `ruscut`: The scriptable CLI.
2. `ruscut-tui`: The interactive TUI wizard.
Both binaries share the same underlying architecture layers (Agent, Capabilities, etc.).
