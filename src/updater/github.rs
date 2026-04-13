use anyhow::Result;
use reqwest::Client;

const GITHUB_API: &str = "https://api.github.com/repos/rust-lang/rust-analyzer";

pub async fn get_latest_release_tag() -> Result<String> {
    let client = Client::builder()
        .user_agent("rust-analyzer-mcp/0.1.0")
        .build()?;

    let resp: serde_json::Value = client
        .get(&format!("{}/releases/latest", GITHUB_API))
        .send()
        .await?
        .json()
        .await?;

    let tag = resp["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No tag_name in response"))?
        .to_string();

    Ok(tag)
}

pub async fn get_release_assets(tag: &str) -> Result<Vec<Asset>> {
    let client = Client::builder()
        .user_agent("rust-analyzer-mcp/0.1.0")
        .build()?;

    let resp: serde_json::Value = client
        .get(&format!("{}/releases/tags/{}", GITHUB_API, tag))
        .send()
        .await?
        .json()
        .await?;

    let assets: Vec<Asset> = serde_json::from_value(resp["assets"].clone())?;
    Ok(assets)
}

#[derive(Debug, serde::Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn get_download_url(tag: &str) -> String {
    let (os, arch, ext) = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => ("linux", "x86_64", "gz"),
        ("linux", "aarch64") => ("linux", "aarch64", "gz"),
        ("macos", "x86_64") => ("macos", "x86_64", "gz"),
        ("macos", "aarch64") => ("macos", "aarch64", "gz"),
        ("windows", "x86_64") => ("windows", "x86_64", "zip"),
        _ => ("linux", "x86_64", "gz"),
    };

    format!(
        "https://github.com/rust-lang/rust-analyzer/releases/download/{}/rust-analyzer-{}-{}-{}.{}",
        tag, os, arch, os, ext
    )
}
