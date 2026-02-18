use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionProvider {
    Auto,
    Cpu,
    CoreML,
    Cuda,
    DirectML,
}

impl FromStr for ExecutionProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "cpu" => Ok(Self::Cpu),
            "coreml" | "core_ml" => Ok(Self::CoreML),
            "cuda" => Ok(Self::Cuda),
            "directml" | "direct_ml" => Ok(Self::DirectML),
            _ => Err(format!("Unknown execution provider: {}", s)),
        }
    }
}

impl std::fmt::Display for ExecutionProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Cpu => write!(f, "cpu"),
            Self::CoreML => write!(f, "coreml"),
            Self::Cuda => write!(f, "cuda"),
            Self::DirectML => write!(f, "directml"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub api_key: Option<String>,
    pub model_path: Option<PathBuf>,
    pub execution_provider: ExecutionProvider,
    pub workers: usize,
    pub max_input_chars: usize,
}

impl Config {
    pub fn from_env_and_args() -> Result<Self> {
        let cli = CliArgs::parse();

        let config = Self {
            host: cli.host,
            port: cli.port,
            api_key: cli.api_key,
            model_path: cli.model_path,
            execution_provider: cli.execution_provider,
            workers: cli.workers,
            max_input_chars: cli.max_input_chars,
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Validate workers range
        if self.workers == 0 || self.workers > 8 {
            anyhow::bail!("Workers must be between 1 and 8, got {}", self.workers);
        }

        // Validate port
        if self.port == 0 {
            anyhow::bail!("Port cannot be 0");
        }

        // Validate max_input_chars
        if self.max_input_chars == 0 {
            anyhow::bail!("Max input chars cannot be 0");
        }

        // Validate execution provider based on platform
        #[cfg(not(target_os = "macos"))]
        if self.execution_provider == ExecutionProvider::CoreML {
            anyhow::bail!("CoreML is only available on macOS");
        }

        #[cfg(not(target_os = "windows"))]
        if self.execution_provider == ExecutionProvider::DirectML {
            anyhow::bail!("DirectML is only available on Windows");
        }

        Ok(())
    }

    pub fn accepted_model_ids() -> Vec<&'static str> {
        vec!["tts-1", "kokoro"]
    }
}

#[derive(Parser, Debug)]
#[command(name = "kokoro-openai-server")]
#[command(about = "OpenAI-compatible TTS server for Kokoro model")]
#[command(version)]
struct CliArgs {
    /// Host address to bind to
    #[arg(long, env = "HOST", default_value = "0.0.0.0")]
    host: String,

    /// Port to listen on
    #[arg(long, env = "PORT", default_value = "8000")]
    port: u16,

    /// API key for authentication (optional)
    #[arg(long, env = "API_KEY")]
    api_key: Option<String>,

    /// Path to model files (optional, will download if not provided)
    #[arg(long, env = "MODEL_PATH")]
    model_path: Option<PathBuf>,

    /// Execution provider for inference
    #[arg(long, env = "EXECUTION_PROVIDER", default_value = "auto")]
    execution_provider: ExecutionProvider,

    /// Number of worker threads for parallel inference
    #[arg(long, env = "WORKERS", default_value = "2")]
    workers: usize,

    /// Maximum characters allowed in input text
    #[arg(long, env = "MAX_INPUT_CHARS", default_value = "4096")]
    max_input_chars: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_provider_parsing() {
        assert_eq!(
            ExecutionProvider::from_str("auto").unwrap(),
            ExecutionProvider::Auto
        );
        assert_eq!(
            ExecutionProvider::from_str("cpu").unwrap(),
            ExecutionProvider::Cpu
        );
        assert_eq!(
            ExecutionProvider::from_str("coreml").unwrap(),
            ExecutionProvider::CoreML
        );
        assert_eq!(
            ExecutionProvider::from_str("CORE_ML").unwrap(),
            ExecutionProvider::CoreML
        );
        assert!(ExecutionProvider::from_str("invalid").is_err());
    }

    #[test]
    fn test_config_validation() {
        let valid_config = Config {
            host: "0.0.0.0".to_string(),
            port: 8000,
            api_key: None,
            model_path: None,
            execution_provider: ExecutionProvider::Cpu,
            workers: 2,
            max_input_chars: 4096,
        };
        assert!(valid_config.validate().is_ok());

        let invalid_workers = Config {
            workers: 0,
            ..valid_config.clone()
        };
        assert!(invalid_workers.validate().is_err());

        let invalid_workers_high = Config {
            workers: 9,
            ..valid_config.clone()
        };
        assert!(invalid_workers_high.validate().is_err());
    }

    #[test]
    fn test_accepted_model_ids() {
        let ids = Config::accepted_model_ids();
        assert!(ids.contains(&"tts-1"));
        assert!(ids.contains(&"kokoro"));
    }
}
