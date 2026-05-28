# ruscut

A high-performance, AI-powered background remover CLI and TUI tool written in Rust.

`ruscut` leverages ONNX Runtime (`ort`) and the state-of-the-art **BRIA RMBG-2.0** model to perform pixel-perfect, local background removal. The application ships two standalone binaries — a scriptable CLI (`ruscut`) and a guided interactive TUI (`ruscut-tui`) — and is built on a strict **AES (Agentic Engineering System) Clean Architecture** to ensure clean layer separation, loose coupling, and high testability.

---

## vs. The Most Popular Competitor

The most widely used open-source background removal tool is **rembg** (21,700+ GitHub stars). Below is a direct comparison:

| Feature                    | **ruscut** (This project)       | **rembg** (Python)      | **remove.bg** (SaaS) |
| -------------------------- | ------------------------------------- | ----------------------------- | -------------------------- |
| **Language**         | Rust                                  | Python                        | Cloud API                  |
| **Runtime required** | None — standalone binary             | Python 3 + pip                | Internet + API key         |
| **Privacy**          | 100% local, zero upload               | 100% local                    | Images uploaded to cloud   |
| **Cost**             | Free, open-source                     | Free, open-source             | Paid (per image)           |
| **Interactive TUI**  | Yes —`ruscut-tui`                  | No                            | No                         |
| **Install**          | `./install.sh` or `cargo install` | `pip install rembg`         | API key signup             |
| **Binary size**      | ~5 MB (Rust binary)                   | ~150 MB+ (Python env)         | N/A                        |
| **Cold start speed** | Fast (compiled native code)           | Slow (Python interpreter)     | Depends on network         |
| **Memory usage**     | Low (Rust zero-cost abstractions)     | High (Python overhead)        | N/A                        |
| **Cross-compile**    | Single `cargo build --release`      | Requires Docker / pyinstaller | N/A                        |
| **Works offline**    | Yes (after model download)            | Yes (after model download)    | No                         |
| **AI Model**         | BRIA RMBG-2.0 (ONNX)                  | U2Net / ISNET (ONNX)          | Proprietary                |
| **Model variants**   | Full (Single Auto Model)              | Single model                  | Managed                    |

**Key Advantage**: `ruscut` is a zero-dependency standalone binary. No Python, no `pip`, no `virtualenv`. Download and run.

---

## Features

- **Dual Interface**: Ships both a headless CLI (`ruscut`) for scripting and an interactive TUI wizard (`ruscut-tui`) for guided, menu-driven usage.
- **Local AI Inference**: Fast background removal using ONNX Runtime. No external APIs or internet access required after the model is downloaded.
- **Single High-Precision Model**: Exclusively uses the **BRIA RMBG-2.0 model (1.02 GB)** to ensure maximum accuracy and pixel-perfect quality out-of-the-box, without requiring complex parameter configurations.
- **Hardware Acceleration**: High-performance resizing using `fast_image_resize` and hardware-accelerated direct GPU execution on AMD Radeon RDNA2 architectures (such as RX 6800 XT) via a custom direct Vulkan Compute backend (`ash` crate), bypassing ROCm entirely.
- **Auto Cache Management**: Automatic downloading and local caching of Hugging Face ONNX assets silently.
- **Strict Architectural Integrity**: 100/100 AES architectural compliance score. Zero bypass, zero cyclic dependencies, and zero layer boundary violations.

---

## Prerequisites

Before installation, ensure your system meets the following requirements:

| Requirement    | Minimum                   | Notes                                                                                                         |
| :------------- | :------------------------ | :------------------------------------------------------------------------------------------------------------ |
| OS             | Linux x64 / Windows x64   | Requires Vulkan 1.3 runtime environment                                                                       |
| Hardware (GPU) | Vulkan 1.3 compatible GPU | Strictly required. CPU fallback is disabled. Prioritizes discrete AMD GPUs (Navi 21 / AMD Radeon RX 6800 XT). |
| RAM            | 4 GB                      | 8 GB recommended for model loading and frame processing                                                       |
| Disk Space     | 2 GB                      | Includes model cache (~1.02 GB for BRIA RMBG-2.0)                                                             |
| Internet       | Required once             | Only for initial model download from Hugging Face                                                             |
| Rust           | 1.75+                     | Only needed if compiling from source                                                                          |

