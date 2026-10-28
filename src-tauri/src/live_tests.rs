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

/// Downloads a distro's image to try it by hand: EASYWSL_IMAGE=parrotsec/core, into
/// EASYWSL_IMAGE_DIR (kept), then `wsl --import` the layer (or merge-layers.sh several).
#[tokio::test]
#[ignore]
async fn downloads_a_distro_image() {
    let image = std::env::var("EASYWSL_IMAGE").expect("EASYWSL_IMAGE");
    let dir = std::path::PathBuf::from(std::env::var("EASYWSL_IMAGE_DIR").expect("EASYWSL_IMAGE_DIR"));
    let layers = docker::download(&image, &dir, |done, total| println!("{done}/{total}")).await.unwrap();
    println!("{layers:?}");
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

/// Installs the assistant into P:\WSL\easywsl-ai-test (Ollama 1.6 GB and the standard model),
/// asks it something, and leaves it there for the next tests. EASYWSL_AI_MODEL picks the size.
#[tokio::test]
#[ignore]
async fn installs_the_assistant_and_asks_it() {
    let dir = std::path::PathBuf::from(r"P:\WSL\easywsl-ai-test");
    let id = std::env::var("EASYWSL_AI_MODEL").unwrap_or("standard".into());
    let model = crate::ai::MODELS.iter().find(|m| m.id == id).unwrap();
    let job = crate::jobs::Job::detached("assistant");
    let started = std::time::Instant::now();
    let result = crate::ai::install(job.clone(), dir.clone(), model, None).await;
    println!("install: {result:?} in {:?}\n{}", started.elapsed(), job.log_tail());
    result.unwrap();

    let ask = |q: &str| vec![
        crate::ai::Message { role: "system".into(), content: "You help people use Linux on Windows (WSL). Answer briefly.".into() },
        crate::ai::Message { role: "user".into(), content: q.into() },
    ];
    for q in ["How do I copy a file from Windows to my Ubuntu home folder in WSL?", "What does `sudo apt update` do?"] {
        let t = std::time::Instant::now();
        let a = crate::ai::answer_for_test(&dir, model.tag, &ask(q)).await.unwrap();
        println!("\n--- {q} ({:?})\n{a}", t.elapsed());
        assert!(!a.trim().is_empty());
    }
    crate::ai::stop();
}

/// Asks the installed test assistant (see installs_the_assistant_and_asks_it) with a fresh
/// server; prints how long loading and answering take.
#[tokio::test]
#[ignore]
async fn assistant_answers_quickly() {
    let dir = std::path::PathBuf::from(r"P:\WSL\easywsl-ai-test");
    let id = std::env::var("EASYWSL_AI_MODEL").unwrap_or("standard".into());
    let model = crate::ai::MODELS.iter().find(|m| m.id == id).unwrap();
    crate::ai::stop();
    let t = std::time::Instant::now();
    crate::ai::start(&dir).await.unwrap();
    println!("server: {:?}", t.elapsed());
    let t = std::time::Instant::now();
    crate::ai::warm_up(&dir, model.tag).await;
    println!("warm-up: {:?}", t.elapsed());
    let q = vec![crate::ai::Message { role: "user".into(), content: "Name three Linux distributions.".into() }];
    let t = std::time::Instant::now();
    let a = crate::ai::answer_for_test(&dir, model.tag, &q).await.unwrap();
    println!("answer: {:?}\n{a}", t.elapsed());
    crate::ai::stop();
}
