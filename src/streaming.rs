use crate::{
    backend::KokoroBackend,
    error::AppError,
};
use axum::body::{Body, Bytes};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};


/// Create a PCM audio stream
pub async fn create_pcm_stream(
    backend: Arc<KokoroBackend>,
    text: String,
    voice: String,
    speed: f32,
    request_id: String,
) -> Result<Body, AppError> {
    // Chunk the text by sentences/phrases
    let chunks = chunk_text(&text);
    
    debug!(
        request_id = %request_id,
        num_chunks = chunks.len(),
        "Creating PCM stream with {} chunks",
        chunks.len()
    );

    let (tx, mut rx) = mpsc::unbounded_channel::<Result<Bytes, std::io::Error>>();

    // Spawn synthesis task
    tokio::spawn(async move {
        for (idx, chunk) in chunks.iter().enumerate() {
            debug!(
                request_id = %request_id,
                chunk_idx = idx,
                chunk_text = %chunk,
                "Synthesizing chunk"
            );

            match backend.synthesize(chunk, &voice, speed).await {
                Ok(audio) => {
                    // Convert f32 samples to PCM bytes
                    let pcm_bytes = samples_to_pcm_bytes(&audio.samples);
                    
                    if tx.send(Ok(Bytes::from(pcm_bytes))).is_err() {
                        warn!("Stream receiver dropped, stopping synthesis");
                        break;
                    }
                }
                Err(e) => {
                    error!(
                        request_id = %request_id,
                        chunk_idx = idx,
                        error = %e,
                        "Chunk synthesis failed"
                    );
                    let _ = tx.send(Err(std::io::Error::other(
                        format!("Synthesis failed: {}", e)
                    )));
                    break;
                }
            }
        }

        info!(
            request_id = %request_id,
            "PCM stream synthesis complete"
        );
    });

    // Create body from receiver stream
    let stream = async_stream::stream! {
        while let Some(result) = rx.recv().await {
            yield result;
        }
    };

    Ok(Body::from_stream(stream))
}

/// Create a WAV audio stream
pub async fn create_wav_stream(
    backend: Arc<KokoroBackend>,
    text: String,
    voice: String,
    speed: f32,
    request_id: String,
) -> Result<Body, AppError> {
    // For WAV streaming, we need to:
    // 1. Write WAV header first
    // 2. Stream PCM chunks
    // 3. Update header with final size (optional for streaming)

    let chunks = chunk_text(&text);
    
    debug!(
        request_id = %request_id,
        num_chunks = chunks.len(),
        "Creating WAV stream with {} chunks",
        chunks.len()
    );

    let (tx, mut rx) = mpsc::unbounded_channel::<Result<Bytes, std::io::Error>>();

    // Spawn synthesis task
    tokio::spawn(async move {
        const SAMPLE_RATE: u32 = 24000;
        const BITS_PER_SAMPLE: u16 = 16;
        const NUM_CHANNELS: u16 = 1;

        // Write WAV header (44 bytes, will be placeholder for streaming)
        let header = create_wav_header_placeholder(SAMPLE_RATE, BITS_PER_SAMPLE, NUM_CHANNELS);
        
        if tx.send(Ok(Bytes::from(header))).is_err() {
            warn!("Stream receiver dropped immediately");
            return;
        }

        let mut total_samples: u32 = 0;

        for (idx, chunk) in chunks.iter().enumerate() {
            debug!(
                request_id = %request_id,
                chunk_idx = idx,
                chunk_text = %chunk,
                "Synthesizing chunk for WAV"
            );

            match backend.synthesize(chunk, &voice, speed).await {
                Ok(audio) => {
                    // Convert f32 samples to PCM bytes
                    let pcm_bytes = samples_to_pcm_bytes(&audio.samples);
                    total_samples += audio.samples.len() as u32;
                    
                    if tx.send(Ok(Bytes::from(pcm_bytes))).is_err() {
                        warn!("Stream receiver dropped, stopping synthesis");
                        break;
                    }
                }
                Err(e) => {
                    error!(
                        request_id = %request_id,
                        chunk_idx = idx,
                        error = %e,
                        "Chunk synthesis failed"
                    );
                    let _ = tx.send(Err(std::io::Error::other(
                        format!("Synthesis failed: {}", e)
                    )));
                    break;
                }
            }
        }

        info!(
            request_id = %request_id,
            total_samples = total_samples,
            "WAV stream synthesis complete"
        );
    });

    // Create body from receiver stream
    let stream = async_stream::stream! {
        while let Some(result) = rx.recv().await {
            yield result;
        }
    };

    Ok(Body::from_stream(stream))
}

