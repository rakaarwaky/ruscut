# Changelog

All notable changes to the **Ruscut** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.8] - 2026-05-28

### Added
- **Enforced GPU-Only Execution**: Configured both CLI and TUI to strictly require Vulkan hardware compute capability. The application now exits cleanly with code `1` on start if a Vulkan-compatible GPU is unavailable, preventing CPU fallbacks.
- **Real Vulkan JIT Shader Integration**: Restored the actual Vulkan direct compute JIT Sigmoid activation shader (`SIGMOID_COMPUTE_SPIRV`) and memory transfers (`vulkan_map_memory`/`vulkan_unmap_memory`) for AMD RDNA2 GPU architecture (specifically optimized for AMD RX 6800 XT) inside the `DirectAmdgpuRemoverAdapter`.
- **Silent FFmpeg Video Adapter**: Redirected the `stdout` and `stderr` streams of the FFmpeg frame extractor and video encoder sub-processes to `std::process::Stdio::null()`, preventing lengthy FFmpeg build and metadata information from leaking into the terminal.

### Changed
- **Premium Console Logging Redirection**: Redirected all `tracing` instrumentation logs to a file located at `~/.cache/ruscut/ruscut.log`, leaving the CLI terminal pristine and dedicated solely to the progress bar and success indicators.
- **ROCm Bypass Integration**: Completely bypassed the thread-unsafe ROCm execution provider from the ONNX Runtime compilation setup in `OnnxRemoverAdapter`, resolving compilation issues and ensuring thread safety (`Send`/`Sync`) on the use case runner.

### Fixed
- **Index Out of Bounds Panic**: Resolved a crash on non-1024x1024 images by properly resizing the output model mask to match the original image's dimensions before running the mask application algorithm.
- **Safe DI Initialization**: Added safe `match` handling for `DependencyInjectionContainer::new()` in both `cli_main_entry.rs` and `tui_main_entry.rs` to catch initialization failures gracefully.
- **Clippy Warning-Errors**: Fixed `clippy::ineffective_open_options` warning-errors in `OpenOptions` by removing redundant `.write(true)` when `.append(true)` is present.

---

## [0.1.7] - 2026-05-28

### Fixed
- Corrected progress bars for model downloads.
- Improved live progress tracking for execution benchmarks.
- Removed duplicate model loading triggers during capabilities runs.

---

## [0.1.6] - 2026-05-28

### Fixed
- Updated `install.sh` to support automatic fallback compilation from source if GitHub Release artifacts are not yet generated.

---

## [0.1.3] - 2026-05-28

### Changed
- **Architectural Overhaul**: Migrated the codebase to clean Hexagonal Architecture (AES standard) with well-separated domain entities, use cases, ports, and adapters.
- Introduced `EngineNameVo` and `VulkanComputePort` for clean, decoupled hardware execution abstractions.

---

## [0.1.1] - 2026-05-28

### Added
- Upgraded the interactive terminal UI to a rich full-screen `ratatui` (v0.30) dashboard.

### Fixed
- Strengthened release workflow scripts and deployment resilience.

---

## [0.1.0] - 2026-05-28

### Added
- **Initial Release**: High-performance AI-powered background remover CLI and TUI.
- Integrated premium alpha edge matting and smoothstep edge sharpening algorithms.
- Translated all interactive messages, guides, and TUI fields to English for broad international usability.
