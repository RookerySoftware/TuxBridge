mod ai;
mod apps;
mod catalog;
mod containers;
mod desktop;
mod docker;
mod engine;
mod feedback;
mod install;
mod jobs;
mod paths;
mod skills;
mod store;
mod system;
mod updates;
mod win;
mod wsl;

#[cfg(test)]
mod live_tests;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use jobs::{Job, Jobs};
use store::Store;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Distro {
    name: String,
    running: bool,
    is_default: bool,
    wsl_version: u32,
    base_path: String,
    info: store::DistroInfo,
    /// The catalogue entry it came from, for its title, colour and Hyprland support.
    catalog: Option<catalog::Entry>,
    /// A job (install, adding Hyprland) is changing it.
    busy: bool,
}

#[tauri::command]
async fn system_check() -> system::Readiness {
    system::check().await
}

#[tauri::command]
async fn system_fix(state: String) -> Result<system::Readiness, String> {
    system::fix(&state).await
}

/// Asks a running system TuxBridge did not make what it is: its kind (for the right title and
/// colour), its user, whether it has hyprland-wsl's desktop. Only running ones are asked, since
/// asking would start a stopped one.
async fn probe(app: AppHandle, name: String) {
    let script = r#"id -un; . /etc/os-release 2>/dev/null; echo "ID=$ID"
        [ -x /usr/local/bin/hyprland-wsl ] && echo HYPRLAND
        sed -n 's/^HYPR_USER=/HYPR_USER=/p' /etc/hyprland-wsl.conf 2>/dev/null
        [ -e /etc/hyprland-wsl/first-start ] && echo FIRST_START
        [ -f /etc/easywsl/desktop ] && echo "DESKTOP=$(cat /etc/easywsl/desktop)"
        [ -f /etc/easywsl/desktop-user ] && echo "DESKTOP_USER=$(cat /etc/easywsl/desktop-user)"
        awk -F: '$3 >= 1000 && $3 < 60000 && $6 ~ /^\/home\// { print "FIRST_USER=" $1; exit }' /etc/passwd; true"#;
    let Ok(out) = wsl::output(["-d", &name, "-e", "sh", "-c", script]).await else { return };
    let _ = std::fs::create_dir_all(paths::logs_dir());
    let _ = std::fs::write(paths::logs_dir().join(format!("probe-{name}.log")), format!("exit {}\n{}", out.code, out.text));
    if out.code != 0 {
        return;
    }
    let lines: Vec<&str> = out.text.lines().map(str::trim).collect();
    let default_user = lines.first().map(|s| s.to_string()).filter(|u| !u.is_empty() && u != "root");
    let id = lines.iter().find_map(|l| l.strip_prefix("ID=")).unwrap_or("");
    let catalog_id = match id {
        "arch" | "archarm" => Some("arch"),
        "ubuntu" => Some("ubuntu"),
        "debian" => Some("debian"),
        "kali" => Some("kali"),
        "parrot" => Some("parrot"),
        "linuxmint" => Some("mint"),
        "cachyos" => Some("cachyos"),
        "omarchy" => Some("omarchy"),
        _ => None,
    };
    let hyprland = lines.contains(&"HYPRLAND");
    let hypr_user = lines.iter().find_map(|l| l.strip_prefix("HYPR_USER=")).map(String::from).filter(|u| !u.is_empty());
    let first_start = lines.contains(&"FIRST_START");
    let desktop_user = lines.iter().find_map(|l| l.strip_prefix("DESKTOP_USER=")).map(String::from).filter(|u| !u.is_empty());
    // For a system that logs in as root and has no record of whose desktop it is.
    let first_user = lines.iter().find_map(|l| l.strip_prefix("FIRST_USER=")).map(String::from).filter(|u| !u.is_empty());
    let own_desktop = lines.iter().find_map(|l| l.strip_prefix("DESKTOP=")).map(String::from).filter(|d| !d.is_empty());
    app.state::<Store>().update_distro(&name, |d| {
        d.probed = true;
        d.catalog_id = catalog_id.map(String::from);
        d.hyprland = hyprland;
        d.user = hypr_user.clone().or(desktop_user.clone()).or(default_user.clone()).or(first_user.clone());
        d.first_start_pending = first_start;
        d.desktop = own_desktop.clone();
    });
    let _ = app.emit("distros-changed", ());
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Listing {
    distros: Vec<Distro>,
    /// False when WSL did not answer: the window offers to restart it.
    responding: bool,
}

#[tauri::command]
async fn list_distros(app: AppHandle, store: State<'_, Store>, jobs: State<'_, Jobs>) -> Result<Listing, String> {
    let answer = wsl::running().await;
    let responding = answer.is_some();
    let running = answer.unwrap_or_default();
    static PROBING: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    for name in &running {
        if !store.distro(name).known() && !jobs.busy(name) && wsl::exists(name) {
            let mut probing = PROBING.lock().unwrap();
            if !probing.contains(name) {
                probing.push(name.clone());
                let (app, name) = (app.clone(), name.clone());
                tauri::async_runtime::spawn(async move {
                    probe(app, name.clone()).await;
                    PROBING.lock().unwrap().retain(|n| n != &name);
                });
            }
        }
    }
    let distros = wsl::registered()
        .into_iter()
        // The throwaway distro used while unpacking a download is not one of the user's.
        .filter(|d| !d.name.ends_with("-merge"))
        // Docker's engine runs in a system of its own, shown on the Docker page instead.
        .filter(|d| d.name != containers::DISTRO)
        .map(|d| {
            let info = store.distro(&d.name);
            let catalog = info.catalog_id.as_deref().and_then(catalog::get);
            Distro {
                running: running.contains(&d.name),
                is_default: d.is_default,
                wsl_version: d.version,
                base_path: d.base_path,
                busy: jobs.busy(&d.name),
                info,
                catalog,
                name: d.name,
            }
        })
        .collect();
    Ok(Listing { distros, responding })
}

/// Restarts WSL when it stopped answering. Every running system stops; the window asks first.
#[tauri::command]
async fn restart_wsl(app: AppHandle) -> Result<(), String> {
    let result = wsl::shutdown().await.map_err(|e| {
        if e.is::<wsl::NotResponding>() {
            "WSL could not be restarted either. Restart Windows; that always clears it.".to_string()
        } else {
            format!("Restarting WSL did not work: {e}")
        }
    });
    let _ = app.emit("distros-changed", ());
    result
}

#[tauri::command]
fn get_catalog() -> Vec<catalog::Entry> {
    catalog::all()
}

/// A name that is not taken yet: "ubuntu", "ubuntu2", ...
#[tauri::command]
fn suggest_name(catalog_id: String) -> String {
    let base = catalog_id.to_lowercase();
    (1..)
        .map(|i| if i == 1 { base.clone() } else { format!("{base}{i}") })
        .find(|n| !wsl::exists(n))
        .unwrap()
}

/// The Windows user name, as a Linux user name, to prefill the form.
#[tauri::command]
fn suggest_user() -> String {
    let name: String = std::env::var("USERNAME")
        .unwrap_or_default()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_' || *c == '-')
        .collect();
    if install::valid_user(&name) { name } else { "me".into() }
}

#[tauri::command]
fn default_location(store: State<'_, Store>, name: String) -> String {
    store.install_dir().join(name).to_string_lossy().into_owned()
}

#[tauri::command]
fn check_install(app: AppHandle, request: install::Request) -> Result<String, String> {
    install::check(&app, &request).map(|(_, location)| location.to_string_lossy().into_owned())
}

#[tauri::command]
fn start_install(app: AppHandle, request: install::Request) -> Result<u64, String> {
    let (entry, location) = install::check(&app, &request)?;
    let job = Job::new(&app, "install", &request.name, &format!("Installing {}", entry.title)).map_err(|e| e.to_string())?;
    let id = job.id();
    tauri::async_runtime::spawn(async move {
        let result = install::run(app.clone(), job.clone(), request, entry, location).await;
        job.finish(&result);
        let _ = app.emit("distros-changed", ());
    });
    Ok(id)
}

/// Adds a desktop to an installed system: Hyprland, or the system's own (Cinnamon, KDE Plasma, Xfce).
#[tauri::command]
fn add_desktop(app: AppHandle, name: String, kind: desktop::Kind, main_key: String, allow_third_party: bool) -> Result<u64, String> {
    let store = app.state::<Store>();
    if app.state::<Jobs>().busy(&name) {
        return Err(format!("{name} is busy with another change."));
    }
    let info = store.distro(&name);
    let own = info.catalog_id.as_deref().and_then(catalog::get).and_then(|e| e.desktop);
    let (title, estimate) = match (kind, &own) {
        (desktop::Kind::Hyprland, _) => ("Installing the Hyprland desktop".to_string(), "2 to 5 minutes".to_string()),
        (desktop::Kind::Own, Some(d)) => (format!("Installing the {} desktop", d.name), format!("{} minutes", d.minutes)),
        (desktop::Kind::Own, None) => return Err(format!("{name} has no desktop of its own that TuxBridge can install.")),
    };
    let job = Job::new(&app, "desktop", &name, &format!("Adding a desktop to {name}")).map_err(|e| e.to_string())?;
    let id = job.id();
    tauri::async_runtime::spawn(async move {
        let result: Result<(), String> = async {
            job.plan(&[("Finding your account", None), (&title, Some(&estimate))]);
            job.step("Finding your account");
            let user = match info.user {
                Some(u) => u,
                // A distro TuxBridge did not make: its default user.
                None => wsl::output(["-d", &name, "-e", "id", "-un"])
                    .await
                    .map(|o| o.text.trim().to_string())
                    .ok()
                    .filter(|u| !u.is_empty() && u != "root")
                    .ok_or("This system has no user account besides root. Create one first.")?,
            };
            job.step(&title);
            match (kind, own) {
                (desktop::Kind::Hyprland, _) => {
                    let engine = engine::prepare(&app).map_err(|e| e.to_string())?;
                    install::install_hyprland(&job, &engine, &name, &user, allow_third_party, Some(&main_key)).await?;
                    app.state::<Store>().update_distro(&name, |d| {
                        d.hyprland = true;
                        d.user = Some(user.clone());
                        d.main_key = Some(main_key.clone());
                    });
                }
                (desktop::Kind::Own, Some(d)) => {
                    let linux = engine::prepare_linux(&app).map_err(|e| e.to_string())?;
                    install::install_own_desktop(&job, &linux, &name, &user, &d).await?;
                    app.state::<Store>().update_distro(&name, |info| {
                        info.desktop = Some(d.id.to_string());
                        info.user = Some(user.clone());
                    });
                }
                _ => {}
            }
            Ok(())
        }
        .await;
        job.finish(&result);
        let _ = app.emit("distros-changed", ());
    });
    Ok(id)
}

#[tauri::command]
fn list_jobs(jobs: State<'_, Jobs>) -> Vec<jobs::JobView> {
    jobs.all()
}

/// The last lines of a job's log (for asking the assistant about a failure).
#[tauri::command]
fn job_log_tail(jobs: State<'_, Jobs>, id: u64) -> String {
    let Some(job) = jobs.all().into_iter().find(|j| j.id == id) else { return String::new() };
    let text = std::fs::read_to_string(&job.log).unwrap_or_default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(25)..].join("\n")
}

