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

/// Needs EasyWSL's Docker engine running. Points nginx:alpine at an older nginx, starts a
/// container from it, then updates it to the real nginx:alpine.
#[tokio::test]
#[ignore]
async fn updates_a_container() {
    use crate::containers;
    let docker = |args: &[&str]| {
        let out = std::process::Command::new(containers::bin_dir().join("docker.exe")).args(args).output().unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    docker(&["rm", "-f", "easywsl-update-test"]);
    docker(&["pull", "nginx:1.26-alpine"]);
    docker(&["tag", "nginx:1.26-alpine", "nginx:alpine"]);
    docker(&[
        "run", "-d", "--name", "easywsl-update-test", "-p", "18089:80", "-e", "GREETING=hello world",
        "-v", "easywsl-update-test:/data", "--restart", "unless-stopped", "--label", "easywsl.app=test", "nginx:alpine",
    ]);
    docker(&["exec", "easywsl-update-test", "sh", "-c", "echo kept > /data/file"]);
    let old = docker(&["inspect", "-f", "{{.Image}}", "easywsl-update-test"]);
    docker(&["pull", "nginx:alpine"]);

    let c = containers::inspect(containers::Engine::Easywsl, "container", "easywsl-update-test").await.unwrap();
    containers::recreate(containers::Engine::Easywsl, &c).await.unwrap();

    let new = docker(&["inspect", "-f", "{{.Image}}", "easywsl-update-test"]);
    let summary = docker(&[
        "inspect", "-f",
        "{{.State.Running}} {{.HostConfig.RestartPolicy.Name}} {{index .Config.Labels \"easywsl.app\"}} {{(index (index .HostConfig.PortBindings \"80/tcp\") 0).HostPort}}",
        "easywsl-update-test",
    ]);
    let env = docker(&["exec", "easywsl-update-test", "printenv", "GREETING"]);
    let data = docker(&["exec", "easywsl-update-test", "cat", "/data/file"]);
    let nginx = docker(&["exec", "easywsl-update-test", "sh", "-c", "printenv NGINX_VERSION"]);
    let leftovers = docker(&["ps", "-a", "--filter", "name=easywsl-update-test-before-update", "-q"]);
    docker(&["rm", "-f", "easywsl-update-test"]);
    docker(&["volume", "rm", "easywsl-update-test"]);
    println!("{old} -> {new}: {summary}, env {env:?}, data {data:?}, nginx {nginx}");
    assert_ne!(old, new);
    assert_eq!(summary, "true unless-stopped test 18089");
    assert_eq!(env, "hello world");
    assert_eq!(data, "kept");
    assert!(!nginx.starts_with("1.26"), "the new image's own NGINX_VERSION applies");
    assert!(leftovers.is_empty());
}
