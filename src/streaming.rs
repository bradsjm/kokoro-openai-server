use crate::{backend::KokoroBackend, error::AppError, validation::DEFAULT_SAMPLE_RATE};
use axum::body::{Body, Bytes};
use regex::Regex;
use std::sync::{Arc, LazyLock};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

const STREAM_CHANNEL_CAPACITY: usize = 8;
const BREAK_WORDS: &[&str] = &[
    "and", "or", "but", "&", "because", "if", "since", "though", "although", "however", "which",
];

/// Create a PCM audio stream
pub async fn create_pcm_stream(
    backend: Arc<KokoroBackend>,
    text: String,
    voice: String,
    speed: f32,
    initial_silence: Option<usize>,
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

    let (tx, mut rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(STREAM_CHANNEL_CAPACITY);

    // Spawn synthesis task
    tokio::spawn(async move {
        for (idx, chunk) in chunks.iter().enumerate() {
            let chunk_silence = if idx == 0 { initial_silence } else { None };
            debug!(
                request_id = %request_id,
                chunk_idx = idx,
                chunk_text = %chunk,
                "Synthesizing chunk"
            );

            match backend
                .synthesize(chunk, &voice, speed, chunk_silence)
                .await
            {
                Ok(audio) => {
                    // Convert f32 samples to PCM bytes
                    let pcm_bytes = samples_to_pcm_bytes(&audio.samples);

                    if tx.send(Ok(Bytes::from(pcm_bytes))).await.is_err() {
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
                    let _ = tx
                        .send(Err(std::io::Error::other(format!(
                            "Synthesis failed: {}",
                            e
                        ))))
                        .await;
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
    initial_silence: Option<usize>,
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

    let (tx, mut rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(STREAM_CHANNEL_CAPACITY);

    // Spawn synthesis task
    tokio::spawn(async move {
        const BITS_PER_SAMPLE: u16 = 16;
        const NUM_CHANNELS: u16 = 1;

        // Write WAV header (44 bytes, will be placeholder for streaming)
        let header =
            create_wav_header_placeholder(DEFAULT_SAMPLE_RATE, BITS_PER_SAMPLE, NUM_CHANNELS);

        if tx.send(Ok(Bytes::from(header))).await.is_err() {
            warn!("Stream receiver dropped immediately");
            return;
        }

        let mut total_samples: u32 = 0;

        for (idx, chunk) in chunks.iter().enumerate() {
            let chunk_silence = if idx == 0 { initial_silence } else { None };
            debug!(
                request_id = %request_id,
                chunk_idx = idx,
                chunk_text = %chunk,
                "Synthesizing chunk for WAV"
            );

            match backend
                .synthesize(chunk, &voice, speed, chunk_silence)
                .await
            {
                Ok(audio) => {
                    // Convert f32 samples to PCM bytes
                    let pcm_bytes = samples_to_pcm_bytes(&audio.samples);
                    total_samples += audio.samples.len() as u32;

                    if tx.send(Ok(Bytes::from(pcm_bytes))).await.is_err() {
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
                    let _ = tx
                        .send(Err(std::io::Error::other(format!(
                            "Synthesis failed: {}",
                            e
                        ))))
                        .await;
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
    let mut chunks = split_text_into_speech_chunks(text, 10);
    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(text.trim().to_string());
    }
    chunks
}

fn split_text_into_speech_chunks(text: &str, words_per_chunk: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut word_count = 0;

    for word in text.split_whitespace() {
        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }

        let is_numbered_break = is_numbered_list_item(word);

        if is_numbered_break && !current_chunk.is_empty() {
            let trimmed = current_chunk.trim().to_string();
            if !trimmed.is_empty() {
                chunks.push(trimmed);
            }
            current_chunk.clear();
            word_count = 0;
        }

        current_chunk.push_str(word);
        word_count += 1;

        let ends_with_unconditional = word.ends_with('.')
            || word.ends_with('!')
            || word.ends_with('?')
            || word.ends_with(':')
            || word.ends_with(';');
        let ends_with_conditional = word.ends_with(',');

        if ends_with_unconditional
            || is_numbered_break
            || (ends_with_conditional && word_count >= words_per_chunk)
        {
            let trimmed = current_chunk.trim().to_string();
            if !trimmed.is_empty() {
                chunks.push(trimmed);
            }
            current_chunk.clear();
            word_count = 0;
        }
    }

    if !current_chunk.trim().is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }

    let mut final_chunks = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let threshold = 12;
        let use_punctuation = index < 2;
        let split_chunks = split_long_chunk_with_depth(chunk, threshold, use_punctuation, 0);
        final_chunks.extend(split_chunks);
    }

    for i in 0..final_chunks.len().saturating_sub(1) {
        let current = &final_chunks[i];
        let words: Vec<&str> = current.trim().split_whitespace().collect();
        if let Some(last_word) = words.last() {
            if BREAK_WORDS.contains(&last_word.to_lowercase().as_str()) && words.len() > 1 {
                let new_current = words[..words.len() - 1].join(" ");
                let next_chunk = &final_chunks[i + 1];
                let new_next = format!("{} {}", last_word, next_chunk);
                final_chunks[i] = new_current;
                final_chunks[i + 1] = new_next;
            }
        }
    }

    final_chunks.retain(|chunk| !chunk.trim().is_empty());
    final_chunks
}

fn is_numbered_list_item(word: &str) -> bool {
    static NUMBERED_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\(?[0-9]+[.\)\:],?$").expect("valid regex"));
    NUMBERED_REGEX.is_match(word)
}

fn split_long_chunk_with_depth(
    chunk: &str,
    threshold: usize,
    use_punctuation: bool,
    depth: usize,
) -> Vec<String> {
    if depth >= 3 {
        return vec![chunk.to_string()];
    }

    let words: Vec<&str> = chunk.split_whitespace().collect();
    if words.len() < threshold {
        return vec![chunk.to_string()];
    }

    let center = words.len() / 2;

    if use_punctuation {
        if let Some(pos) = find_closest_punctuation(&words, center, &[","]) {
            if pos >= 3 && pos < words.len() {
                let first_chunk = words[..pos].join(" ");
                let second_chunk = words[pos..].join(" ");
                let mut result = Vec::new();
                result.extend(split_long_chunk_with_depth(
                    &first_chunk,
                    threshold,
                    use_punctuation,
                    depth + 1,
                ));
                result.extend(split_long_chunk_with_depth(
                    &second_chunk,
                    threshold,
                    use_punctuation,
                    depth + 1,
                ));
                return result;
            }
        }
    }

    if let Some(pos) = find_closest_break_word(&words, center, BREAK_WORDS) {
        if pos >= 3 && pos < words.len() {
            let first_chunk = words[..pos].join(" ");
            let second_chunk = words[pos..].join(" ");
            let mut result = Vec::new();
            result.extend(split_long_chunk_with_depth(
                &first_chunk,
                threshold,
                use_punctuation,
                depth + 1,
            ));
            result.extend(split_long_chunk_with_depth(
                &second_chunk,
                threshold,
                use_punctuation,
                depth + 1,
            ));
            return result;
        }
    }

    vec![chunk.to_string()]
}

fn find_closest_punctuation(words: &[&str], center: usize, punctuation: &[&str]) -> Option<usize> {
    let mut closest_pos = None;
    let mut min_distance = usize::MAX;

    for (i, word) in words.iter().enumerate() {
        if punctuation.iter().any(|p| word.ends_with(p)) {
            let distance = if i < center { center - i } else { i - center };
            if distance < min_distance {
                min_distance = distance;
                closest_pos = Some(i + 1);
            }
        }
    }

    closest_pos
}

fn find_closest_break_word(words: &[&str], center: usize, break_words: &[&str]) -> Option<usize> {
    let mut closest_pos = None;
    let mut min_distance = usize::MAX;

    for (i, word) in words.iter().enumerate() {
        if break_words.contains(&word.to_lowercase().as_str()) {
            let distance = if i < center { center - i } else { i - center };
            if distance < min_distance {
                min_distance = distance;
                closest_pos = Some(i);
            }
        }
    }

    closest_pos
}

/// Convert f32 samples [-1.0, 1.0] to 16-bit PCM bytes
fn samples_to_pcm_bytes(samples: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);

    for &sample in samples {
        let int_sample = pcm_i16_from_f32(sample);
        bytes.extend_from_slice(&int_sample.to_le_bytes());
    }

    bytes
}

fn pcm_i16_from_f32(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    if clamped <= -1.0 {
        i16::MIN
    } else {
        (clamped * i16::MAX as f32).round() as i16
    }
}

/// Create WAV header placeholder for streaming
fn create_wav_header_placeholder(
    sample_rate: u32,
    bits_per_sample: u16,
    num_channels: u16,
) -> Vec<u8> {
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
    fn test_chunk_text_numbered_list() {
        let text = "1. First item 2. Second item";
        let chunks = chunk_text(text);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "1. First item");
        assert_eq!(chunks[1], "2. Second item");
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