#[tauri::command]
fn dismiss_job(jobs: State<'_, Jobs>, id: u64) {
    jobs.dismiss(id);
}

/// A terminal window in a distro, in the home folder; with `script`, running that bash script
/// first and then staying open.
fn terminal(name: &str, script: Option<&str>) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let mut args: Vec<String> = ["-d", name, "--cd", "~"].map(String::from).to_vec();
    if let Some(script) = script {
        args.extend(["-e", "bash", "-lc", &format!("{script}; exec bash -l")].map(String::from));
    }
    let wt = dirs::data_local_dir().map(|d| d.join(r"Microsoft\WindowsApps\wt.exe")).filter(|p| p.exists());
    let result = match wt {
        Some(wt) => std::process::Command::new(wt).args(["-w", "new", "wsl.exe"]).args(&args).spawn(),
        None => std::process::Command::new(wsl::exe()).args(&args).creation_flags(wsl::CREATE_NEW_CONSOLE).spawn(),
    };
    result.map(|_| ()).map_err(|e| format!("The terminal did not open: {e}"))
}

#[tauri::command]
fn open_terminal(name: String) -> Result<(), String> {
    terminal(&name, None)
}

/// Updates a system's programs. Returns the job, or nothing when the system has its own update
/// tool that needs the person (Omarchy's), which then runs in a terminal window.
#[tauri::command]
async fn update_system(app: AppHandle, name: String) -> Result<Option<u64>, String> {
    if app.state::<Jobs>().busy(&name) {
        return Err(format!("{name} is busy with another change."));
    }
    let linux = engine::prepare_linux(&app).map_err(|e| e.to_string())?;
    let script = format!("{linux}/system-update.sh");
    let check = wsl::output(["-d", &name, "-u", "root", "-e", "sh", &script, "check"]).await.map_err(|e| e.to_string())?;
    match check.text.trim() {
        "omarchy" => {
            terminal(&name, Some(r#"export PATH="$HOME/.local/share/omarchy/bin:$PATH"; omarchy-update"#))?;
            return Ok(None);
        }
        "none" => return Err(format!("TuxBridge does not know how to update {name}: it has none of the usual package managers.")),
        _ if check.code != 0 => return Err(format!("{name} did not answer. Try again, or restart WSL.")),
        _ => {}
    }
    let job = Job::new(&app, "update", &name, &format!("Updating {name}")).map_err(|e| e.to_string())?;
    let id = job.id();
    let _ = app.emit("distros-changed", ());
    tauri::async_runtime::spawn(async move {
        job.plan(&[("Downloading and installing updates", Some("a few minutes"))]);
        job.step("Downloading and installing updates");
        let result = match job.run(wsl::command(["-d", &name, "-u", "root", "-e", "sh", &script, "run"]), None).await {
            Ok(()) => {
                let tail = job.log_tail();
                let count = tail.lines().rev().find_map(|l| l.strip_prefix("easywsl-updated: ")).and_then(|n| n.trim().parse::<u32>().ok());
                match count {
                    Some(0) => job.step("Everything was up to date already"),
                    Some(1) => job.step("1 package updated"),
                    Some(n) => job.step(&format!("{n} packages updated")),
                    None => {}
                }
                Ok(())
            }
            Err(_) => {
                let tail = job.log_tail();
                Err(if tail.contains("Could not get lock") || tail.contains("unable to lock database") {
                    format!("Another program is installing or updating in {name} right now. Wait for it to finish, then try again.")
                } else if tail.contains("No space left") || tail.contains("not enough free disk space") {
                    "The drive is full. Free a few GB and try again.".to_string()
                } else if tail.contains("Temporary failure resolving") || tail.contains("Could not resolve") || tail.contains("failed retrieving file") {
                    "The updates could not be downloaded. Check the internet connection and try again.".to_string()
                } else {
                    "Updating did not finish. The log shows where it stopped; trying again often works.".to_string()
                })
            }
        };
        job.finish(&result);
        let _ = app.emit("distros-changed", ());
    });
    Ok(Some(id))
}

/// Sets up a Learn quest inside a system: runs the bundled script (which builds a folder of
/// practice files as the person's own user), then opens a terminal there. The flags the script
/// hides are checked in the app (src/learn/quests.ts).
#[tauri::command]
async fn start_quest(app: AppHandle, name: String, quest: String) -> Result<(), String> {
    // (script file, folder, first file to show) for each known quest.
    let (script, folder, first) = match quest.as_str() {
        "kids" => ("quest-kids.sh", "treasure-hunt", "START-HERE.txt"),
        "linux" => ("quest-linux.sh", "linux-quest", "README.txt"),
        "kali" => ("quest-kali.sh", "security-quest", "BRIEFING.txt"),
        _ => return Err("Unknown quest.".into()),
    };
    if !wsl::exists(&name) {
        return Err(format!("{name} is not installed."));
    }
    let linux = engine::prepare_linux(&app).map_err(|e| e.to_string())?;
    let out = wsl::output(["-d", &name, "-e", "bash", &format!("{linux}/{script}")])
        .await
        .map_err(|e| e.to_string())?;
    if out.code != 0 {
        return Err(format!("The quest could not be set up. {}", out.text.lines().last().unwrap_or("")));
    }
    // Open a terminal in the quest folder, showing the first file to read.
    terminal(&name, Some(&format!("cd ~/{folder} 2> /dev/null; clear; cat {first} 2> /dev/null; echo")))
}

#[tauri::command]
fn open_files(store: State<'_, Store>, name: String) -> Result<(), String> {
    let base = format!(r"\\wsl.localhost\{name}");
    let folder = match store.distro(&name).user {
        Some(user) => format!(r"{base}\home\{user}"),
        None => base,
    };
    std::process::Command::new("explorer.exe").arg(folder).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
fn reveal(path: String) -> Result<(), String> {
    std::process::Command::new("explorer.exe").arg(format!("/select,{path}")).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_distro(app: AppHandle, name: String) -> Result<(), String> {
    wsl::terminate(&name).await;
    let _ = app.emit("distros-changed", ());
    Ok(())
}

#[tauri::command]
async fn set_default(app: AppHandle, name: String) -> Result<(), String> {
    let out = wsl::output(["--set-default", &name]).await.map_err(|e| e.to_string())?;
    let _ = app.emit("distros-changed", ());
    if out.code != 0 { Err(out.text.trim().to_string()) } else { Ok(()) }
}

#[tauri::command]
async fn remove_distro(app: AppHandle, name: String) -> Result<(), String> {
    if app.state::<Jobs>().busy(&name) {
        return Err(format!("{name} is busy with another change."));
    }
    let base = wsl::registered().into_iter().find(|d| d.name == name).map(|d| d.base_path);
    let out = wsl::output_within(["--unregister", &name], std::time::Duration::from_secs(120)).await.map_err(|e| e.to_string())?;
    if out.code != 0 {
        return Err(format!("Removing {name} did not work: {}", out.text.trim()));
    }
    // WSL deletes the disk; its folder is removed when nothing else is in it.
    if let Some(base) = base {
        let _ = std::fs::remove_dir(base);
    }
    app.state::<Store>().forget_distro(&name);
    let _ = app.emit("distros-changed", ());
    Ok(())
}

#[tauri::command]
fn open_desktop(app: AppHandle, name: String, kind: desktop::Kind) {
    tauri::async_runtime::spawn(async move {
        let _ = desktop::open(app.clone(), name, kind).await;
        let _ = app.emit("distros-changed", ());
    });
}

#[tauri::command]
fn list_sessions(sessions: State<'_, desktop::Sessions>) -> Vec<desktop::Session> {
    sessions.all()
}

#[tauri::command]
fn get_settings(store: State<'_, Store>) -> store::Settings {
    store.settings()
}

#[tauri::command]
fn set_settings(store: State<'_, Store>, settings: store::Settings) {
    store.set_settings(settings);
}

#[tauri::command]
fn default_install_dir() -> String {
    paths::default_distros_dir().to_string_lossy().into_owned()
}

// ---------------------------------------------------------------------------------------------
// Docker (containers.rs, apps.rs)

#[tauri::command]
async fn docker_status() -> containers::Status {
    containers::status().await
}

#[tauri::command]
fn docker_enable(app: AppHandle, store: State<'_, Store>) -> Result<u64, String> {
    if app.state::<Jobs>().busy(containers::DISTRO) {
        return Err("Docker is being set up already.".into());
    }
    let location = store.install_dir().join(containers::DISTRO);
    let linux = engine::prepare_linux(&app).map_err(|e| e.to_string())?;
    let job = Job::new(&app, "docker", containers::DISTRO, "Turning on Docker").map_err(|e| e.to_string())?;
    let id = job.id();
    tauri::async_runtime::spawn(async move {
        let result = containers::enable(job.clone(), location, linux).await;
        job.finish(&result);
        let _ = app.emit("docker-changed", ());
    });
    Ok(id)
}

#[tauri::command]
async fn docker_start(app: AppHandle) -> Result<(), String> {
    containers::start()?;
    let ready = containers::wait_ready(std::time::Duration::from_secs(60)).await;
    let _ = app.emit("docker-changed", ());
    if ready { Ok(()) } else { Err("Docker did not start. Try again, or restart WSL from My systems.".into()) }
}

#[tauri::command]
async fn docker_stop(app: AppHandle) -> Result<(), String> {
    let result = containers::stop().await;
    let _ = app.emit("docker-changed", ());
    result
}

#[tauri::command]
async fn docker_remove(app: AppHandle) -> Result<(), String> {
    let result = containers::remove().await;
    let _ = app.emit("docker-changed", ());
    result
}

#[tauri::command]
async fn docker_containers() -> Vec<containers::Container> {
    containers::containers().await
}

#[tauri::command]
async fn docker_container_action(app: AppHandle, engine: containers::Engine, id: String, action: String) -> Result<(), String> {
    let result = containers::act(engine, &id, &action).await;
    let _ = app.emit("docker-changed", ());
    result
}

/// Updates a container to the newest version of its image.
#[tauri::command]
fn docker_update(app: AppHandle, engine: containers::Engine, id: String, name: String) -> Result<u64, String> {
    let key = format!("container {name}");
    if app.state::<Jobs>().busy(&key) {
        return Err(format!("{name} is being updated already."));
    }
    let job = Job::new(&app, "container-update", &key, &format!("Updating {name}")).map_err(|e| e.to_string())?;
    let jid = job.id();
    tauri::async_runtime::spawn(async move {
        let result = containers::update(job.clone(), engine, &id).await;
        job.finish(&result);
        let _ = app.emit("docker-changed", ());
    });
    Ok(jid)
}

#[tauri::command]
async fn docker_logs(engine: containers::Engine, id: String) -> Result<String, String> {
    containers::logs(engine, &id).await
}

#[tauri::command]
async fn docker_disk() -> Result<Vec<containers::DiskUse>, String> {
    containers::disk_use().await
}

#[tauri::command]
async fn docker_clean_up(app: AppHandle, images: bool) -> Result<String, String> {
    let result = containers::clean_up(images).await;
    let _ = app.emit("docker-changed", ());
    result
}

/// Makes `docker` in a distro use TuxBridge's engine.
#[tauri::command]
fn docker_use_in(app: AppHandle, name: String) -> Result<u64, String> {
    if app.state::<Jobs>().busy(&name) {
        return Err(format!("{name} is busy with another change."));
    }
    let linux = engine::prepare_linux(&app).map_err(|e| e.to_string())?;
    let known = app.state::<Store>().distro(&name).user;
    let job = Job::new(&app, "docker", &name, &format!("Setting up docker in {name}")).map_err(|e| e.to_string())?;
    let id = job.id();
    tauri::async_runtime::spawn(async move {
        let result: Result<(), String> = async {
            let user = match known {
                Some(u) => u,
                None => wsl::output(["-d", &name, "-e", "id", "-un"])
                    .await
                    .map(|o| o.text.trim().to_string())
                    .ok()
                    .filter(|u| !u.is_empty())
                    .ok_or("This system did not answer.")?,
            };
            containers::use_in(job.clone(), &name, &user, &linux).await
        }
        .await;
        if result.is_ok() {
            app.state::<Store>().update_distro(&name, |d| d.docker = true);
        }
        job.finish(&result);
        let _ = app.emit("distros-changed", ());
    });
    Ok(id)
}

/// The gallery, with folder suggestions like %USERPROFILE%\Videos filled in for this user.
#[tauri::command]
fn list_apps() -> serde_json::Value {
    let mut list = serde_json::to_value(apps::all()).unwrap_or_default();
    let profile = dirs::home_dir().map(|h| h.to_string_lossy().into_owned()).unwrap_or_default();
    for question in list.as_array_mut().into_iter().flatten().flat_map(|a| a["questions"].as_array_mut().into_iter().flatten()) {
        if let Some(default) = question["default"].as_str() {
            question["default"] = default.replace("%USERPROFILE%", &profile).into();
        }
    }
    list
}

#[tauri::command]
fn install_app(app: AppHandle, id: String, answers: std::collections::HashMap<String, String>) -> Result<u64, String> {
    let def = apps::get(&id).ok_or("That app is not in the gallery.")?;
    if app.state::<Jobs>().busy(def.id) {
        return Err(format!("{} is being installed already.", def.name));
    }
    let job = Job::new(&app, "app", def.id, &format!("Installing {}", def.name)).map_err(|e| e.to_string())?;
    let id = job.id();
    tauri::async_runtime::spawn(async move {
        let result = containers::install_app(job.clone(), def, answers).await;
        job.finish(&result);
        let _ = app.emit("docker-changed", ());
    });
    Ok(id)
}

// ---------------------------------------------------------------------------------------------
// The assistant (ai.rs)

#[tauri::command]
async fn ai_status(app: AppHandle) -> ai::Status {
    let info = app.state::<Store>().ai();
    ai::status(info).await
}

/// Installs the assistant with a model ("small", "standard", "large"), or changes its model.
#[tauri::command]
fn ai_install(app: AppHandle, model: String) -> Result<u64, String> {
    const KEY: &str = "assistant";
    if app.state::<Jobs>().busy(KEY) {
        return Err("The assistant is being set up already.".into());
    }
    let model = ai::MODELS.iter().find(|m| m.id == model).ok_or("Unknown model.")?;
    let store = app.state::<Store>();
    let old = store.ai();
    let dir = old.as_ref().map(|i| std::path::PathBuf::from(&i.dir)).unwrap_or_else(|| store.install_dir().join("easywsl-ai"));
    let title = if old.is_some() { "Changing the assistant's model" } else { "Installing the assistant" };
    let job = Job::new(&app, "assistant", KEY, title).map_err(|e| e.to_string())?;
    let id = job.id();
    tauri::async_runtime::spawn(async move {
        let result = ai::install(job.clone(), dir.clone(), model, old.map(|o| o.model)).await;
        if result.is_ok() {
            app.state::<Store>().set_ai(Some(store::AiInfo { dir: dir.to_string_lossy().into_owned(), model: model.tag.to_string() }));
        }
        job.finish(&result);
        let _ = app.emit("ai-changed", ());
    });
    Ok(id)
}

#[tauri::command]
async fn ai_remove(app: AppHandle) -> Result<(), String> {
    let Some(info) = app.state::<Store>().ai() else { return Ok(()) };
    let dir = std::path::PathBuf::from(&info.dir);
    tauri::async_runtime::spawn_blocking(move || ai::remove(&dir)).await.map_err(|e| e.to_string())??;
    app.state::<Store>().set_ai(None);
    let _ = app.emit("ai-changed", ());
    Ok(())
}

/// Answers a conversation; the answer arrives as "ai-chunk" events with this id.
#[tauri::command]
fn ai_chat(app: AppHandle, id: u64, messages: Vec<ai::Message>) -> Result<(), String> {
    let info = app.state::<Store>().ai().ok_or("The assistant is not installed.")?;
    tauri::async_runtime::spawn(ai::chat(app.clone(), info.dir.into(), info.model, id, messages));
    Ok(())
}

/// Loads the model ahead of the first question (the Assistant page asks this when it opens).
#[tauri::command]
async fn ai_warm_up(app: AppHandle) {
    if let Some(info) = app.state::<Store>().ai() {
        ai::warm_up(std::path::Path::new(&info.dir), &info.model).await;
    }
}

#[tauri::command]
fn ai_cancel(id: u64) {
    ai::cancel(id);
}

// ---------------------------------------------------------------------------------------------
// Skills for Claude and Codex (skills.rs)

#[tauri::command]
fn skills_status() -> Vec<skills::TargetStatus> {
    skills::status()
}

#[tauri::command]
fn skill_install(app: AppHandle, target: skills::Target) -> Result<(), String> {
    skills::install(&app, target)
}

#[tauri::command]
fn skill_remove(target: skills::Target) -> Result<(), String> {
    skills::remove(target)
}

/// Saves the skill as a .zip in Downloads and shows it in File Explorer.
#[tauri::command]
fn skill_export(app: AppHandle) -> Result<String, String> {
    let zip = skills::export_zip(&app)?;
    let _ = std::process::Command::new("explorer.exe").arg(format!("/select,{zip}")).spawn();
    Ok(zip)
}

#[tauri::command]
async fn update_check(app: AppHandle) -> Result<updates::UpdateInfo, String> {
    updates::check(&app).await
}

#[tauri::command]
async fn update_install(app: AppHandle) -> Result<(), String> {
    updates::install(&app).await
}

#[tauri::command]
fn feedback_can_send() -> bool {
    feedback::can_send()
}

#[tauri::command]
async fn feedback_report(app: AppHandle) -> String {
    feedback::build_report(&app).await
}

#[tauri::command]
async fn feedback_send(app: AppHandle, request: feedback::Request) -> Result<(), String> {
    feedback::send(&app, request).await
}

#[tauri::command]
async fn feedback_save(app: AppHandle, request: feedback::Request) -> Result<String, String> {
    feedback::save(&app, request).await
}

/// Brings the main window back from the tray (or from behind other windows).
fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// The tray menu's width, in logical pixels; its height follows what it lists (`tray_fit`).
const TRAY_WIDTH: f64 = 290.0;

/// Where the tray icon was right-clicked: the tray menu opens just above it.
#[derive(Default)]
struct TrayAnchor(std::sync::Mutex<Option<tauri::PhysicalPosition<f64>>>);

/// Pip in the notification area. A left click opens the window; a right click opens TuxBridge's own
/// tray menu (a small window, src/tray/TrayMenu.tsx), because a native menu can't tell a left
/// click on an item from a right one: recent systems open their desktop on a left click and offer
/// more on a right click.
fn tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    tauri::WebviewWindowBuilder::new(app, "tray", tauri::WebviewUrl::App("index.html".into()))
        .initialization_script("window.__EASYWSL_TRAY__ = true;")
        .title("TuxBridge")
        .inner_size(TRAY_WIDTH, 300.0)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;

    let mut builder = TrayIconBuilder::with_id("main").tooltip("TuxBridge").on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click { button, button_state: MouseButtonState::Up, position, .. } = event {
            let app = tray.app_handle();
            match button {
                MouseButton::Left => show_main(app),
                MouseButton::Right => {
                    *app.state::<TrayAnchor>().0.lock().unwrap() = Some(position);
                    // The menu loads its list and calls tray_fit, which places and shows it.
                    let _ = app.emit_to("tray", "tray-shown", ());
                }
                _ => {}
            }
        }
    });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Sizes the tray menu to its content and shows it above the spot the icon was clicked, kept on
/// that screen.
#[tauri::command]
fn tray_fit(app: AppHandle, height: f64) -> Result<(), String> {
    let window = app.get_webview_window("tray").ok_or("No tray menu.")?;
    let anchor = (*app.state::<TrayAnchor>().0.lock().unwrap()).ok_or("The tray icon wasn't clicked.")?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let (w, h) = (TRAY_WIDTH * scale, height.min(640.0) * scale);
    let (mut x, mut y) = (anchor.x - w, anchor.y - h);
    if let Ok(Some(m)) = window.monitor_from_point(anchor.x, anchor.y) {
        let (mx, my) = (m.position().x as f64, m.position().y as f64);
        let (mw, mh) = (m.size().width as f64, m.size().height as f64);
        x = x.clamp(mx, mx + mw - w);
        // A taskbar at the top: open below the click instead.
        if y < my {
            y = anchor.y;
        }
        y = y.min(my + mh - h);
    }
    window.set_size(tauri::PhysicalSize::new(w, h)).map_err(|e| e.to_string())?;
    window.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())
}

#[tauri::command]
fn tray_hide(app: AppHandle) {
    if let Some(window) = app.get_webview_window("tray") {
        let _ = window.hide();
    }
}

/// Shows the main window, on `page` if given ("settings").
#[tauri::command]
fn show_window(app: AppHandle, page: Option<String>) {
    tray_hide(app.clone());
    show_main(&app);
    if let Some(page) = page {
        let _ = app.emit_to("main", "navigate", page);
    }
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// What the window remembers (Pip, Learn's progress, recent systems: its localStorage) is kept by
/// WebView2 under the app's id, which changed with each name (EasyWSL, Pengi, TuxBridge). Copied
/// once from the newest earlier one, before the window opens; caches are left behind.
fn migrate_window_data() {
    fn copy(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)?.flatten() {
            let name = entry.file_name();
            if matches!(name.to_str(), Some("Cache" | "Code Cache" | "GPUCache" | "Crashpad" | "ShaderCache" | "GrShaderCache" | "GPUPersistentCache")) {
                continue;
            }
            let target = to.join(&name);
            if entry.file_type()?.is_dir() {
                let _ = copy(&entry.path(), &target);
            } else {
                let _ = std::fs::copy(entry.path(), target);
            }
        }
        Ok(())
    }
    let Some(local) = dirs::data_local_dir() else { return };
    let new = local.join("app.rookerysoftware.tuxbridge").join("EBWebView");
    if new.exists() {
        return;
    }
    let earlier = ["com.itsoverk1ll.pengi", "com.itsoverk1ll.easywsl"].map(|id| local.join(id).join("EBWebView"));
    if let Some(old) = earlier.iter().find(|d| d.exists()) {
        let _ = copy(old, &new);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    migrate_window_data();
    tauri::Builder::default()
        // Opening TuxBridge again while it sits in the tray shows the window it already has.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_main(app)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Closing the window keeps TuxBridge in the tray (desktops, installs and Docker keep going);
        // Quit in the tray's menu ends it.
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
                api.prevent_close();
                let _ = window.hide();
            }
            // The tray menu goes away when you click elsewhere, like a menu.
            tauri::WindowEvent::Focused(false) if window.label() == "tray" => {
                let _ = window.hide();
            }
            _ => {}
        })
        .manage(TrayAnchor::default())
        .manage(Store::load())
        .manage(Jobs::default())
        .manage(desktop::Sessions::default())
        .setup(|app| {
            tray(app)?;
            // 0.1.1 could leave a runaway copy inside its own folder; tidy it away in the background.
            let handle = app.handle().clone();
            std::thread::spawn(move || engine::clean_up_runaway_copies(&handle));
            // Docker comes back with the app (after Windows restarted, say), when it was set up
            // and the setting is on.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if handle.state::<Store>().settings().start_docker && containers::installed() && !containers::status().await.running {
                    if containers::start().is_ok() && containers::wait_ready(std::time::Duration::from_secs(60)).await {
                        let _ = handle.emit("docker-changed", ());
                    }
                }
            });
            // Installed TuxBridge skills follow the app's version.
            let handle = app.handle().clone();
            std::thread::spawn(move || skills::refresh(&handle));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            system_check,
            system_fix,
            list_distros,
            get_catalog,
            suggest_name,
            suggest_user,
            default_location,
            check_install,
            start_install,
            add_desktop,
            list_jobs,
            dismiss_job,
            job_log_tail,
            open_terminal,
            update_system,
            start_quest,
            open_files,
            reveal,
            stop_distro,
            set_default,
            remove_distro,
            open_desktop,
            list_sessions,
            get_settings,
            set_settings,
            default_install_dir,
            restart_wsl,
            docker_status,
            docker_enable,
            docker_start,
            docker_stop,
            docker_remove,
            docker_containers,
            docker_container_action,
            docker_update,
            docker_logs,
            docker_disk,
            docker_clean_up,
            list_apps,
            install_app,
            docker_use_in,
            ai_status,
            ai_install,
            ai_remove,
            ai_chat,
            ai_cancel,
            ai_warm_up,
            skills_status,
            skill_install,
            skill_remove,
            skill_export,
            tray_fit,
            tray_hide,
            show_window,
            quit_app,
            update_check,
            update_install,
            feedback_can_send,
            feedback_report,
            feedback_send,
            feedback_save,
        ])
        .run(tauri::generate_context!())
        .expect("error while running TuxBridge");
}
