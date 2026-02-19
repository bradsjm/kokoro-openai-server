# Kokoro OpenAI Server

[![Crates.io](https://img.shields.io/crates/v/kokoro-openai-server)](https://crates.io/crates/kokoro-openai-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

An OpenAI-compatible TTS (Text-to-Speech) server implementation using the Kokoro model, written in pure Rust. This server provides a drop-in replacement for OpenAI's audio speech endpoint, running entirely on your own hardware with no external API calls or dependencies.

## Features

- **OpenAI API Compatible**: Drop-in replacement for OpenAI's `/v1/audio/speech` endpoint
- **Pure Rust**: No external dependencies, strict Rust-only implementation
- **Streaming Support**: Real-time audio streaming with chunked transfer encoding
- **Multiple Voices**: 49 voices across 8 languages (English, Chinese, Japanese, Spanish, French, Hindi, Italian, Portuguese)
- **Hardware Acceleration**: CoreML execution provider support for Metal acceleration on macOS
- **Strict Format Support**: WAV and PCM formats only (no external codec dependencies)
- **API Key Authentication**: Optional Bearer token authentication for secure deployment
- **Flexible Configuration**: Configure via environment variables or command-line arguments
- **Health Monitoring**: Built-in health check endpoints for monitoring and orchestration

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [API Documentation](#api-documentation)
- [Examples](#examples)
- [Building from Source](#building-from-source)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Installation

### Quick Install (Binary Release)

Download the binary directly from the [releases page](https://github.com/bradsjm/kokoro-openai-server/releases).

### Build from Source

**Prerequisites:**
- **Rust toolchain**: Install via [rustup](https://rustup.rs/) (version 1.75 or later)
- **Git**: For cloning the repository

```bash
git clone https://github.com/bradsjm/kokoro-openai-server.git
cd kokoro-openai-server
cargo build --release
```

The compiled binary will be available at `target/release/kokoro-openai-server`.

## Quick Start

### 1. Start the Server

```bash
./target/release/kokoro-openai-server
```

Or with custom configuration:

```bash
./target/release/kokoro-openai-server --port 8080 --workers 4
```

### 2. Make Your First Request

```bash
curl -X POST http://localhost:8000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secret-key-here" \
  -d '{
    "model": "tts-1",
    "input": "Hello, this is a test of the Kokoro text-to-speech system.",
    "voice": "af_alloy",
    "response_format": "wav"
  }' \
  --output output.wav
```

That's it! The server will begin processing your text-to-speech requests immediately.

## Configuration

The server can be configured using environment variables or command-line arguments. Command-line arguments take precedence over environment variables.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `KOKORO_MODEL_PATH` | Auto | Path to Kokoro ONNX model (optional, auto-downloads if not provided) |
| `KOKORO_ACCELERATION` | `auto` | Hardware acceleration mode: `auto`, `cpu`, `coreml`, `cuda`, `directml` |
| `KOKORO_WORKERS` | `1` | Number of parallel inference workers (1-8) |
| `KOKORO_MAX_INPUT_CHARS` | `4096` | Maximum input text length in characters |
| `HOST` | `0.0.0.0` | Server host address |
| `PORT` | `8000` | Server port |
| `API_KEY` | - | Optional API key for authentication (if unset, no auth required) |

### Command-Line Arguments

```bash
cargo run --release -- --help
```

| Argument | Description |
|----------|-------------|
| `--host <HOST>` | Server host address |
| `--port <PORT>` | Server port |
| `--api-key <KEY>` | API key for authentication |
| `--model-path <PATH>` | Path to Kokoro ONNX model |
| `--acceleration <MODE>` | Hardware acceleration mode |
| `--workers <N>` | Number of parallel inference workers (1-8) |
| `--max-input-chars <N>` | Maximum input text length |

### Acceleration Modes

| Mode | Description |
|------|-------------|
| `auto` | Automatically select the best available provider |
| `cpu` | CPU-only inference |
| `coreml` | Apple CoreML (macOS with Apple Silicon) |
| `cuda` | NVIDIA CUDA (Linux/Windows with CUDA) |
| `directml` | DirectML (Windows) |

## API Documentation

The server implements OpenAI-compatible endpoints for text-to-speech.

### Endpoints Overview

- `GET /` - Server information
- `GET /health` - Health check endpoint
- `GET /v1` - API information
- `GET /v1/models` - List available models
- `GET /v1/audio/voices` - List available voices
- `POST /v1/audio/speech` - Generate speech from text

### POST /v1/audio/speech

Generates speech from input text.

**Request:**

```bash
curl -X POST http://localhost:8000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "model": "tts-1",
    "input": "Hello, this is a test.",
    "voice": "af_alloy",
    "response_format": "wav",
    "speed": 1.0,
    "stream": false
  }' \
  --output speech.wav
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| model | String | Yes | Model ID (`tts-1` or `kokoro`) |
| input | String | Yes | Text to convert to speech |
| voice | String | Yes | Voice ID (see [Voice Reference](#voice-reference)) |
| response_format | String | No | Audio format: `wav` or `pcm` (default: `wav`) |
| speed | Float | No | Speech speed multiplier (default: 1.0) |
| stream | Boolean | No | Stream audio as it's generated (default: false) |

**Response:** Audio file in requested format

### GET /v1/models

Lists available models.

```bash
curl http://localhost:8000/v1/models
```

Response:
```json
{
  "object": "list",
  "data": [
    {
      "id": "tts-1",
      "object": "model",
      "created": 1234567890,
      "owned_by": "kokoro-openai-server"
    },
    {
      "id": "kokoro",
      "object": "model",
      "created": 1234567890,
      "owned_by": "kokoro-openai-server"
    }
  ]
}
```

### GET /v1/audio/voices

Lists available voices.

```bash
curl http://localhost:8000/v1/audio/voices
```

Response: List of 49 available voices

## Examples

### Basic Speech Generation

```bash
curl -X POST http://localhost:8000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "model": "tts-1",
    "input": "Hello world!",
    "voice": "af_alloy",
    "response_format": "wav"
  }' \
  --output hello.wav
```

### Streaming Audio

```bash
curl -X POST http://localhost:8000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tts-1",
    "input": "This will be streamed as it is generated.",
    "voice": "af_alloy",
    "response_format": "pcm",
    "stream": true
  }' \
  --output - | ffplay -
```

### Custom Speed

```bash
curl -X POST http://localhost:8000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "model": "tts-1",
    "input": "This is spoken faster than normal.",
    "voice": "af_alloy",
    "speed": 1.5
  }' \
  --output fast.wav
```

### Health Check

```bash
curl http://localhost:8000/health
```

Response: `{"status":"ok"}`

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

### Running Tests

```bash
cargo test
cargo test --release
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Project Structure

```
kokoro-openai-server/
├── src/
│   ├── main.rs           # Server entry point
│   ├── config.rs         # Configuration management
│   ├── api.rs            # OpenAI-compatible API routes
│   ├── backend.rs        # ONNX Runtime integration
│   ├── error.rs          # Error handling
│   ├── streaming.rs      # Chunked audio streaming
│   └── validation.rs     # Request validation and voice definitions
├── Cargo.toml           # Rust package manifest
├── run.sh               # Convenience script
└── README.md            # This file
```

## Troubleshooting

### Model Loading Issues

**Problem:** Model fails to load.

**Solutions:**
- Check your internet connection (for auto-download)
- Verify `KOKORO_MODEL_PATH` points to a valid ONNX file
- Ensure the model file is not corrupted

### Audio Format Errors

**Problem:** "Unsupported format" error.

**Solutions:**
- Use only `wav` or `pcm` formats
- Other formats (mp3, opus, aac, flac) are not supported
- Convert your audio using FFmpeg if needed:
  ```bash
  ffmpeg -i input.mp3 -acodec pcm_s16le output.wav
  ```

### Memory Issues

**Problem:** Server runs out of memory.

**Solutions:**
- Reduce `KOKORO_WORKERS` to 1
- Increase system swap space
- Use a system with more RAM

### Slow Performance

**Problem:** Speech generation takes too long.

**Solutions:**
- Enable hardware acceleration with `--acceleration coreml` (macOS) or `--acceleration cuda` (Linux/Windows)
- Increase `KOKORO_WORKERS` (up to 8) for concurrent requests
- Ensure you're running the release build (`cargo run --release`)

### Authentication Errors

**Problem:** Receiving 401 Unauthorized errors.

**Solutions:**
- Ensure `API_KEY` is set on the server
- Include the `Authorization: Bearer <API_KEY>` header in requests
- Check for typos in the API key

### Port Already in Use

**Problem:** Server fails to start with "address already in use" error.

**Solutions:**
- Change the port: `export PORT=8001`
- Find and kill the process using the port:
  ```bash
  lsof -i :8000
  kill -9 <PID>
  ```

### Logging

Set `RUST_LOG` environment variable for debug output:

```bash
# Debug logging
RUST_LOG=debug ./kokoro-openai-server

# Specific module
RUST_LOG=kokoro_openai_server=debug,axum=warn ./kokoro-openai-server
```

### Behavior Notes

#### Request Validation

- **Model ID validation**: Only `tts-1` and `kokoro` are accepted
- **Voice validation**: Voice ID must be from the supported list
- **Input length**: Limited to `KOKORO_MAX_INPUT_CHARS` (default: 4096)
- **Required parameters**: Both `input` and `voice` are mandatory

#### Concurrency and Memory

- **Worker isolation**: Each worker loads its own model context
- **Memory scaling**: Memory usage scales linearly with `KOKORO_WORKERS`
- **Request queuing**: Requests exceeding parallelism limit are queued
- **Parallelism limits**: Minimum 1, maximum 8 workers

#### Authentication

- **Optional auth**: If `API_KEY` is not set, no authentication is required
- **Bearer token**: When enabled, all endpoints require `Authorization: Bearer <API_KEY>`
- **Consistent validation**: The same API key must be used for all authenticated requests

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- [Kokoro](https://github.com/nicholasguimaraes/kokoro) - The TTS model
- [Kokoros](https://github.com/lucasjinreal/Kokoros) - Rust implementation reference
- [OpenAI](https://platform.openai.com/docs/guides/text-to-speech) - API specification

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Support

For issues, questions, or contributions, please visit the [GitHub repository](https://github.com/bradsjm/kokoro-openai-server).
