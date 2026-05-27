# ruscut

A high-performance, AI-powered background remover CLI and TUI tool written in Rust.

`ruscut` leverages ONNX Runtime (`ort`) and the state-of-the-art **BRIA RMBG-1.4** model to perform pixel-perfect, local background removal. The application ships two standalone binaries — a scriptable CLI (`ruscut`) and a guided interactive TUI (`ruscut-tui`) — and is built on a strict **AES (Architecture Enforcement System) Clean Architecture** to ensure clean layer separation, loose coupling, and high testability.

---

## vs. The Most Popular Competitor

The most widely used open-source background removal tool is **rembg** (21,700+ GitHub stars). Below is a direct comparison:

| Feature | **ruscut** (This project) | **rembg** (Python) | **remove.bg** (SaaS) |
|---|---|---|---|
| **Language** | Rust | Python | Cloud API |
| **Runtime required** | None — standalone binary | Python 3 + pip | Internet + API key |
| **Privacy** | 100% local, zero upload | 100% local | Images uploaded to cloud |
| **Cost** | Free, open-source | Free, open-source | Paid (per image) |
| **Interactive TUI** | Yes — `ruscut-tui` | No | No |
| **Install** | `./install.sh` or `cargo install` | `pip install rembg` | API key signup |
| **Binary size** | ~5 MB (Rust binary) | ~150 MB+ (Python env) | N/A |
| **Cold start speed** | Fast (compiled native code) | Slow (Python interpreter) | Depends on network |
| **Memory usage** | Low (Rust zero-cost abstractions) | High (Python overhead) | N/A |
| **Cross-compile** | Single `cargo build --release` | Requires Docker / pyinstaller | N/A |
| **Works offline** | Yes (after model download) | Yes (after model download) | No |
| **AI Model** | BRIA RMBG-1.4 (ONNX) | U2Net / ISNET (ONNX) | Proprietary |
| **Model variants** | Quantized / FP16 / Full | Single model | Managed |

**Key Advantage**: `ruscut` is a zero-dependency standalone binary. No Python, no `pip`, no `virtualenv`. Download and run.

---

## Features

- **Dual Interface**: Ships both a headless CLI (`ruscut`) for scripting and an interactive TUI wizard (`ruscut-tui`) for guided, menu-driven usage.
- **Local AI Inference**: Fast background removal using ONNX Runtime. No external APIs or internet access required after the model is downloaded.
- **Model Variety**: Supports three versions of the BRIA RMBG-1.4 model:
  - **Quantized (44.4 MB)**: Default option. Extremely fast, lightweight, and low resource usage.
  - **FP16 (88.2 MB)**: Good balance between size, speed, and precision.
  - **Full Precision (176 MB)**: Maximum precision and quality.
- **Hardware Acceleration**: High-performance resizing using `fast_image_resize` and hardware acceleration via ONNX Runtime (CPU/GPU).
- **Auto Cache Management**: Automatic downloading and local caching of Hugging Face ONNX assets with progress bar tracking.
- **Strict Architectural Integrity**: 100/100 AES architectural compliance score. Zero bypass, zero cyclic dependencies, and zero layer boundary violations.

---

## Prerequisites

Before installation, ensure your system meets the following requirements:

| Requirement | Minimum | Notes |
|---|---|---|
| OS | Linux x64 / macOS (Intel/Apple Silicon) / Windows x64 | Pre-built binaries available for all platforms |
| RAM | 2 GB | 4 GB recommended for Full Precision model |
| Disk Space | 500 MB | Includes model cache (~44–176 MB depending on variant) |
| Internet | Required once | Only for initial model download from Hugging Face |
| Rust | 1.75+ | Only needed if building from source |

---

## Installation

### Option 1: Quick Installer Script (Recommended — Linux/macOS)

Automatically checks dependencies, compiles both binaries in release mode, and places them in your local PATH:

```bash
./install.sh
```

This installs **both** `ruscut` (CLI) and `ruscut-tui` (Interactive TUI).

### Option 2: Cargo Install from crates.io

```bash
cargo install ruscut
```

### Option 3: Download Pre-built Binary (No Rust Required)

