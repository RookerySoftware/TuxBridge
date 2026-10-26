//! easywsl-agent: runs inside Linux, next to a desktop session, and hands it to the EasyWSL
//! viewer on Windows. Frames go out on stdout, input comes in on stdin (see easywsl-protocol);
//! the viewer starts it through wsl.exe, which carries both over WSL's own Hyper-V socket.
//! Everything else (logs) goes to stderr.
//!
//!   easywsl-agent gnome [CONNECTOR]     a GNOME Shell session (Mutter's ScreenCast and
//!                                       RemoteDesktop interfaces); CONNECTOR defaults to Meta-0,
//!                                       the virtual monitor of gnome-shell --headless
//!   easywsl-agent wlroots               a wlroots-style Wayland compositor such as Hyprland
//!                                       (wlr-screencopy, virtual pointer and keyboard), on
//!                                       $WAYLAND_DISPLAY
//!   easywsl-agent x11                   an X11 desktop on a virtual X server (Xvfb), on $DISPLAY
//!
//! `--fps N` before the kind sets the most frames a second it sends (default 60).

#[cfg(target_os = "linux")]
mod gnome;
mod sender;
#[cfg(target_os = "linux")]
mod wlroots;
#[cfg(target_os = "linux")]
mod x11;

/// The most frames a second to send.
pub static FPS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(60);

/// Whether to log details (timings, buffers): EASYWSL_AGENT_DEBUG is set.
pub fn debug() -> bool {
    static DEBUG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *DEBUG.get_or_init(|| std::env::var_os("EASYWSL_AGENT_DEBUG").is_some())
}

/// The shortest time between two frames.
pub fn frame_interval() -> std::time::Duration {
    std::time::Duration::from_secs_f64(1.0 / FPS.load(std::sync::atomic::Ordering::Relaxed).max(1) as f64)
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--fps") {
        match args.get(1).and_then(|v| v.parse::<u32>().ok()).filter(|&f| (1..=500).contains(&f)) {
            Some(fps) => FPS.store(fps, std::sync::atomic::Ordering::Relaxed),
            None => {
                eprintln!("easywsl-agent: --fps needs a number from 1 to 500");
                std::process::exit(2);
            }
        }
        args.drain(..2);
    }
    let result: Result<(), Box<dyn std::error::Error>> = match args.first().map(String::as_str) {
        #[cfg(target_os = "linux")]
        Some("gnome") => gnome::run(args.get(1).map(String::as_str).unwrap_or("Meta-0")),
        #[cfg(target_os = "linux")]
        Some("wlroots") => wlroots::run(),
        #[cfg(target_os = "linux")]
        Some("x11") => x11::run(),
        _ => Err("usage: easywsl-agent [--fps N] gnome [CONNECTOR] | wlroots | x11".into()),
    };
    if let Err(e) = result {
        eprintln!("easywsl-agent: {e}");
        std::process::exit(1);
    }
}
