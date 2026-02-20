use crate::config::Config;
use crate::validation::DEFAULT_SAMPLE_RATE;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Audio synthesis result
#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

/// Kokoro backend for TTS inference
pub struct KokoroBackend {
    /// TTS engine
    tts_engine: Arc<kokoros::tts::koko::TTSKoko>,
    /// Concurrency limiter
    semaphore: Arc<Semaphore>,
    /// Sample rate (Kokoro default is 24000)
    sample_rate: u32,
    /// Configured upper bound for concurrent synth jobs
    worker_limit: usize,
}

impl KokoroBackend {
    /// Initialize the backend
    pub async fn new(config: &Config) -> Result<Self> {
        info!("Initializing Kokoro backend...");

        // Determine model and voices paths
        let (model_path, voices_path) = if let Some(ref path) = config.model_path {
            let voices_path = path.parent().unwrap_or(path).join("voices.json");
            (path.clone(), voices_path)
        } else {
            // Use default path in cache directory
            let cache_dir = dirs::cache_dir()
                .context("Failed to determine cache directory")?
                .join("kokoro-openai-server")
                .join("models");

            std::fs::create_dir_all(&cache_dir)?;
            let model_path = cache_dir.join("kokoro.onnx");
            let voices_path = cache_dir.join("voices.json");
            (model_path, voices_path)
        };

        // Initialize TTS engine (async)
        let model_path_str = model_path.to_string_lossy().to_string();
        let voices_path_str = voices_path.to_string_lossy().to_string();

        let tts_engine =
            Arc::new(kokoros::tts::koko::TTSKoko::new(&model_path_str, &voices_path_str).await);

        info!("Backend initialized with {} workers", config.workers);

        Ok(Self {
            tts_engine,
            semaphore: Arc::new(Semaphore::new(config.workers)),
            sample_rate: DEFAULT_SAMPLE_RATE,
            worker_limit: config.workers,
        })
    }

    pub fn worker_limit(&self) -> usize {
        self.worker_limit
    }

    /// Check if backend is healthy
    pub async fn is_healthy(&self) -> bool {
        self.sample_rate > 0 && !self.semaphore.is_closed()
    }

    /// Synthesize speech from text
    pub async fn synthesize(
        &self,
        text: &str,
        voice_id: &str,
        speed: f32,
        initial_silence: Option<usize>,
    ) -> Result<AudioData> {
        // Acquire permit for concurrent limit
        let _permit = self
            .semaphore
            .acquire()
            .await
            .context("Failed to acquire inference permit")?;

        debug!(
            voice_id = %voice_id,
            text_chars = text.chars().count(),
            "Synthesizing speech"
        );

        // Clone data for the blocking task
        let tts_engine = self.tts_engine.clone();
        let text = text.to_string();
        let voice_id = voice_id.to_string();
        let sample_rate = self.sample_rate;

        // Run inference in blocking task
        let samples = tokio::task::spawn_blocking(move || {
            match tts_engine.tts_raw_audio(
                &text,
                "en-us", // Default language
                &voice_id,
                speed,
                initial_silence,
                None, // request_id
                None, // instance_id
                None, // chunk_number
            ) {
                Ok(audio) => Ok(audio),
                Err(e) => Err(anyhow::anyhow!("TTS inference failed: {}", e)),
            }
        })
        .await
        .context("Inference task panicked")?
        .context("Inference failed")?;

        Ok(AudioData {
            samples,
            sample_rate,
        })
    }
}
