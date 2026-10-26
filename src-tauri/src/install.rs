//! Installing a distro from the catalogue: getting it, its account, optionally the Hyprland
//! desktop. The same steps as hyprland-wsl's new-distro.ps1 and install-omarchy.ps1, which were
//! tested on every distro listed.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::catalog::{self, Hyprland, Source};
use crate::jobs::{Job, now};
use crate::store::Store;
use crate::{engine, paths, wsl};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub catalog_id: String,
    pub name: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub desktop: Choice,
    pub main_key: Option<String>,
    pub allow_third_party: bool,
    pub location: Option<String>,
    pub set_default: bool,
}

/// Which desktop a system gets: its own (Cinnamon, KDE Plasma, Xfce), Hyprland, or none (only a
/// terminal).
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Choice {
    Own,
    Hyprland,
    None,
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 64 && name.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

pub fn valid_user(user: &str) -> bool {
    let mut chars = user.chars();
    user != "root"
        && user.len() <= 32
        && chars.next().is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Checks a request before anything is downloaded; the message is shown next to the form.
pub fn check(app: &AppHandle, req: &Request) -> Result<(catalog::Entry, PathBuf), String> {
    let entry = catalog::get(&req.catalog_id).ok_or("That system is not in the catalogue.")?;
    if !valid_name(&req.name) {
        return Err("The name can have letters, digits, dots, dashes and underscores.".into());
    }
    if wsl::exists(&req.name) {
        return Err(format!("You already have a system called '{}'. Choose another name.", req.name));
    }
    if entry.source != Source::Omarchy {
        let user = req.user.as_deref().unwrap_or("");
        if !valid_user(user) {
            return Err("The user name has to be lower-case letters, digits, - and _, starting with a letter.".into());
        }
        if req.password.as_deref().unwrap_or("").is_empty() {
            return Err("Choose a password. Linux asks for it when you install software.".into());
        }
    }
    let hyprland = req.desktop == Choice::Hyprland;
    if hyprland && entry.hyprland == Hyprland::No {
        return Err(format!("The Hyprland desktop is not available for {} yet.", entry.title));
    }
    if req.desktop == Choice::Own && entry.desktop.is_none() {
        return Err(format!("{} has no desktop of its own here; choose Hyprland or none.", entry.title));
    }
    if hyprland && entry.third_party.is_some() && !req.allow_third_party {
        return Err("Hyprland needs the community package sources for this system; allow them, or leave Hyprland off.".into());
    }
    let store = app.state::<Store>();
    let location = match &req.location {
        Some(l) if !l.trim().is_empty() => PathBuf::from(l.trim()),
        _ => store.install_dir().join(&req.name),
    };
    if location.exists() && std::fs::read_dir(&location).map(|mut d| d.next().is_some()).unwrap_or(true) {
        return Err(format!("The folder {} is not empty. Choose another name or folder.", location.display()));
    }
    Ok((entry, location))
}

/// One plain sentence for a failed step: what did not work and what to try, recognising the
/// common causes in the log.
fn explain(job: &Job, what: &str, try_this: &str) -> String {
    let tail = job.log_tail();
    let try_this = if wsl::vm_stuck(&tail) {
        "WSL stopped responding. Restart WSL (My systems shows a button for it), then try again."
    } else if tail.contains("PATH_NOT_FOUND") || tail.contains("cannot find the path") {
        "The folder for the system cannot be used; check that its drive exists, or choose another folder in Settings."
    } else if tail.contains("DISK_FULL") || tail.contains("No space left") || tail.contains("not enough space") {
        "The drive is full. Free a few GB, or choose a folder on another drive in Settings."
    } else if ["Could not resolve", "Temporary failure in name resolution", "timed out", "Connection refused", "error sending request"]
        .iter()
        .any(|s| tail.contains(s))
    {
        "Check the internet connection and try again."
    } else {
        try_this
    };
    format!("{what} did not work. {try_this}")
}

const NET: &str = "Check the internet connection and try again.";
const AGAIN: &str = "Try again. If it fails again, please report it with the log file.";

pub async fn run(app: AppHandle, job: Arc<Job>, req: Request, entry: catalog::Entry, location: PathBuf) -> Result<(), String> {
    let name = req.name.clone();
    let title = entry.title;
    let omarchy = entry.source == Source::Omarchy;
    let hyprland = omarchy || req.desktop == Choice::Hyprland;
    let own = (req.desktop == Choice::Own && !omarchy).then(|| entry.desktop.clone()).flatten();
    let own_step = own.as_ref().map(|d| format!("Installing the {} desktop", d.name));
    let own_estimate = own.as_ref().map(|d| format!("{} minutes", d.minutes));
    let user = req.user.clone().unwrap_or_default();

    let get = format!("Downloading {title}");
    let account = format!("Setting up your account ({user})");
    let mut plan: Vec<(&str, Option<&str>)> = vec![(&get, Some("1 to 2 minutes"))];
    if let Source::Docker { .. } = entry.source {
        plan.push(("Unpacking it", Some("1 minute")));
    }
    if omarchy {
        plan.push(("Installing Omarchy", Some("10 to 15 minutes")));
    } else {
        plan.push((&account, Some("1 to 2 minutes")));
        if hyprland {
            plan.push(("Installing the Hyprland desktop", Some("2 to 5 minutes")));
        }
        if let (Some(step), Some(estimate)) = (&own_step, &own_estimate) {
            plan.push((step, Some(estimate)));
        }
    }
    plan.push(("Finishing", None));
    job.plan(&plan);

    let engine = engine::prepare(&app).map_err(|e| format!("Preparing the installer did not work: {e}"))?;
    let linux = engine::prepare_linux(&app).map_err(|e| format!("Preparing the installer did not work: {e}"))?;
    if let Some(dir) = location.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("The folder {} cannot be created: {e}", dir.display()))?;
    }
    let loc = location.to_string_lossy().to_string();

    // From here on a failure leaves a half-made distro, which is removed again.
    let result: Result<(), String> = async {
        job.step(&get);
        match entry.source {
            Source::Wsl { name: wsl_name } => {
                job.run(wsl::command(["--install", wsl_name, "--name", &name, "--location", &loc, "--no-launch"]), None)
                    .await
                    .map_err(|_| explain(&job, &format!("Downloading {title}"), NET))?;
            }
            Source::Omarchy => {
                job.run(wsl::command(["--install", "archlinux", "--name", &name, "--location", &loc, "--no-launch"]), None)
                    .await
                    .map_err(|_| explain(&job, "Downloading Arch Linux", NET))?;
            }
            Source::Docker { image } => {
                let dir = paths::downloads_dir().join(format!("image-{name}"));
                let _ = std::fs::remove_dir_all(&dir);
                let mut last = 0u64;
                let layers = crate::docker::download(image, &dir, |done, total| {
                    let percent = if total > 0 { done * 100 / total } else { 0 };
                    if percent != last {
                        last = percent;
                        job.log_line(&format!("downloaded {percent}% of {} MB", total / 1_000_000));
                    }
                })
                .await
                .map_err(|e| {
                    job.log_line(&format!("{e:#}"));
                    explain(&job, &format!("Downloading {title}"), NET)
                })?;
                job.step("Unpacking it");
                let rootfs = if layers.len() == 1 {
                    layers[0].clone()
                } else {
                    // Several layers are merged into one in a throwaway distro made from the first.
                    let merge = format!("{name}-merge");
                    let merge_dir = dir.join("merge");
                    let unpack = "Try again; the download may have been incomplete.";
                    job.run(wsl::command(["--import".as_ref(), merge.as_ref(), merge_dir.as_os_str(), layers[0].as_os_str()]), None)
                        .await
                        .map_err(|_| explain(&job, "Unpacking the download", unpack))?;
                    let dir_wsl = paths::to_wsl(&dir).map_err(|e| e.to_string())?;
                    let merged = job
                        .run(
                            wsl::command(["-d", &merge, "-u", "root", "-e", "bash", &format!("{engine}/tools/merge-layers.sh"), &dir_wsl, &format!("{dir_wsl}/rootfs.tar.gz")]),
                            None,
                        )
                        .await;
                    let _ = wsl::output(["--unregister", &merge]).await;
                    merged.map_err(|_| explain(&job, "Unpacking the download", unpack))?;
                    dir.join("rootfs.tar.gz")
                };
                let created = job
                    .run(wsl::command(["--import".as_ref(), name.as_ref(), location.as_os_str(), rootfs.as_os_str()]), None)
                    .await;
                let _ = std::fs::remove_dir_all(&dir);
                created.map_err(|_| explain(&job, &format!("Creating {title}"), "Check that the drive has a few GB free and try again."))?;
            }
        }

        if omarchy {
            job.step("Installing Omarchy");
            let script = format!("{engine}/tools/omarchy-wsl.sh");
            // The script stops once with 75 when the packages it installed replaced systemd:
            // the instance restarts and the script carries on where it stopped.
            for attempt in 0..2 {
                match job.run(wsl::command(["-d", &name, "-u", "root", "-e", "bash", &script]), None).await {
                    Ok(()) => break,
                    Err(e) if attempt == 0 && e.to_string().contains("exit code 75") => {
                        job.log_line("restarting the instance, then continuing");
                        wsl::terminate(&name).await;
                    }
                    Err(_) => return Err(explain(&job, "Installing Omarchy", AGAIN)),
                }
            }
        } else {
            job.step(&account);
            let hostname: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' }).collect();
            let mut args = vec!["-d".to_string(), name.clone(), "-u".into(), "root".into(), "-e".into(), "env".into()];
            args.push(format!("NEW_USER={user}"));
            args.push(format!("NEW_HOSTNAME={hostname}"));
            if entry.id == "mint" {
                args.push("NEW_MINT=1".into());
            }
            args.extend(["bash".into(), format!("{engine}/tools/new-distro-setup.sh")]);
            job.run(wsl::command(&args), req.password.as_deref())
                .await
                .map_err(|_| explain(&job, "Setting up the account", "Try again in a few minutes; the system's download servers may be busy."))?;

            if hyprland {
                job.step("Installing the Hyprland desktop");
                install_hyprland(&job, &engine, &name, &user, req.allow_third_party, req.main_key.as_deref()).await?;
            }
            if let (Some(desktop), Some(step)) = (&own, &own_step) {
                job.step(step);
                install_own_desktop(&job, &linux, &name, &user, desktop).await?;
            }
        }
        Ok(())
    }
    .await;

    if let Err(e) = result {
        if wsl::exists(&name) {
            // A half-made system is of no use; taking it away lets a retry start clean.
            let _ = wsl::output_within(["--unregister", &name], std::time::Duration::from_secs(120)).await;
            if std::fs::read_dir(&location).map(|mut d| d.next().is_none()).unwrap_or(false) {
                let _ = std::fs::remove_dir(&location);
            }
            job.log_line(&format!("removed the half-made '{name}'"));
        }
        return Err(e);
    }

    job.step("Finishing");
    // Restart for the hostname and default user, and skip the distro's own first-run account
    // creation, since the account exists (Omarchy's setup makes its own at the first start).
    wsl::terminate(&name).await;
    if !omarchy {
        wsl::skip_oobe(&name);
    }
    if req.set_default {
        let _ = wsl::output(["--set-default", &name]).await;
    }
    app.state::<Store>().update_distro(&name, |d| {
        d.catalog_id = Some(entry.id.to_string());
        d.user = (!omarchy).then(|| user.clone());
        d.hyprland = hyprland;
        d.desktop = own.as_ref().map(|o| o.id.to_string());
        d.main_key = hyprland.then(|| req.main_key.clone().unwrap_or_else(|| "SUPER".into()));
        d.created_by_easywsl = true;
        d.first_start_pending = omarchy;
        d.installed_at = Some(now());
    });
    Ok(())
}

