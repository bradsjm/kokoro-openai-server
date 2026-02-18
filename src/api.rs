use crate::{
    backend::KokoroBackend,
    error::{ApiResult, AppError},
    validation::{
        get_available_voices, validate_input, validate_model, validate_response_format,
        validate_speed, validate_voice, Voice,
    },
};
use axum::{
    body::{Body, Bytes},
    extract::{Json, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Request body for POST /v1/audio/speech
#[derive(Debug, Deserialize)]
pub struct SpeechRequest {
    /// Model ID ("tts-1" or "kokoro")
    pub model: String,
    /// Input text to synthesize
    pub input: String,
    /// Voice ID
    #[serde(default = "default_voice")]
    pub voice: String,
    /// Response format ("wav" or "pcm")
    #[serde(default = "default_response_format")]
    pub response_format: String,
    /// Speed multiplier (0.25 to 4.0, default 1.0)
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Whether to stream the response
    #[serde(default)]
    pub stream: Option<bool>,
}

fn default_voice() -> String {
    "af_alloy".to_string()
}

fn default_response_format() -> String {
    "mp3".to_string() // Will be rejected with 400, but matches OpenAI default
}

fn default_speed() -> f32 {
    1.0
}

/// Response body for GET /v1/models
#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Debug, Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

/// Response body for GET /v1/audio/voices
#[derive(Debug, Serialize)]
pub struct VoicesResponse {
    pub object: String,
    pub data: Vec<Voice>,
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub backend: Arc<KokoroBackend>,
    pub api_key: Option<String>,
    pub max_input_chars: usize,
}

/// Create the API router
pub fn create_router(backend: Arc<KokoroBackend>, api_key: Option<String>, max_input_chars: usize) -> Router {
    let state = AppState {
        backend,
        api_key,
        max_input_chars,
    };

    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/v1", get(root_handler))
        .route("/v1/models", get(list_models_handler))
        .route("/v1/audio/speech", post(speech_handler))
        .route("/v1/audio/voices", get(list_voices_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

/// Authentication middleware
async fn auth_middleware(
    State(state): State<AppState>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Response {
    // Skip auth for root and health endpoints
    let path = req.uri().path();
    if path == "/" || path == "/health" || path.starts_with("/v1/audio/voices") {
        return next.run(req).await;
    }

    // Check API key if configured
    if let Some(ref expected_key) = state.api_key {
        let auth_header = req
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let provided_key = &header[7..];
                if provided_key != expected_key {
                    warn!("Invalid API key provided");
                    return AppError::Unauthorized.into_response();
                }
            }
            _ => {
                warn!("Missing or invalid Authorization header");
                return AppError::Unauthorized.into_response();
            }
        }
    }

    next.run(req).await
}

/// Root handler
async fn root_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "Kokoro OpenAI TTS Server",
        "version": env!("CARGO_PKG_VERSION"),
        "docs": "/v1/models"
    }))
}

/// Health check handler
async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let healthy = state.backend.is_healthy().await;
    
    if healthy {
        (StatusCode::OK, Json(serde_json::json!({"status": "healthy"})))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "unhealthy"})),
        )
    }
}

/// List available models
async fn list_models_handler() -> ApiResult<impl IntoResponse> {
    let models = vec![
        Model {
            id: "tts-1".to_string(),
            object: "model".to_string(),
            created: 1704067200, // 2024-01-01
            owned_by: "kokoro".to_string(),
        },
        Model {
            id: "kokoro".to_string(),
            object: "model".to_string(),
            created: 1704067200,
            owned_by: "kokoro".to_string(),
        },
    ];

    Ok(Json(ModelsResponse {
        object: "list".to_string(),
        data: models,
    }))
}

/// List available voices
async fn list_voices_handler() -> impl IntoResponse {
    let voices = get_available_voices().clone();
    
    Json(VoicesResponse {
        object: "list".to_string(),
        data: voices,
    })
}

/// Text-to-speech handler
async fn speech_handler(
    State(state): State<AppState>,
    Json(req): Json<SpeechRequest>,
) -> ApiResult<impl IntoResponse> {
    let request_id = Uuid::new_v4().to_string();
    
    debug!(
        request_id = %request_id,
        model = %req.model,
        voice = %req.voice,
        format = %req.response_format,
        stream = ?req.stream,
        "Received speech request"
    );

    // Validate model
    let _model = validate_model(&req.model)?;

    // Validate input
    validate_input(&req.input, state.max_input_chars)?;

    // Validate response format (strict: wav and pcm only)
    let format = validate_response_format(&req.response_format)?;

    // Validate voice
    let voices = get_available_voices();
    let _voice = validate_voice(&req.voice, &voices)?;

    // Validate speed
    let speed = validate_speed(req.speed)?;

    // Check if streaming is requested
    let stream = req.stream.unwrap_or(false);

    if stream {
        // Streaming response
        let (content_type, body) = if format == "wav" {
            (
                "audio/wav",
                crate::streaming::create_wav_stream(
                    state.backend.clone(),
                    req.input,
                    req.voice,
                    speed,
                    request_id.clone(),
                )
                .await?,
            )
        } else {
            (
                "audio/pcm",
                crate::streaming::create_pcm_stream(
                    state.backend.clone(),
                    req.input,
                    req.voice,
                    speed,
                    request_id.clone(),
                )
                .await?,
            )
        };

        info!(
            request_id = %request_id,
            "Streaming response initiated"
        );

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header("Transfer-Encoding", "chunked")
            .header("X-Accel-Buffering", "no")
            .header("Cache-Control", "no-cache")
            .header("X-Request-Id", request_id)
            .body(body)
            .map_err(|_e| AppError::Internal)?)
    } else {
        // Non-streaming response
        let audio_data = state
            .backend
            .synthesize(&req.input,
                &req.voice,
                speed,
            )
            .await
            .map_err(|e| {
                error!("Synthesis failed: {}", e);
                AppError::Backend(e.to_string())
            })?;

        // Encode to requested format
        let (content_type, bytes) = if format == "wav" {
            (
                "audio/wav",
                encode_wav(&audio_data.samples,
                    audio_data.sample_rate,
                )?,
            )
        } else {
            (
                "audio/pcm",
                encode_pcm(&audio_data.samples),
            )
        };

        info!(
            request_id = %request_id,
            samples = audio_data.samples.len(),
            duration_ms = audio_data.samples.len() * 1000 / audio_data.sample_rate as usize,
            "Synthesis complete"
        );

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header("X-Request-Id", request_id)
            .body(Body::from(bytes))
            .map_err(|_| AppError::Internal)?)
    }
}

/// Encode float samples to WAV format
fn encode_wav(samples: &[f32], sample_rate: u32) -> Result<Bytes, AppError> {
    use hound::{WavSpec, WavWriter};
    use std::io::Cursor;

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut cursor, spec)
            .map_err(|_e| AppError::Internal)?;
        
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            writer.write_sample(int_sample)
                .map_err(|_e| AppError::Internal)?;
        }
        
        writer.finalize()
            .map_err(|_e| AppError::Internal)?;
    }

    Ok(Bytes::from(cursor.into_inner()))
}

/// Encode float samples to raw PCM (16-bit little-endian)
fn encode_pcm(samples: &[f32]) -> Bytes {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&int_sample.to_le_bytes());
    }
    
    Bytes::from(bytes)
}