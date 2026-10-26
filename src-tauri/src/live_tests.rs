//! Tests against the real PC: WSL, the registry, the network. Not run by default;
//! `cargo test -- --ignored` runs them.

use crate::{docker, system, win, wsl};

#[tokio::test]
#[ignore]
async fn this_pc_is_ready() {
    let state = system::check().await;
    println!("{state:?}");
    assert!(matches!(state, system::Readiness::Ready { .. }));
}

#[tokio::test]
#[ignore]
async fn lists_distros_and_running_ones() {
    let list = wsl::registered();
    println!("{list:#?}");
    assert!(!list.is_empty());
    println!("running: {:?}", wsl::running().await);
}

#[test]
#[ignore]
fn signatures() {
    assert!(win::signature_valid(&wsl::exe()), "wsl.exe is signed by Microsoft");
    let unsigned = std::env::temp_dir().join("easywsl-unsigned.exe");
    std::fs::write(&unsigned, b"MZ not really a program").unwrap();
    assert!(!win::signature_valid(&unsigned));
    let _ = std::fs::remove_file(unsigned);
}

#[tokio::test]
#[ignore]
async fn downloads_a_docker_image() {
    let dir = std::env::temp_dir().join("easywsl-docker-test");
    let _ = std::fs::remove_dir_all(&dir);
    let layers = docker::download("library/hello-world", &dir, |done, total| println!("{done}/{total}")).await.unwrap();
    assert!(!layers.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
