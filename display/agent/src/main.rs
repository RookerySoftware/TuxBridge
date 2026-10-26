//! easywsl-agent: runs inside Linux, next to a desktop session, and hands it to the EasyWSL
//! viewer on Windows. Frames go out on stdout, input comes in on stdin (see easywsl-protocol);
//! the viewer starts it through wsl.exe, which carries both over WSL's own Hyper-V socket.
//! Everything else (logs) goes to stderr.
//!
//!   easywsl-agent gnome [CONNECTOR]     a GNOME Shell session (Mutter's ScreenCast and
//!                                       RemoteDesktop interfaces); CONNECTOR defaults to Meta-0,
//!                                       the virtual monitor of gnome-shell --headless

#[cfg(target_os = "linux")]
mod gnome;
mod sender;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result: Result<(), Box<dyn std::error::Error>> = match args.first().map(String::as_str) {
        #[cfg(target_os = "linux")]
        Some("gnome") => gnome::run(args.get(1).map(String::as_str).unwrap_or("Meta-0")),
        _ => Err("usage: easywsl-agent gnome [CONNECTOR]".into()),
    };
    if let Err(e) = result {
        eprintln!("easywsl-agent: {e}");
        std::process::exit(1);
    }
}
