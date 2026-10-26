mod catalog;
mod desktop;
mod docker;
mod engine;
mod install;
mod jobs;
mod paths;
mod store;
mod system;
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

/// Asks a running system EasyWSL did not make what it is: its kind (for the right title and
/// colour), its user, whether it has hyprland-wsl's desktop. Only running ones are asked, since
/// asking would start a stopped one.
async fn probe(app: AppHandle, name: String) {
    let script = r#"id -un; . /etc/os-release 2>/dev/null; echo "ID=$ID"
        [ -x /usr/local/bin/hyprland-wsl ] && echo HYPRLAND
        sed -n 's/^HYPR_USER=/HYPR_USER=/p' /etc/hyprland-wsl.conf 2>/dev/null
        [ -e /etc/hyprland-wsl/first-start ] && echo FIRST_START; true"#;
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
        "linuxmint" => Some("mint"),
        "cachyos" => Some("cachyos"),
        "omarchy" => Some("omarchy"),
        _ => None,
    };
    let hyprland = lines.contains(&"HYPRLAND");
    let hypr_user = lines.iter().find_map(|l| l.strip_prefix("HYPR_USER=")).map(String::from).filter(|u| !u.is_empty());
    let first_start = lines.contains(&"FIRST_START");
    app.state::<Store>().update_distro(&name, |d| {
        d.probed = true;
        d.catalog_id = catalog_id.map(String::from);
        d.hyprland = hyprland;
        d.user = hypr_user.clone().or(default_user.clone());
        d.first_start_pending = first_start;
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

#[tauri::command]
fn add_hyprland(app: AppHandle, name: String, main_key: String, allow_third_party: bool) -> Result<u64, String> {
    let store = app.state::<Store>();
    if app.state::<Jobs>().busy(&name) {
        return Err(format!("{name} is busy with another change."));
    }
    let info = store.distro(&name);
    let job = Job::new(&app, "hyprland", &name, &format!("Adding the Hyprland desktop to {name}")).map_err(|e| e.to_string())?;
    let id = job.id();
    tauri::async_runtime::spawn(async move {
        let result: Result<(), String> = async {
            job.plan(&[("Finding your account", None), ("Installing the Hyprland desktop", Some("2 to 5 minutes"))]);
            job.step("Finding your account");
            let user = match info.user {
                Some(u) => u,
                // A distro EasyWSL did not make: its default user.
                None => wsl::output(["-d", &name, "-e", "id", "-un"])
                    .await
                    .map(|o| o.text.trim().to_string())
                    .ok()
                    .filter(|u| !u.is_empty() && u != "root")
                    .ok_or("This system has no user account besides root. Create one first.")?,
            };
            let engine = engine::prepare(&app).map_err(|e| e.to_string())?;
            job.step("Installing the Hyprland desktop");
            install::install_hyprland(&job, &engine, &name, &user, allow_third_party, Some(&main_key)).await?;
            app.state::<Store>().update_distro(&name, |d| {
                d.hyprland = true;
                d.user = Some(user.clone());
                d.main_key = Some(main_key.clone());
            });
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

#[tauri::command]
fn dismiss_job(jobs: State<'_, Jobs>, id: u64) {
    jobs.dismiss(id);
}

#[tauri::command]
fn open_terminal(name: String) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let terminal = dirs::data_local_dir().map(|d| d.join(r"Microsoft\WindowsApps\wt.exe")).filter(|p| p.exists());
    let result = match terminal {
        Some(wt) => std::process::Command::new(wt).args(["-w", "new", "wsl.exe", "-d", &name, "--cd", "~"]).spawn(),
        None => std::process::Command::new(wsl::exe())
            .args(["-d", &name, "--cd", "~"])
            .creation_flags(wsl::CREATE_NEW_CONSOLE)
            .spawn(),
    };
    result.map(|_| ()).map_err(|e| format!("The terminal did not open: {e}"))
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
fn open_desktop(app: AppHandle, name: String) {
    tauri::async_runtime::spawn(async move {
        let _ = desktop::open(app.clone(), name).await;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Store::load())
        .manage(Jobs::default())
        .manage(desktop::Sessions::default())
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
            add_hyprland,
            list_jobs,
            dismiss_job,
            open_terminal,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running EasyWSL");
}
