use reqwest::Client;
use serde::Deserialize;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let owner = "drager";
    let repo = "wasm-pack";
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let licenses = Licenses {
        keys: vec!["asl20".to_owned(), "mit".to_owned()],
    };

    // Now returns the full ReleaseResponse struct
    let release = fetch_latest_release(&url).await?;

    println!("Latest Version: {}", release.tag_name);
    for asset in &release.assets {
        println!("Name: {}\n  Digest: {}\n", asset.name, asset.digest);
    }

    // Pass the reference to the full release object
    generate_sources_nix(&release, licenses)?;

    println!("Successfully generated sources.nix");

    let licenses = fetch_repository_license(owner, repo).await?;

    println!("{:?}", licenses.license);

    Ok(())
}

pub struct Licenses {
    pub keys: Vec<String>,
}

impl Display for Licenses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Join the keys with spaces to create a Nix-style list
        let list_content = self
            .keys
            .iter()
            .map(|k| format!("\"{k}\""))
            .collect::<Vec<String>>()
            .join(" ");

        // This output format is ready to be pasted directly into a meta block
        write!(f, "licenseKeys = [ {} ];", list_content)
    }
}

async fn fetch_latest_release(url: &str) -> anyhow::Result<ReleaseResponse> {
    let client = Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "rust-release-notifier")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    let release: ReleaseResponse = response.json().await?;
    Ok(release)
}

#[derive(Deserialize, Debug)]
pub struct RepositoryLicense {
    pub name: String,
    pub path: String,
    pub content: String, // Base64 encoded
    pub encoding: String,
    pub license: LicenseInfo,
}

#[derive(Deserialize, Debug)]
pub struct LicenseInfo {
    pub key: String,
    pub name: String,
    pub spdx_id: String,
    pub url: Option<String>,
}

async fn fetch_repository_license(owner: &str, repo: &str) -> anyhow::Result<RepositoryLicense> {
    // Note: Use 'license' singular
    let url = format!("https://api.github.com/repos/{}/{}/license", owner, repo);

    let client = Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "rust-release-notifier")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch license: {}",
            response.status()
        ));
    }

    let license_data: RepositoryLicense = response.json().await?;
    Ok(license_data)
}

#[derive(Deserialize, Debug)]
pub struct ReleaseResponse {
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize, Debug)]
pub struct ReleaseAsset {
    pub name: String,
    pub digest: String,
    pub browser_download_url: String,
}

fn generate_sources_nix(release: &ReleaseResponse, licenses: Licenses) -> anyhow::Result<()> {
    let mut file = File::create("sources.nix")?;

    writeln!(file, "{{")?;
    writeln!(file, "  version = {:?};", release.tag_name)?;
    writeln!(file, "  assets = {{")?;

    for asset in &release.assets {
        let platform = match asset.name.as_str() {
            n if n.contains("x86_64-unknown-linux-musl") => Some("x86_64-linux"),
            n if n.contains("aarch64-unknown-linux-musl") => Some("aarch64-linux"),
            n if n.contains("x86_64-apple-darwin") => Some("x86_64-darwin"),
            n if n.contains("aarch64-apple-darwin") => Some("aarch64-darwin"),
            _ => None,
        };

        if let Some(p) = platform {
            writeln!(file, "    {:?} = {{", p)?;
            writeln!(file, "      url = {:?};", asset.browser_download_url)?;
            writeln!(file, "      hash = {:?};", asset.digest)?;
            writeln!(file, "    }};")?;
        }
    }

    writeln!(file, "  }};")?;

    writeln!(file, "  {}", licenses)?;

    writeln!(file, "}}")?;
    Ok(())
}
