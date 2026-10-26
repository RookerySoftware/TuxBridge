//! Downloads the layers of a Docker Hub image (linux/amd64), so a distro without a WSL image can
//! be imported with `wsl --import`. No Docker needed. Every layer is checked against its digest.

use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

const ACCEPT: &str = "application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, \
                      application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json";

#[derive(Deserialize)]
struct Token {
    token: String,
}

#[derive(Deserialize)]
struct Platform {
    architecture: String,
    os: String,
}

#[derive(Deserialize)]
struct IndexEntry {
    digest: String,
    platform: Option<Platform>,
}

#[derive(Deserialize)]
struct Layer {
    digest: String,
    size: u64,
}

#[derive(Deserialize)]
struct Manifest {
    manifests: Option<Vec<IndexEntry>>,
    layers: Option<Vec<Layer>>,
}

/// Downloads the image's layers into `dir` as layer01.tar.gz, layer02.tar.gz, ... and returns
/// their paths. `progress` gets (bytes so far, total bytes).
pub async fn download(image: &str, dir: &Path, mut progress: impl FnMut(u64, u64)) -> anyhow::Result<Vec<PathBuf>> {
    let client = reqwest::Client::builder().user_agent("EasyWSL").build()?;
    let token: Token = client
        .get(format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:{image}:pull"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let base = format!("https://registry-1.docker.io/v2/{image}");
    let get_manifest = |reference: String| {
        let client = client.clone();
        let url = format!("{base}/manifests/{reference}");
        let bearer = token.token.clone();
        async move {
            let m: Manifest = client
                .get(url)
                .bearer_auth(bearer)
                .header("Accept", ACCEPT)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            anyhow::Ok(m)
        }
    };
    let mut manifest = get_manifest("latest".into()).await?;
    if let Some(index) = &manifest.manifests {
        let entry = index
            .iter()
            .find(|e| e.platform.as_ref().is_some_and(|p| p.architecture == "amd64" && p.os == "linux"))
            .ok_or_else(|| anyhow::anyhow!("{image} has no image for 64-bit Intel/AMD PCs"))?;
        manifest = get_manifest(entry.digest.clone()).await?;
    }
    let layers = manifest.layers.ok_or_else(|| anyhow::anyhow!("{image}: no layers in the manifest"))?;
    let total: u64 = layers.iter().map(|l| l.size).sum();
    tokio::fs::create_dir_all(dir).await?;

    let mut done = 0u64;
    let mut files = Vec::new();
    for (i, layer) in layers.iter().enumerate() {
        let path = dir.join(format!("layer{:02}.tar.gz", i + 1));
        let mut file = tokio::fs::File::create(&path).await?;
        let mut hasher = Sha256::new();
        let response = client
            .get(format!("{base}/blobs/{}", layer.digest))
            .bearer_auth(&token.token)
            .send()
            .await?
            .error_for_status()?;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            hasher.update(&chunk);
            file.write_all(&chunk).await?;
            done += chunk.len() as u64;
            progress(done, total);
        }
        file.flush().await?;
        let got = format!("sha256:{}", hex::encode(hasher.finalize()));
        anyhow::ensure!(got == layer.digest, "layer {} of {image} arrived damaged; try again", i + 1);
        files.push(path);
    }
    Ok(files)
}
