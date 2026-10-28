//! Updating TuxBridge itself (Settings → Update TuxBridge), with Tauri's updater.
//!
//! The app asks https://tuxbridge.com/download/latest.json what the newest version is. That file
//! names the installer and carries its signature, made with Rookery Software's private key when
//! the release was built; the public key is in tauri.conf.json, and nothing whose signature
//! doesn't match is installed. The installer then runs on its own ("passive": a progress window,
//! no questions), replaces the app and starts it again; settings and systems stay.

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current: String,
    /// The newer version on offer, if there is one.
    pub available: Option<String>,
    /// Its release notes.
    pub notes: Option<String>,
}

/// An updater error in words for people.
fn explain(e: tauri_plugin_updater::Error) -> String {
    use tauri_plugin_updater::Error;
    match e {
        Error::Reqwest(_) => "Couldn't reach tuxbridge.com to check for updates. Are you online?".into(),
        Error::Network(_) => "Couldn't reach tuxbridge.com to check for updates. Are you online?".into(),
        Error::ReleaseNotFound => "No update information was found on tuxbridge.com yet.".into(),
        Error::Minisign(_) | Error::SignatureUtf8(_) | Error::Base64(_) => {
            "The update's signature didn't match, so it wasn't installed. Please download TuxBridge from tuxbridge.com instead.".into()
        }
        other => format!("The update didn't work: {other}"),
    }
}

pub async fn check(app: &AppHandle) -> Result<UpdateInfo, String> {
    let current = app.package_info().version.to_string();
    let update = app.updater().map_err(explain)?.check().await.map_err(explain)?;
    Ok(UpdateInfo {
        current,
        available: update.as_ref().map(|u| u.version.clone()),
        notes: update.and_then(|u| u.body.clone()),
    })
}

/// Downloads the newest version (its progress goes to the window as "update-progress" events:
/// [bytes so far, total bytes or null]), checks its signature, and starts its installer. On
/// Windows the app closes while the installer runs, and the installer starts it again.
pub async fn install(app: &AppHandle) -> Result<(), String> {
    let update = app.updater().map_err(explain)?.check().await.map_err(explain)?;
    let Some(update) = update else {
        return Err("TuxBridge is already up to date.".into());
    };
    let progress = app.clone();
    let mut received: u64 = 0;
    update
        .download_and_install(
            move |chunk, total| {
                received += chunk as u64;
                let _ = progress.emit("update-progress", (received, total));
            },
            || {},
        )
        .await
        .map_err(explain)?;
    Ok(())
}