Download the latest release binary for your platform from the [GitHub Releases](https://github.com/rakaarwaky/ruscut/releases) page:

| Platform | CLI Binary | TUI Binary |
|---|---|---|
| Linux x86_64 | `ruscut-linux-x86_64` | `ruscut-tui-linux-x86_64` |
| Linux ARM64 | `ruscut-linux-arm64` | `ruscut-tui-linux-arm64` |
| macOS Intel | `ruscut-macos-x86_64` | `ruscut-tui-macos-x86_64` |
| macOS Apple Silicon | `ruscut-macos-arm64` | `ruscut-tui-macos-arm64` |
| Windows x64 | `ruscut-windows-x86_64.exe` | `ruscut-tui-windows-x86_64.exe` |

### Option 4: Manual Compilation

```bash
cargo build --release
# Binaries will be at:
# target/release/ruscut
# target/release/ruscut-tui
```

---

## Usage

### TUI Mode — Interactive Wizard (No Commands to Memorize)

Launch the guided interactive wizard. The TUI will prompt you for all required inputs step-by-step:

```bash
ruscut-tui
```

You only need to remember one command. The wizard handles everything else.

### CLI Mode — Headless Scripting

Remove the background of an image from the command line:

```bash
# Save to default output path (<input>_no_bg.png)
ruscut input.jpg

# Save to a custom output path
ruscut input.jpg output.png

# Use the FP16 precision model
ruscut --fp16 input.jpg

# Use the full precision model
ruscut --full input.jpg

# Force re-download the model
ruscut --force-download input.jpg

# Use a custom local .onnx model
ruscut --model /path/to/model.onnx input.jpg
```

### Full CLI Reference

```
Usage: ruscut [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   Path to input image file (e.g., JPG, PNG, WebP)
  [OUTPUT]  Path to output transparent PNG. Defaults to <input>_no_bg.png

Options:
  -m, --model <MODEL_PATH>  Path to a custom .onnx model file
      --fp16                Use the FP16 precision model (88.2 MB)
      --full                Use the full precision model (176 MB)
  -f, --force-download      Force re-download the model from Hugging Face
  -h, --help                Print help
  -V, --version             Print version
```

---

## Architecture Specification (AES)

The project is structured according to the strict 6-layer Architecture Enforcement System (AES):

```
┌─────────────────────────────────────────────────────────┐
│  SURFACES (cli_command_handler.rs)                      │  Parses CLI args, maps options to Taxonomy.
│           (tui_command_handler.rs)                      │  Interactive TUI wizard using dialoguer.
├─────────────────────────────────────────────────────────┤
│  AGENT (dependency_injection_container.rs)              │  Assembles layers, registers dependencies.
│        (bg_remover_orchestrator.rs)                     │  Stateless workflow orchestrator.
├─────────────────────────────────────────────────────────┤
│  CAPABILITIES (removal_usecase_executor.rs)             │  Executes core use case (orchestrates ports).
├─────────────────────────────────────────────────────────┤
│  CONTRACT (removal_usecase_protocol.rs)                 │  Defines interface contracts, ports, and
│           (background_remover_port.rs)                  │  transfer aggregates (data boundaries).
├─────────────────────────────────────────────────────────┤
│  INFRASTRUCTURE (onnx_remover_adapter.rs)               │  Concrete technical implementations: ORT,
│                 (huggingface_model_adapter.rs)          │  image operations, and HTTP downloads.
├─────────────────────────────────────────────────────────┤
│  TAXONOMY (removal_types_vo.rs)                         │  Value objects, model types, and domain bounds.
└─────────────────────────────────────────────────────────┘
```

### Directory Structure

```
src-rust/
├── cli_main_entry.rs          # Binary 1: CLI composition root
├── tui_main_entry.rs          # Binary 2: TUI composition root
├── taxonomy/                  # Level 1: Domain types and Value Objects (VOs)
│   ├── removal_types_vo.rs
│   └── mod.rs
├── contract/                  # Level 2: Inter-layer interfaces (Ports & Protocols)
│   ├── background_remover_port.rs
│   ├── model_downloader_port.rs
│   ├── removal_transfer_aggregate.rs
│   ├── removal_usecase_protocol.rs
│   └── mod.rs
├── capabilities/              # Level 3: Use case implementations
│   ├── removal_usecase_executor.rs
│   └── mod.rs
├── infrastructure/            # Level 4: Concrete technology adaptors
│   ├── huggingface_model_adapter.rs
│   ├── onnx_remover_adapter.rs
│   └── mod.rs
├── agent/                     # Level 5: Dependency injection and coordination
│   ├── dependency_injection_container.rs
│   ├── bg_remover_orchestrator.rs
│   └── mod.rs
└── surfaces/                  # Level 6: Surface handlers (CLI and TUI)
    ├── cli_command_handler.rs
    ├── tui_command_handler.rs
    └── mod.rs
```

### Dependency Rules

- **Inward Flow Only**: Surfaces → Agent → Capabilities → Contract → Taxonomy.
- **Technical Isolation**: The capabilities layer has zero knowledge of ONNX Runtime (`ort`) or HTTP (`reqwest`). It communicates solely through interfaces defined in `contract`.
- **Stateless Orchestration**: The `BgRemoverOrchestrator` in the Agent layer holds no state, ensuring safe concurrent invocation.

---

## Model Cache Location

Downloaded models are automatically cached to avoid re-downloading:

| OS | Cache Path |
|---|---|
| Linux | `~/.cache/ruscut/` |
| macOS | `~/Library/Caches/ruscut/` |
| Windows | `%LOCALAPPDATA%\ruscut\` |

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

The underlying AI model BRIA RMBG-1.4 is subject to its own license conditions. Please check [Bria AI](https://bria.ai/) licensing guidelines for commercial purposes.
