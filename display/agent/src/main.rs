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

#[cfg(target_os = "linux")]
mod gnome;
mod sender;
#[cfg(target_os = "linux")]
mod wlroots;
#[cfg(target_os = "linux")]
mod x11;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result: Result<(), Box<dyn std::error::Error>> = match args.first().map(String::as_str) {
        #[cfg(target_os = "linux")]
        Some("gnome") => gnome::run(args.get(1).map(String::as_str).unwrap_or("Meta-0")),
        #[cfg(target_os = "linux")]
        Some("wlroots") => wlroots::run(),
        #[cfg(target_os = "linux")]
        Some("x11") => x11::run(),
        _ => Err("usage: easywsl-agent gnome [CONNECTOR] | wlroots | x11".into()),
    };
    if let Err(e) = result {
        eprintln!("easywsl-agent: {e}");
        std::process::exit(1);
    }
}
