use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub async fn download_and_install(tag: &str) -> Result<()> {
    let download_url = crate::updater::github::get_download_url(tag);
    log::info!("Downloading rust-analyzer {} from {}", tag, download_url);

    let temp_dir = tempfile::tempdir()?;
    let archive_path = temp_dir.path().join(format!("rust-analyzer-{}", tag));

    let response = reqwest::get(&download_url)
        .await
        .context("Failed to download rust-analyzer")?;

    let bytes = response.bytes().await?;

    fs::write(&archive_path, &bytes)?;

    let binary_path = install_binary(&archive_path, tag).await?;

    log::info!("Installed rust-analyzer to: {}", binary_path.display());

    Ok(())
}

async fn install_binary(archive_path: &Path, tag: &str) -> Result<PathBuf> {
    let install_dir = get_install_dir()?;
    let binary_name = if cfg!(target_os = "windows") {
        "rust-analyzer.exe"
    } else {
        "rust-analyzer"
    };

    let dest_path = install_dir.join(binary_name);

    fs::create_dir_all(&install_dir)?;

    let mut file = File::open(archive_path)?;

    if archive_path.extension().and_then(|s| s.to_str()) == Some("zip") {
        extract_zip(&mut file, &dest_path)?;
    } else {
        extract_tar_gz(&mut file, &dest_path)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_path, perms)?;
    }

    Ok(dest_path)
}

fn extract_tar_gz<R: Read>(reader: &mut R, dest: &Path) -> Result<()> {
    let mut decoder = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(&mut decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();

        if path.file_name().and_then(|s| s.to_str()) == Some("rust-analyzer") {
            let mut outfile = File::create(dest)?;
            io::copy(&mut entry, &mut outfile)?;
            return Ok(());
        }
    }

    anyhow::bail!("rust-analyzer binary not found in archive")
}

fn extract_zip<R: io::Read + io::Seek>(reader: &mut R, dest: &Path) -> Result<()> {
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();

        if name.ends_with("rust-analyzer.exe") || name.ends_with("rust-analyzer") {
            let mut outfile = File::create(dest)?;
            io::copy(&mut file, &mut outfile)?;
            return Ok(());
        }
    }

    anyhow::bail!("rust-analyzer binary not found in archive")
}

fn get_install_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Program Files"));
        Ok(base.join("rust-analyzer-mcp").join("bin"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let base = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/usr/local"));
        Ok(base.join(".rust-analyzer-mcp").join("bin"))
    }
}
