# AI Agent Developer Playbook (AGENT.md)

This document serves as the technical guidelines and architectural constraints for AI agents and developer tools working on the Ruscut codebase. All code modifications, refactorings, and integrations must strictly adhere to the specifications detailed below.

---

## 1. Hexagonal Architecture (AES Standard)

Ruscut is implemented using the **Agentic Engineering System (AES)** design pattern, enforcing a strict hexagonal separation between business logic and technical infrastructure. The directory structure is divided into six layers:

| Layer | Path | Description |
| :--- | :--- | :--- |
| **Level 6: Surfaces** | `src-rust/surfaces/` | User interfaces, specifically CLI command handlers and the full-screen Ratatui TUI. |
| **Level 5: Agent** | `src-rust/agent/` | Composition Root, comprising the `DependencyInjectionContainer` and the `BgRemoverOrchestrator`. |
| **Level 4: Infrastructure** | `src-rust/infrastructure/`| Hard technical adapters, including Vulkan Compute Engine, ONNX Runtime, HuggingFace downloader, and FFmpeg adapter. |
| **Level 3: Capabilities** | `src-rust/capabilities/` | Stateless business use cases, primarily `RemovalUseCase`. |
| **Level 2: Contract** | `src-rust/contract/` | Decoupled trait definitions and ports establishing boundary interfaces. |
| **Level 1: Taxonomy** | `src-rust/taxonomy/` | Domain entities, configuration models, Value Objects, and transfer models. |

### Architectural Boundary Rules
* **No Direct Leaks**: Surfaces (Level 6) are prohibited from directly accessing Infrastructure (Level 4) or Capabilities (Level 3). All interaction must pass through the Agent (Level 5) composition root.
* **Abstract Trait Boundaries**: The business use cases in Capabilities (Level 3) must only interact with Infrastructure (Level 4) adapters through the abstract boundary traits defined in the Contract (Level 2) layer.

---

## 2. Hardware Constraints (Vulkan GPU Acceleration)

Ruscut is designed to run exclusively on dedicated GPU hardware.

### No CPU Fallback
* **Strict GPU Dependency**: Execution on CPU is disabled. If a Vulkan-compatible GPU is unavailable or initialization fails, the application must immediately abort startup and exit with code `1`.
* **DI Container Implementation**: `DependencyInjectionContainer::new()` returns a `Result<Self>`. It must eagerly instantiate `VulkanComputeEngine` and return a descriptive error upon failure to prevent fallback.

### Vulkan Compute Engine (ash v0.38)
* **AMD Radeon RX 6800 XT Priority**: Prioritize discrete AMD GPUs (Navi 21 / RDNA2 architecture) using the `ash` binding library.
* **Production Shader Operations**: Mock or stub operations are prohibited in production adapters. `DirectAmdgpuRemoverAdapter` must map GPU memory (`vulkan_map_memory` and `vulkan_unmap_memory`) and dispatch the JIT Sigmoid activation shader (`SIGMOID_COMPUTE_SPIRV`).

### ONNX ROCm Bypass
* **Thread Safety Constraint**: The ONNX Runtime ROCm execution provider utilizes raw pointers (`NonNull`) which do not implement `Send` and `Sync`, preventing safe session caching in multi-threaded environments.
* **ROCm Bypassed**: ONNX Runtime is compiled in a standard CPU-only configuration inside `OnnxRemoverAdapter`. Subsequent tensor activations and post-processing steps are offloaded to the custom Vulkan GPU compute engine.

---

## 3. Terminal Console Output and Logging Protocols

The terminal output for both CLI and Ratatui TUI must remain clear and free of debugging outputs.

* **Log Redirection**: All internal `tracing` instrumentation logs (including GPU selection, model downloads, benchmarks, and ONNX setup) must be redirected to:
  `~/.cache/ruscut/ruscut.log`
* **Standard Streams Protection**: Writing debug or operational logs to `stdout` or `stderr` during background removal is prohibited.
* **Silent FFmpeg Subprocesses**: When processing video background removal, standard output and standard error streams of the FFmpeg subprocesses must be redirected to `std::process::Stdio::null()` to prevent terminal pollution.

---

## 4. Safety & Development Workflow

### Zero Panics Rule
* **Error Propagation**: Using `.unwrap()` or `.expect()` in non-test production code is prohibited. Propagate errors utilizing `anyhow::Result` and context-rich `.context()`.

### Compiler Warning Enforcement
* The local installation script treats compiler and clippy warnings as compilation errors.
* **Clippy standard**: All code must pass `cargo clippy --all-targets --all-features -- -D warnings` with zero warnings before completion.

### Verification Routine
Execute the following verification commands from the project root before final delivery:

```bash
# 1. Run local automated code quality & standards checking
auto-lint check

# 2. Run unit and integration tests
cargo test

# 3. Build optimized binaries and install locally
./scripts/dev.sh
```