---

## Installation

### Option 1: Automated Installer (Recommended — Linux/macOS)

Downloads the latest pre-built binaries from GitHub Releases, verifies SHA-256 checksums, and installs to `~/.cargo/bin`:

```bash
# Install latest release
./scripts/install.sh

# Install a specific version
./scripts/install.sh --version=0.1.4

# Show help
./scripts/install.sh --help
```

This installs **both** `ruscut` (CLI) and `ruscut-tui` (Interactive TUI).

Requirements: `curl`, `jq`

### Option 2: Build from Source (Development)

Runs format and clippy quality gates, then compiles and installs both binaries:

```bash
./scripts/dev.sh
```

Requirements: Rust toolchain (`cargo`, `rustfmt`, `clippy`)

### Option 3: Download Pre-built Binary (No Rust Required)

Download the latest release binary for your platform from the [GitHub Releases](https://github.com/rakaarwaky/ruscut/releases) page:

| Platform            | CLI Binary                    | TUI Binary                        |
| ------------------- | ----------------------------- | --------------------------------- |
| Linux x86_64        | `ruscut-linux-x86_64`       | `ruscut-tui-linux-x86_64`       |
| Linux ARM64         | `ruscut-linux-arm64`        | `ruscut-tui-linux-arm64`        |
| macOS Intel         | `ruscut-macos-x86_64`       | `ruscut-tui-macos-x86_64`       |
| macOS Apple Silicon | `ruscut-macos-arm64`        | `ruscut-tui-macos-arm64`        |
| Windows x64         | `ruscut-windows-x86_64.exe` | `ruscut-tui-windows-x86_64.exe` |

Each release includes a `checksums.txt` with SHA-256 hashes for supply-chain verification.

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
  -f, --force-download      Force re-download the model from Hugging Face
  -h, --help                Print help
  -V, --version             Print version
```

---

## Developer Workflow

All developer scripts live in `scripts/` and require a bash shell.

| Script                 | Purpose                                                      | Prerequisites                              |
| ---------------------- | ------------------------------------------------------------ | ------------------------------------------ |
| `scripts/dev.sh`     | Quality gates (fmt + clippy) then build & install locally    | Rust toolchain                             |
| `scripts/bump.sh`    | Bump version locally — no push, no tag, no publish          | `cargo-release`                          |
| `scripts/release.sh` | Full release: changelog → bump → tag → push (triggers CI) | `cargo-release`, optional: `git-cliff` |
| `scripts/install.sh` | Download & install pre-built binaries from GitHub Releases   | `curl`, `jq`                           |

### Typical development cycle

```bash
# 1. Make changes, then build and install locally
./scripts/dev.sh

# 2. Test the install
ruscut --help

# 3. Bump version locally (no push) to preview what the release will look like
./scripts/bump.sh patch    # 0.1.4 → 0.1.5

# 4. When ready to release — this creates a tag and pushes, triggering GitHub Actions
./scripts/release.sh patch
```

### Installing prerequisites

```bash
# cargo-release (required for bump.sh and release.sh)
cargo install cargo-release

# git-cliff (optional — generates CHANGELOG.md in release.sh)
cargo install git-cliff

# jq (required for install.sh)
sudo apt install jq   # Linux
brew install jq       # macOS
```

---

## Architecture Specification (AES)

The project is structured according to the strict 6-layer Agentic Engineering System (AES):

```
┌─────────────────────────────────────────────────────────┐
│  SURFACES (cli_command_handler.rs)                      │  Parses CLI args, maps options to Taxonomy.
│           (tui_command_page.rs)                         │  Interactive TUI dashboard using ratatui.
├─────────────────────────────────────────────────────────┤
│  AGENT (dependency_injection_container.rs)              │  Assembles layers, registers dependencies.
│        (bg_remover_orchestrator.rs)                     │  Stateless workflow orchestrator.
├─────────────────────────────────────────────────────────┤
│  CAPABILITIES (removal_usecase_executor.rs)             │  Executes core use case, holds all image
│                                                         │  processing and timing/benchmarking logic.
├─────────────────────────────────────────────────────────┤
│  CONTRACT (removal_usecase_protocol.rs)                 │  Defines interface contracts, ports, and
│           (onnx_remover_port.rs, pci_bar_port.rs, etc.) │  transfer aggregates (data boundaries).
├─────────────────────────────────────────────────────────┤
│  INFRASTRUCTURE (onnx_remover_adapter.rs)               │  Concrete technical implementations: ORT, Vulkan
│                 (pci_bar_provider.rs, etc.)             │  BAR, PM4 loader, ring buffer, video adapter.
├─────────────────────────────────────────────────────────┤
│  TAXONOMY (removal_types_vo.rs, removal_transfer_vo.rs) │  Value objects, model types, and domain bounds.
└─────────────────────────────────────────────────────────┘
```

### Directory Structure

```
src-rust/
├── cli_main_entry.rs          # Binary 1: CLI composition root
├── tui_main_entry.rs          # Binary 2: TUI composition root
├── taxonomy/                  # Level 1: Domain types and Value Objects (VOs)
│   ├── removal_transfer_vo.rs
│   ├── removal_types_vo.rs
│   └── mod.rs
├── contract/                  # Level 2: Inter-layer interfaces (Ports & Protocols)
│   ├── bg_remover_aggregate.rs
│   ├── di_container_aggregate.rs
│   ├── direct_amdgpu_remover_port.rs
│   ├── model_downloader_port.rs
│   ├── onnx_remover_port.rs
│   ├── pci_bar_port.rs
│   ├── pm4_packet_port.rs
│   ├── removal_usecase_protocol.rs
│   ├── ring_buffer_port.rs
│   ├── video_processor_port.rs
│   ├── vulkan_compute_port.rs
│   └── mod.rs
├── capabilities/              # Level 3: Use case implementations
│   ├── removal_usecase_executor.rs
│   └── mod.rs
├── infrastructure/            # Level 4: Concrete technology adaptors (Flat structure!)
│   ├── direct_amdgpu_remover_adapter.rs
│   ├── ffmpeg_video_adapter.rs
│   ├── huggingface_model_adapter.rs
│   ├── onnx_remover_adapter.rs
│   ├── pci_bar_provider.rs
│   ├── pm4_packet_loader.rs
│   ├── ring_buffer_provider.rs
│   ├── vulkan_compute_provider.rs
│   └── mod.rs
├── agent/                     # Level 5: Dependency injection and coordination
│   ├── dependency_injection_container.rs
│   ├── bg_remover_orchestrator.rs
│   └── mod.rs
└── surfaces/                  # Level 6: Surface handlers (CLI and TUI)
    ├── cli_command_handler.rs
    ├── tui_command_page.rs
    ├── tui_state_store.rs
    ├── tui_view_controller.rs
    └── mod.rs
```

### Dependency Rules

- **Inward Flow Only**: Surfaces → Agent → Capabilities → Contract → Taxonomy.
- **Technical Isolation**: The capabilities layer has zero knowledge of ONNX Runtime (`ort`) or HTTP (`reqwest`). It communicates solely through interfaces defined in `contract`.
- **Stateless Orchestration**: The `BgRemoverOrchestrator` in the Agent layer holds no state, ensuring safe concurrent invocation.

---

## Model Cache Location

Downloaded models are automatically cached to avoid re-downloading:

| OS      | Cache Path                   |
| ------- | ---------------------------- |
| Linux   | `~/.cache/ruscut/`         |
| macOS   | `~/Library/Caches/ruscut/` |
| Windows | `%LOCALAPPDATA%\ruscut\`   |

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

The underlying AI model BRIA RMBG-2.0 is subject to its own license conditions. Please check [Bria AI](https://bria.ai/) licensing guidelines for commercial purposes.
