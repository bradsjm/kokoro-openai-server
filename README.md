# Kokoro OpenAI Server

An OpenAI-compatible TTS (Text-to-Speech) server implementation using the Kokoro model, written in pure Rust.

## Features

- **OpenAI API Compatible**: Drop-in replacement for OpenAI's `/v1/audio/speech` endpoint
- **Pure Rust**: No external dependencies, strict Rust-only implementation
- **Streaming Support**: Real-time audio streaming with chunked transfer encoding
- **Multiple Voices**: 49 voices across 6 languages (English, Chinese, Japanese, Spanish, French, Hindi, Italian, Portuguese)
- **Apple Silicon Optimized**: CoreML execution provider support for Metal acceleration on macOS
- **Strict Format Support**: WAV and PCM formats only (no external codec dependencies)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/kokoro-openai-server.git
cd kokoro-openai-server

# Build (with CoreML support for macOS)
cargo build --release

# Run the server
./target/release/kokoro-openai-server

# Or with custom configuration
./target/release/kokoro-openai-server --port 8080 --workers 4
```

## Configuration

Configuration is done via environment variables or command-line arguments (CLI takes precedence):

| Environment Variable | CLI Flag | Default | Description |
|---------------------|----------|---------|-------------|
| `HOST` | `--host` | `0.0.0.0` | Host address to bind |
| `PORT` | `--port` | `8000` | Port to listen on |
| `API_KEY` | `--api-key` | None | API key for authentication (optional) |
| `MODEL_PATH` | `--model-path` | Auto | Path to Kokoro ONNX model |
| `EXECUTION_PROVIDER` | `--execution-provider` | `auto` | Inference backend: auto, cpu, coreml, cuda |
| `WORKERS` | `--workers` | `2` | Number of parallel inference workers (1-8) |
| `MAX_INPUT_CHARS` | `--max-input-chars` | `4096` | Maximum input text length |

## API Endpoints

### Speech Generation

```bash
POST /v1/audio/speech
```

Request body:
```json
{
  "model": "tts-1",
  "input": "Hello, this is a test of the Kokoro text-to-speech system.",
  "voice": "af_alloy",
  "response_format": "wav",
  "speed": 1.0,
  "stream": false
}
```

Response: Audio file in requested format

### List Models

```bash
GET /v1/models
```

Returns: List of available models (`tts-1`, `kokoro`)

### List Voices

```bash
GET /v1/audio/voices
```

Returns: List of 49 available voices

## Voice Reference

### American English (af/am)
- `af_alloy`, `af_heart`, `af_nova`, `af_river`, `af_shimmer`
- `am_adam`, `am_echo`, `am_fenrir`, `am_onyx`, `am_puck`, `am_santa`

### British English (bf/bm)
- `bf_alice`, `bf_emma`, `bf_lily`
- `bm_daniel`, `bm_fable`, `bm_george`, `bm_lewis`

### Japanese (jf/jm)
- `jf_alpha`, `jf_gongitsune`, `jf_nezumi`, `jf_tebukuro`
- `jm_kumo`

### Chinese (zf/zm)
- `zf_xiaobei`, `zf_xiaoni`, `zf_xiaoxiao`, `zf_yunjian`, `zf_yunxia`, `zf_yunxi`
- `zm_yunjian`

### Spanish (ef/em)
- `ef_dora`
- `em_alex`, `em_santa`

### French (ff)
- `ff_siwis`

### Hindi (hf/hm)
- `hf_alpha`, `hf_beta`
- `hm_omega`, `hm_psi`

### Italian (if/im)
- `if_sara`
- `im_nicola`

### Portuguese (pf/pm)
- `pf_dora`
- `pm_alex`, `pm_santa`

## Response Formats

| Format | Content-Type | Description |
|--------|--------------|-------------|
| `wav` | `audio/wav` | WAV audio file |
| `pcm` | `audio/pcm` | Raw PCM 16-bit little-endian |

**Note**: Only `wav` and `pcm` are supported. Other formats (mp3, opus, aac, flac) will return a 400 error to maintain strict Rust-only dependencies.

## Streaming

Enable streaming by setting `stream: true`:

```bash
curl -X POST http://localhost:8000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tts-1",
    "input": "This will be streamed as it's generated.",
    "voice": "af_alloy",
    "response_format": "pcm",
    "stream": true
  }' \
  --output - | ffplay -
```

Streaming uses chunked transfer encoding for minimal latency.

## Building from Source

### Requirements

- Rust 1.75+
- For CoreML support: macOS with Apple Silicon

### Build Options

```bash
# Default (with CoreML on macOS)
cargo build --release

# CPU-only
cargo build --release --no-default-features --features cpu

# With CUDA (Linux)
cargo build --release --no-default-features --features cuda

# With DirectML (Windows)
cargo build --release --no-default-features --features directml
```

## Architecture

This project implements a clean modular architecture:

- **api.rs**: HTTP handlers and OpenAI-compatible endpoints
- **backend.rs**: ONNX Runtime integration with Kokoro model
- **config.rs**: Environment and CLI configuration
- **error.rs**: OpenAI-style error responses
- **streaming.rs**: Chunked audio streaming implementation
- **validation.rs**: Request validation and voice definitions

## Model Download

The server can automatically download the Kokoro model on first run, or you can provide your own:

```bash
# Set custom model path
export MODEL_PATH=/path/to/kokoro.onnx
./kokoro-openai-server
```

## Logging

Set `RUST_LOG` environment variable:

```bash
# Debug logging
RUST_LOG=debug ./kokoro-openai-server

# Specific module
RUST_LOG=kokoro_openai_server=debug,axum=warn ./kokoro-openai-server
```

## Testing

```bash
# Run tests
cargo test

# Run with more output
cargo test -- --nocapture
```

## Client Examples

### Python

```python
import requests

response = requests.post(
    "http://localhost:8000/v1/audio/speech",
    json={
        "model": "tts-1",
        "input": "Hello world!",
        "voice": "af_alloy",
        "response_format": "wav"
    }
)

with open("output.wav", "wb") as f:
    f.write(response.content)
```

### cURL

```bash
curl -X POST http://localhost:8000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "model": "tts-1",
    "input": "Hello world",
    "voice": "af_alloy",
    "response_format": "wav"
  }' \
  --output output.wav
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- [Kokoro](https://github.com/nicholasguimaraes/kokoro) - The TTS model
- [Kokoros](https://github.com/lucasjinreal/Kokoros) - Rust implementation reference
- [OpenAI](https://platform.openai.com/docs/guides/text-to-speech) - API specification