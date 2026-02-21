use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;
use tracing::info;

const PIPER_PHONEMIZE_TAG: &str = "2023.11.14-4";
const PIPER_URL_ENV: &str = "KOKORO_PIPER_PHONEMIZE_URL";
const ESPEAK_HOME_ENV: &str = "PIPER_ESPEAKNG_DATA_DIRECTORY";

pub async fn ensure_runtime_assets() -> Result<()> {
    ensure_espeak_data_directory().await
}

async fn ensure_espeak_data_directory() -> Result<()> {
    if let Some(existing) = std::env::var_os(ESPEAK_HOME_ENV) {
        let existing = PathBuf::from(existing);
        validate_espeak_home(&existing).with_context(|| {
            format!(
                "{} is set but invalid: {}",
                ESPEAK_HOME_ENV,
                existing.display()
            )
        })?;
        info!(path = %existing.display(), "Using configured eSpeak-ng data directory");
        return Ok(());
    }

    if let Some(existing_home) = discover_existing_espeak_home()? {
        std::env::set_var(ESPEAK_HOME_ENV, &existing_home);
        info!(path = %existing_home.display(), "Discovered local eSpeak-ng data directory");
        return Ok(());
    }

    let runtime_root = dirs::cache_dir()
        .context("Failed to determine cache directory")?
        .join("kokoro-openai-server")
        .join("runtime");
    fs::create_dir_all(&runtime_root).context("Failed to create runtime cache directory")?;

    let install_root = runtime_root.join("piper-phonemize");
    let install_share = install_root.join("share");
    if validate_espeak_home(&install_share).is_ok() {
        std::env::set_var(ESPEAK_HOME_ENV, &install_share);
        info!(path = %install_share.display(), "Using cached eSpeak-ng data directory");
        return Ok(());
    }

    let download_url = match std::env::var(PIPER_URL_ENV) {
        Ok(url) => url,
        Err(_) => default_piper_url()?,
    };
    info!(url = %download_url, "Downloading runtime phonemizer assets");

    let archive_path = runtime_root.join("piper-phonemize.tar.gz");
    download_to_file(&download_url, &archive_path).await?;

    let extract_tmp = runtime_root.join("piper-phonemize.tmp");
    if extract_tmp.exists() {
        fs::remove_dir_all(&extract_tmp).context("Failed to clear temporary extraction dir")?;
    }
    fs::create_dir_all(&extract_tmp).context("Failed to create temporary extraction dir")?;

    extract_tar_gz(&archive_path, &extract_tmp)?;

    let extracted_root = extract_tmp.join("piper-phonemize");
    let extracted_share = extracted_root.join("share");
    validate_espeak_home(&extracted_share).with_context(|| {
        format!(
            "Downloaded archive does not contain expected share/espeak-ng-data at {}",
            extracted_share.display()
        )
    })?;

    if install_root.exists() {
        fs::remove_dir_all(&install_root).context("Failed to replace existing phonemizer cache")?;
    }
    fs::rename(&extracted_root, &install_root).context("Failed to finalize phonemizer assets")?;
    let _ = fs::remove_file(&archive_path);
    let _ = fs::remove_dir_all(&extract_tmp);

    std::env::set_var(ESPEAK_HOME_ENV, &install_share);
    info!(path = %install_share.display(), "Prepared runtime eSpeak-ng data directory");

    Ok(())
}

fn discover_existing_espeak_home() -> Result<Option<PathBuf>> {
    let cwd = std::env::current_dir().context("Failed to read current directory")?;
    if has_espeak_ng_data(&cwd) {
        return Ok(Some(cwd));
    }

    let exe_parent = std::env::current_exe()
        .context("Failed to resolve executable path")?
        .parent()
        .map(Path::to_path_buf);

    if let Some(exe_parent) = exe_parent {
        if has_espeak_ng_data(&exe_parent) {
            return Ok(Some(exe_parent));
        }
    }

    Ok(None)
}

fn has_espeak_ng_data(home: &Path) -> bool {
    home.join("espeak-ng-data").is_dir()
}

fn validate_espeak_home(home: &Path) -> Result<()> {
    if has_espeak_ng_data(home) {
        return Ok(());
    }

    anyhow::bail!(
        "Expected {} directory to contain espeak-ng-data",
        home.display()
    );
}

async fn download_to_file(url: &str, target: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to request phonemizer archive from {url}"))?
        .error_for_status()
        .with_context(|| format!("Phonemizer archive download failed for {url}"))?;

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read phonemizer archive response body from {url}"))?;

    fs::write(target, &bytes).with_context(|| {
        format!(
            "Failed to persist downloaded phonemizer archive to {}",
            target.display()
        )
    })?;
    Ok(())
}

fn extract_tar_gz(archive_path: &Path, target_dir: &Path) -> Result<()> {
    let file = fs::File::open(archive_path).with_context(|| {
        format!(
            "Failed to open downloaded archive at {}",
            archive_path.display()
        )
    })?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(target_dir).with_context(|| {
        format!(
            "Failed to unpack archive {} into {}",
            archive_path.display(),
            target_dir.display()
        )
    })?;
    Ok(())
}

fn default_piper_url() -> Result<String> {
    let archive_name = piper_archive_name_for_target()?;
    Ok(format!(
        "https://github.com/rhasspy/piper-phonemize/releases/download/{PIPER_PHONEMIZE_TAG}/{archive_name}"
    ))
}

fn piper_archive_name_for_target() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("piper-phonemize_macos_aarch64.tar.gz"),
        ("macos", "x86_64") => Ok("piper-phonemize_macos_x64.tar.gz"),
        ("linux", "aarch64") => Ok("piper-phonemize_linux_aarch64.tar.gz"),
        ("linux", "arm") => Ok("piper-phonemize_linux_armv7l.tar.gz"),
        ("linux", "x86_64") => Ok("piper-phonemize_linux_x86_64.tar.gz"),
        (os, arch) => anyhow::bail!(
            "No default piper-phonemize archive for target {os}/{arch}; set {PIPER_URL_ENV}",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_url_contains_release_tag() {
        let url = default_piper_url().unwrap();
        assert!(url.contains(PIPER_PHONEMIZE_TAG));
    }

    #[test]
    fn test_archive_name_for_target_supported_in_tests() {
        let name = piper_archive_name_for_target().unwrap();
        assert!(name.ends_with(".tar.gz"));
        assert!(name.starts_with("piper-phonemize_"));
    }
}
