---
version: 0.1.0
---
# Ruscut Background Remover Agent Skill

This document is a technical reference guide for AI agents to programmatically invoke, control, and integrate the `ruscut` command-line utility. 

## Integration Capability

The `ruscut` utility is a stateless command-line interface. AI agents can execute it as a subprocess to perform local, hardware-accelerated image background removal. It does not require API keys or external server requests after the chosen model is cached.

### Subprocess Invocation Signature

```bash
ruscut [OPTIONS] <INPUT> [OUTPUT]
```

---

## Technical Specifications

### CLI Arguments

- **`INPUT` (Required)**: Absolute or relative path to the source image (supported formats: JPG, JPEG, PNG, WebP).
- **`OUTPUT` (Optional)**: Path to write the output transparent PNG. Defaults to `<INPUT_DIR>/<INPUT_STEM>_no_bg.png`.

### Options

| Flag | Parameter | Type | Description |
|---|---|---|---|
| `-m`, `--model` | `<MODEL_PATH>` | Path | Path to a custom local `.onnx` model file (bypasses Hugging Face download). |
| `--fp16` | None | Flag | Configures use of the FP16 precision model (88.2 MB). |
| `--full` | None | Flag | Configures use of the full precision model (176 MB). |
| `-f`, `--force-download` | None | Flag | Re-downloads the model asset even if it is present in the cache. |

*Note: If neither `--fp16` nor `--full` is specified, the CLI defaults to the lightweight **Quantized** version (44.4 MB).*

---

## Agent Integration Patterns

### Python Integration Example

AI agents running inside a Python runtime environment can invoke background removal operations using `subprocess.run`:

```python
import subprocess
import pathlib
import json

def remove_background(
    input_path: str,
    output_path: str = None,
    precision: str = "quantized",
    force_download: bool = False
) -> dict:
    """
    Programmatic helper for AI agents to execute ruscut.
    
    precision: "quantized", "fp16", or "full"
    """
    input_file = pathlib.Path(input_path)
    if not input_file.exists():
        return {"status": "error", "message": f"Input file {input_path} not found"}

    cmd = ["ruscut"]  # Use installed binary from PATH
    # For development builds, use: "./target/release/ruscut"
    
    if precision == "fp16":
        cmd.append("--fp16")
    elif precision == "full":
        cmd.append("--full")
        
    if force_download:
        cmd.append("--force-download")
        
    cmd.append(str(input_file))
    
    if output_path:
        cmd.append(output_path)
        
    try:
        result = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True
        )
        return {
            "status": "success",
            "stdout": result.stdout,
            "stderr": result.stderr
        }
    except subprocess.CalledProcessError as e:
        return {
            "status": "error",
            "exit_code": e.returncode,
            "stdout": e.stdout,
            "stderr": e.stderr
        }
```

### Node.js Integration Example

```javascript
const { execFile } = require('child_process');
const path = require('path');

function removeBackground(inputPath, outputPath, options = {}) {
    return new Promise((resolve, reject) => {
        const args = [];
        if (options.fp16) args.push('--fp16');
        if (options.full) args.push('--full');
        if (options.forceDownload) args.push('--force-download');
        
        args.push(inputPath);
        if (outputPath) args.push(outputPath);

        execFile('ruscut', args, (error, stdout, stderr) => {  // Use installed binary from PATH
            // For development builds, use: './target/release/ruscut'
            if (error) {
                reject({ error, stdout, stderr });
                return;
            }
            resolve({ stdout, stderr });
        });
    });
}
```

## TUI Binary and Headless Agents

The `ruscut-tui` binary is designed for **interactive human use only**. It uses `dialoguer` to render terminal menus that require keyboard input from a live TTY session.

**AI agents must NOT invoke `ruscut-tui` as a subprocess.** It will hang waiting for terminal input.

For agent automation, always use the headless CLI binary:

```bash
# Correct for agents:
ruscut input.jpg output.png

# Never invoke from an agent script:
ruscut-tui  # Requires interactive TTY — will block
```

---

## Workflow Recommendations for AI Agents

1. **Format Validation**: Ensure the input image path exists before running. If the output path has a `.jpg` or `.jpeg` extension, warn the user/runtime that JPG does not support alpha transparency (resulting in a black or white background). Prefer `.png` or `.webp` for output.
2. **Model Caching**: The default cached models are saved inside `dirs::cache_dir()` in a folder named `ruscut`. If working in low-bandwidth sandboxes, do not pass `--force-download` to avoid re-downloading assets on every execution.
3. **Execution Monitoring**: The CLI writes descriptive logs to `stdout` and prints progress bars. In non-interactive scripting environments, progress indicators will automatically output in a clean, multi-line stream. Parse the final lines containing `SUKSES:` to verify operations.

---

## Error Codes and Troubleshooting

| Exit Code | Reason | Resolution |
|---|---|---|
| `0` | Success | The operation finished successfully. The output file has been created. |
| `1` | General Failure / Validation Error | The input file was missing, model downloading failed, or the ONNX runtime encountered an inference exception. Check `stderr` for detail. |

If a standard inference error occurs:
- Verify that ONNX Runtime supports your system architecture.
- Ensure there is write permission in the target output directory and read permission on the input file.
- Verify the cache directory is writable (usually `~/.cache/ruscut` on Linux systems).