/// Chunk text into sentences/phrases for streaming
fn chunk_text(text: &str) -> Vec<String> {
    // Simple chunking by sentence-ending punctuation
    let delimiters = ['.', '!', '?', '\n'];
    let mut chunks = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        current.push_str(word);
        current.push(' ');

        // Check if word ends with any delimiter
        if delimiters.iter().any(|&d| word.ends_with(d)) {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                chunks.push(trimmed);
            }
            current.clear();
        }
    }

    // Add remaining text
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    // If no chunks found (no delimiters), return whole text
    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(text.trim().to_string());
    }

    chunks
}

/// Convert f32 samples [-1.0, 1.0] to 16-bit PCM bytes
fn samples_to_pcm_bytes(samples: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&int_sample.to_le_bytes());
    }
    
    bytes
}

/// Create WAV header placeholder for streaming
fn create_wav_header_placeholder(sample_rate: u32, bits_per_sample: u16, num_channels: u16) -> Vec<u8> {
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample / 8) as u32;
    let block_align = num_channels * (bits_per_sample / 8);
    
    let mut header = Vec::with_capacity(44);
    
    // RIFF chunk descriptor
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // File size (unknown for streaming)
    header.extend_from_slice(b"WAVE");
    
    // fmt sub-chunk
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
    header.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (1 for PCM)
    header.extend_from_slice(&num_channels.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&block_align.to_le_bytes());
    header.extend_from_slice(&bits_per_sample.to_le_bytes());
    
    // data sub-chunk
    header.extend_from_slice(b"data");
    header.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // Subchunk2Size (unknown for streaming)
    
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let text = "Hello world! This is a test. How are you?";
        let chunks = chunk_text(text);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "Hello world!");
        assert_eq!(chunks[1], "This is a test.");
        assert_eq!(chunks[2], "How are you?");
    }

    #[test]
    fn test_chunk_text_no_delimiters() {
        let text = "Hello world this is a test";
        let chunks = chunk_text(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world this is a test");
    }

    #[test]
    fn test_samples_to_pcm_bytes() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let bytes = samples_to_pcm_bytes(&samples);
        assert_eq!(bytes.len(), 10); // 5 samples * 2 bytes
        
        // Check that 0.0 maps to 0
        assert_eq!(bytes[0..2], [0, 0]);
        
        // Check that 1.0 maps to i16::MAX
        let max_val = i16::MAX.to_le_bytes();
        assert_eq!(bytes[6..8], max_val);
        
        // Check that -1.0 maps to i16::MIN
        let min_val = i16::MIN.to_le_bytes();
        assert_eq!(bytes[8..10], min_val);
    }

    #[test]
    fn test_wav_header() {
        let header = create_wav_header_placeholder(24000, 16, 1);
        assert_eq!(header.len(), 44);
        
        // Check RIFF header
        assert_eq!(&header[0..4], b"RIFF");
        assert_eq!(&header[8..12], b"WAVE");
        
        // Check fmt chunk
        assert_eq!(&header[12..16], b"fmt ");
        
        // Check data chunk
        assert_eq!(&header[36..40], b"data");
    }
}