/// Runs hyprland-wsl's installer in a distro, and sets the main key.
pub async fn install_hyprland(job: &Job, engine: &str, name: &str, user: &str, third_party: bool, main_key: Option<&str>) -> Result<(), String> {
    let mut args = vec!["-d".to_string(), name.into(), "-u".into(), "root".into(), "-e".into(), "env".into()];
    args.push(format!("HYPR_USER={user}"));
    if third_party {
        args.push("HYPRLAND_PPA=1".into());
        args.push("HYPRLAND_MESA_PPA=1".into());
    }
    args.extend(["bash".into(), format!("{engine}/install.sh")]);
    job.run(wsl::command(&args), None).await.map_err(|_| explain(job, "Installing the Hyprland desktop", AGAIN))?;
    let key = main_key.unwrap_or("SUPER");
    job.run(wsl::command(["-d", name, "-u", "root", "-e", "hyprland-wsl", "set-mod", key]), None)
        .await
        .map_err(|_| explain(job, "Setting the desktop's main key", AGAIN))?;
    Ok(())
}

/// Installs a distro's own desktop (linux/desktop-setup.sh).
pub async fn install_own_desktop(job: &Job, linux: &str, name: &str, user: &str, desktop: &catalog::Desktop) -> Result<(), String> {
    job.run(wsl::command(["-d", name, "-u", "root", "-e", "bash", &format!("{linux}/desktop-setup.sh"), desktop.id, user]), None)
        .await
        .map_err(|_| explain(job, &format!("Installing the {} desktop", desktop.name), AGAIN))
}
