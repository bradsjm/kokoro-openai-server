use crate::config::Config;
use crate::error::{ApiResult, AppError};
use std::sync::LazyLock;

/// Valid response formats (strict Rust-only constraint)
pub const VALID_RESPONSE_FORMATS: [&str; 2] = ["wav", "pcm"];

/// Default sample rate for Kokoro TTS
pub const DEFAULT_SAMPLE_RATE: u32 = 24000;

/// All available Kokoro voices - lazily initialized once
pub static AVAILABLE_VOICES: LazyLock<Vec<Voice>> = LazyLock::new(|| {
    vec![
        Voice {
            id: "af_alloy".to_string(),
            name: "Alloy (Female, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "af_heart".to_string(),
            name: "Heart (Female, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "af_nova".to_string(),
            name: "Nova (Female, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "af_river".to_string(),
            name: "River (Female, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "af_shimmer".to_string(),
            name: "Shimmer (Female, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "am_adam".to_string(),
            name: "Adam (Male, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "am_echo".to_string(),
            name: "Echo (Male, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "am_fenrir".to_string(),
            name: "Fenrir (Male, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "am_onyx".to_string(),
            name: "Onyx (Male, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "am_puck".to_string(),
            name: "Puck (Male, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "am_santa".to_string(),
            name: "Santa (Male, American)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "bf_alice".to_string(),
            name: "Alice (Female, British)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "bf_emma".to_string(),
            name: "Emma (Female, British)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "bf_lily".to_string(),
            name: "Lily (Female, British)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "bm_daniel".to_string(),
            name: "Daniel (Male, British)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "bm_fable".to_string(),
            name: "Fable (Male, British)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "bm_george".to_string(),
            name: "George (Male, British)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "bm_lewis".to_string(),
            name: "Lewis (Male, British)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "jf_alpha".to_string(),
            name: "Alpha (Female, Japanese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "jf_gongitsune".to_string(),
            name: "Gongitsune (Female, Japanese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "jf_nezumi".to_string(),
            name: "Nezumi (Female, Japanese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "jf_tebukuro".to_string(),
            name: "Tebukuro (Female, Japanese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "jm_kumo".to_string(),
            name: "Kumo (Male, Japanese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "zf_xiaobei".to_string(),
            name: "Xiaobei (Female, Chinese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "zf_xiaoni".to_string(),
            name: "Xiaoni (Female, Chinese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "zf_xiaoxiao".to_string(),
            name: "Xiaoxiao (Female, Chinese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "zf_yunjian".to_string(),
            name: "Yunjian (Female, Chinese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "zf_yunxia".to_string(),
            name: "Yunxia (Female, Chinese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "zf_yunxi".to_string(),
            name: "Yunxi (Female, Chinese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "zm_yunjian".to_string(),
            name: "Yunjian (Male, Chinese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "ef_dora".to_string(),
            name: "Dora (Female, Spanish)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "em_alex".to_string(),
            name: "Alex (Male, Spanish)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "em_santa".to_string(),
            name: "Santa (Male, Spanish)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "ff_siwis".to_string(),
            name: "Siwis (Female, French)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "hf_alpha".to_string(),
            name: "Alpha (Female, Hindi)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "hf_beta".to_string(),
            name: "Beta (Female, Hindi)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "hm_omega".to_string(),
            name: "Omega (Male, Hindi)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "hm_psi".to_string(),
            name: "Psi (Male, Hindi)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "if_sara".to_string(),
            name: "Sara (Female, Italian)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "im_nicola".to_string(),
            name: "Nicola (Male, Italian)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "pf_dora".to_string(),
            name: "Dora (Female, Portuguese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "pm_alex".to_string(),
            name: "Alex (Male, Portuguese)".to_string(),
            preview_url: None,
        },
        Voice {
            id: "pm_santa".to_string(),
            name: "Santa (Male, Portuguese)".to_string(),
            preview_url: None,
        },
    ]
});

/// Validate response format
pub fn validate_response_format(format: &str) -> ApiResult<String> {
    let format_lower = format.to_lowercase();
    if VALID_RESPONSE_FORMATS.contains(&format_lower.as_str()) {
        Ok(format_lower)
    } else {
        Err(AppError::unsupported_format(format))
    }
}

/// Validate input text
pub fn validate_input(input: &str, max_chars: usize) -> ApiResult<()> {
    if input.is_empty() {
        return Err(AppError::invalid_request("Input text cannot be empty"));
    }

    let input_chars = input.chars().count();

    if input_chars > max_chars {
        return Err(AppError::invalid_request(format!(
            "Input text exceeds maximum length of {} characters",
            max_chars
        )));
    }

    Ok(())
}

/// Validate model ID
pub fn validate_model(model: &str) -> ApiResult<String> {
    let accepted = Config::accepted_model_ids();
    if accepted.contains(&model) {
        Ok(model.to_string())
    } else {
        Err(AppError::model_not_found(model))
    }
}

/// Validate voice ID against available voices
pub fn validate_voice(voice: &str, available_voices: &[Voice]) -> ApiResult<String> {
    if available_voices.iter().any(|v| v.id == voice) {
        Ok(voice.to_string())
    } else {
        Err(AppError::voice_not_found(voice))
    }
}

/// Validate speed parameter (0.25 to 4.0)
pub fn validate_speed(speed: f32) -> ApiResult<f32> {
    const MIN_SPEED: f32 = 0.25;
    const MAX_SPEED: f32 = 4.0;

    if speed.is_nan() || speed.is_infinite() {
        return Err(AppError::invalid_request("Speed must be a finite number"));
    }

    if !(MIN_SPEED..=MAX_SPEED).contains(&speed) {
        return Err(AppError::invalid_request(format!(
            "Speed must be between {} and {}, got {}",
            MIN_SPEED, MAX_SPEED, speed
        )));
    }

    Ok(speed)
}

/// Voice information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Voice {
    pub id: String,
    pub name: String,
    pub preview_url: Option<String>,
}

/// Get all available Kokoro voices (returns reference to static)
pub fn get_available_voices() -> &'static [Voice] {
    AVAILABLE_VOICES.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_response_format() {
        assert!(validate_response_format("wav").is_ok());
        assert!(validate_response_format("WAV").is_ok());
        assert!(validate_response_format("pcm").is_ok());
        assert!(validate_response_format("PCM").is_ok());
        assert!(validate_response_format("mp3").is_err());
        assert!(validate_response_format("opus").is_err());
    }

    #[test]
    fn test_validate_input() {
        assert!(validate_input("Hello", 100).is_ok());
        assert!(validate_input("", 100).is_err());
        assert!(validate_input("a".repeat(101).as_str(), 100).is_err());
    }

    #[test]
    fn test_validate_model() {
        assert!(validate_model("tts-1").is_ok());
        assert!(validate_model("kokoro").is_ok());
        assert!(validate_model("invalid").is_err());
    }

    #[test]
    fn test_validate_speed() {
        assert!(validate_speed(0.25).is_ok());
        assert!(validate_speed(1.0).is_ok());
        assert!(validate_speed(4.0).is_ok());
        assert!(validate_speed(0.24).is_err());
        assert!(validate_speed(4.01).is_err());
        assert!(validate_speed(f32::NAN).is_err());
    }
}
