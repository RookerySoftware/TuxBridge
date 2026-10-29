//! easywsl-viewer: a window on Windows showing a Linux desktop. It starts the command it is given
//! (wsl.exe running easywsl-agent next to the desktop session), draws the frames the agent writes
//! to its stdout, and writes keyboard and mouse input to its stdin. No network is involved:
//! wsl.exe carries both over WSL's own Hyper-V socket.
//!
//!   easywsl-viewer [--fullscreen] [--title TITLE] [--size WxH] -- COMMAND [ARGS...]
//!
//! Ctrl+Alt+Enter, or "Full screen" in the window's menu, switches between borderless full screen
//! and a window. Closing the window ends the command
//! (its input is closed first, so it can end the session itself). Exit code 3: the command ended
//! before it sent a picture.

#![windows_subsystem = "windows"]

mod fullscreen;
mod hook;
mod keys;

use std::collections::HashSet;
use std::io::{BufReader, BufWriter, Write};
use std::num::NonZeroU32;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use easywsl_protocol::{Event, Input, VERSION, apply_tiles};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CustomCursor, Fullscreen, Window, WindowId};

/// The desktop's picture as last received: BGRA pixels, the protocol's own order, which is also
/// what the window wants (0RGB in a little-endian u32).
struct Desktop {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

pub enum Message {
    /// New pixels arrived.
    Frame,
    /// The agent ended (the desktop session closed); the text says why, when it is known.
    Ended(String),
    /// The desktop's pointer took a new shape (RGBA), or was hidden (no pixels).
    Cursor { hot_x: u16, hot_y: u16, width: u16, height: u16, rgba: Vec<u8> },
    /// Full screen or a window (from the window's menu).
    ToggleFullscreen,
    /// The full-screen hint's time is up.
    HintOver,
}

/// Input on its way to the agent, in one queue (so a key never overtakes the one before it),
/// written by a thread of its own: sending never waits for Linux, which the keyboard hook's
/// thread must not (Windows quietly removes a hook that keeps it waiting).
#[derive(Clone)]
pub struct InputSink(Arc<Mutex<Option<Sender<Input>>>>);

impl InputSink {
    fn new(stdin: ChildStdin) -> InputSink {
        let (queue, inputs) = std::sync::mpsc::channel::<Input>();
        std::thread::spawn(move || {
            let mut w = BufWriter::new(stdin);
            for input in inputs {
                if input.write(&mut w).and_then(|_| w.flush()).is_err() {
                    break;
                }
            }
        });
        InputSink(Arc::new(Mutex::new(Some(queue))))
    }

    pub fn send(&self, input: Input) {
        if let Some(queue) = self.0.lock().unwrap().as_ref() {
            let _ = queue.send(input);
        }
    }

    /// Ends the agent's input (once what is queued is written), which tells it to end the session.
    fn close(&self) {
        self.0.lock().unwrap().take();
    }
}

struct Args {
    fullscreen: bool,
    title: String,
    size: (u32, u32),
    command: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args { fullscreen: false, title: "TuxBridge".into(), size: (1600, 900), command: Vec::new() };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--fullscreen" => args.fullscreen = true,
            "--title" => args.title = it.next().ok_or("--title needs a value")?,
            "--size" => {
                let v = it.next().ok_or("--size needs WxH")?;
                let (w, h) = v.split_once('x').ok_or("--size needs WxH")?;
                args.size = (w.parse().map_err(|_| "bad width")?, h.parse().map_err(|_| "bad height")?);
            }
            "--" => {
                args.command = it.collect();
                break;
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    if args.command.is_empty() {
        return Err("usage: easywsl-viewer [--fullscreen] [--title T] [--size WxH] -- COMMAND [ARGS...]".into());
    }
    Ok(args)
}

struct App {
    args: Args,
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    desktop: Arc<Mutex<Desktop>>,
    input: InputSink,
    child: Child,
    /// Keys held down (evdev codes), released when the window loses focus.
    held: HashSet<u32>,
    ctrl: bool,
    alt: bool,
    /// Where the desktop is drawn in the window: offset and scale (it keeps its aspect ratio).
    placement: (f64, f64, f64),
    ended: Option<String>,
    /// A picture has arrived.
    pictured: bool,
    proxy: EventLoopProxy<Message>,
    /// Until when the full-screen hint shows.
    hint_until: Option<std::time::Instant>,
}

impl App {
    fn toggle_fullscreen(&mut self) {
        if let Some(w) = &self.window {
            let full = w.fullscreen().is_some();
            w.set_fullscreen(if full { None } else { Some(Fullscreen::Borderless(None)) });
            if full {
                self.hint_until = None;
            } else {
                self.show_hint();
            }
        }
    }

    /// Says for a few seconds how to leave full screen.
    fn show_hint(&mut self) {
        const SHOWN: std::time::Duration = std::time::Duration::from_secs(4);
        self.hint_until = Some(std::time::Instant::now() + SHOWN);
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            std::thread::sleep(SHOWN);
            let _ = proxy.send_event(Message::HintOver);
        });
    }

    fn release_all(&mut self) {
        for code in self.held.drain() {
            self.input.send(Input::Key { code, pressed: false });
        }
        hook::release();
        self.ctrl = false;
        self.alt = false;
    }

    fn key(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else { return };
        let pressed = event.state == ElementState::Pressed;
        // Linux repeats held keys itself; Windows' repeats would count as more presses there.
        if event.repeat {
            return;
        }
        match code {
            KeyCode::ControlLeft | KeyCode::ControlRight => self.ctrl = pressed,
            KeyCode::AltLeft | KeyCode::AltRight => self.alt = pressed,
            _ => {}
        }
        // Ctrl+Alt+Enter belongs to the viewer: full screen or window.
        if pressed && code == KeyCode::Enter && self.ctrl && self.alt {
            self.toggle_fullscreen();
            return;
        }
        let Some(evdev) = keys::evdev(code) else { return };
        if pressed {
            self.held.insert(evdev);
        } else if !self.held.remove(&evdev) {
            return;
        }
        self.input.send(Input::Key { code: evdev, pressed });
    }

    fn redraw(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else { return };
        let size = window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return };
        if surface.resize(w, h).is_err() {
            return;
        }
        let Ok(mut buffer) = surface.buffer_mut() else { return };
        let desktop = self.desktop.lock().unwrap();
        let (ww, wh) = (size.width as usize, size.height as usize);
        if desktop.width == 0 {
            buffer.fill(0x0011_1318);
        } else if desktop.width as usize == ww && desktop.height as usize == wh {
            // The usual case: one desktop pixel per window pixel.
            for (dst, src) in buffer.iter_mut().zip(&desktop.pixels) {
                *dst = src & 0x00ff_ffff;
            }
            self.placement = (0.0, 0.0, 1.0);
        } else {
            // A different size: scale to fit, keeping the aspect ratio (nearest pixel).
            let scale = (ww as f64 / desktop.width as f64).min(wh as f64 / desktop.height as f64);
            let (dw, dh) = ((desktop.width as f64 * scale) as usize, (desktop.height as f64 * scale) as usize);
            let (ox, oy) = ((ww - dw.min(ww)) / 2, (wh - dh.min(wh)) / 2);
            self.placement = (ox as f64, oy as f64, scale);
            buffer.fill(0);
            // Which desktop column each window column shows, worked out once per picture.
            let columns: Vec<usize> = (0..dw.min(ww)).map(|x| ((x as f64 / scale) as usize).min(desktop.width as usize - 1)).collect();
            let source = &desktop.pixels;
            for y in 0..dh.min(wh) {
                let sy = ((y as f64 / scale) as usize).min(desktop.height as usize - 1);
                let row = &source[sy * desktop.width as usize..];
                let start = (oy + y) * ww + ox;
                let dst = &mut buffer[start..start + columns.len()];
                for (d, &sx) in dst.iter_mut().zip(&columns) {
                    *d = row[sx] & 0x00ff_ffff;
                }
            }
        }
        drop(desktop);
        let _ = buffer.present();
        if self.hint_until.is_some_and(|t| t > std::time::Instant::now()) {
            fullscreen::draw_hint(window);
        }
    }

    fn pointer(&self, position: PhysicalPosition<f64>) {
        let (ox, oy, scale) = self.placement;
        let desktop = self.desktop.lock().unwrap();
        if desktop.width == 0 {
            return;
        }
        let x = ((position.x - ox) / scale).clamp(0.0, desktop.width as f64 - 1.0);
        let y = ((position.y - oy) / scale).clamp(0.0, desktop.height as f64 - 1.0);
        drop(desktop);
        self.input.send(Input::PointerMove { x, y });
    }
}

impl ApplicationHandler<Message> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let mut attributes = Window::default_attributes()
            .with_title(&self.args.title)
            // Real pixels: one desktop pixel per screen pixel keeps the picture sharp on a scaled
            // Windows display (the desktop draws its own scaling).
            .with_inner_size(PhysicalSize::new(self.args.size.0, self.args.size.1));
        if self.args.fullscreen {
            attributes = attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        // Pip, from the icon build.rs puts in the exe, on the title bar and the taskbar.
        #[cfg(windows)]
        {
            use winit::platform::windows::{IconExtWindows, WindowAttributesExtWindows};
            if let Ok(icon) = winit::window::Icon::from_resource(1, None) {
                attributes = attributes.with_window_icon(Some(icon.clone())).with_taskbar_icon(Some(icon));
            }
        }
        let window = Rc::new(event_loop.create_window(attributes).expect("window"));
        let context = softbuffer::Context::new(window.clone()).expect("drawing context");
        self.surface = Some(softbuffer::Surface::new(&context, window.clone()).expect("drawing surface"));
        hook::set_window(&window);
        fullscreen::add_menu_item(&window, self.proxy.clone());
        if self.args.fullscreen {
            self.show_hint();
        }
        self.window = Some(window);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, message: Message) {
        match message {
            Message::Frame => {
                self.pictured = true;
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            Message::ToggleFullscreen => self.toggle_fullscreen(),
            Message::HintOver => {
                self.hint_until = None;
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            Message::Ended(reason) => {
                self.ended = Some(reason);
                event_loop.exit();
            }
            // The desktop's pointer becomes this window's own Windows pointer: it moves with the
            // mouse at once, with no round trip to Linux.
            Message::Cursor { hot_x, hot_y, width, height, rgba } => {
                let Some(window) = &self.window else { return };
                if width == 0 {
                    window.set_cursor_visible(false);
                    return;
                }
                if let Ok(source) = CustomCursor::from_rgba(rgba, width, height, hot_x, hot_y) {
                    window.set_cursor(event_loop.create_custom_cursor(source));
                    window.set_cursor_visible(true);
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Resized(_) => {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::Focused(focused) => {
                hook::set_focused(focused);
                if !focused {
                    self.release_all();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.key(&event),
            WindowEvent::CursorMoved { position, .. } => self.pointer(position),
            WindowEvent::MouseInput { state, button, .. } => {
                let code = match button {
                    MouseButton::Left => 0x110,
                    MouseButton::Right => 0x111,
                    MouseButton::Middle => 0x112,
                    MouseButton::Back => 0x113,
                    MouseButton::Forward => 0x114,
                    MouseButton::Other(_) => return,
                };
                self.input.send(Input::Button { code, pressed: state == ElementState::Pressed });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x.round() as i32, y.round() as i32),
                    MouseScrollDelta::PixelDelta(p) => ((p.x / 40.0).round() as i32, (p.y / 40.0).round() as i32),
                };
                // Windows counts up as positive, Linux down.
                if x != 0 || y != 0 {
                    self.input.send(Input::Scroll { vertical: -y, horizontal: x });
                }
            }
            _ => {}
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        // The window goes at once. The session gets a few seconds to end by itself: killing
        // wsl.exe kills the Linux side outright, with no chance to clean up.
        self.surface = None;
        self.window = None;
        self.input.close();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if !matches!(self.child.try_wait(), Ok(None)) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let _ = self.child.kill();
    }
}

/// The pixels as the protocol's BGRA bytes.
fn as_bytes(pixels: &mut [u32]) -> &mut [u8] {
    // SAFETY: u8 has no alignment needs and every byte pattern is valid either way.
    unsafe { std::slice::from_raw_parts_mut(pixels.as_mut_ptr().cast::<u8>(), pixels.len() * 4) }
}

/// Reads the agent's events into the shared picture, and tells the window when to redraw. One
/// thread reads the pipe and another decodes, so the pipe keeps flowing while a frame is
/// decompressed.
fn read_events(stdout: std::process::ChildStdout, desktop: Arc<Mutex<Desktop>>, proxy: EventLoopProxy<Message>) {
    let (tx, rx) = std::sync::mpsc::sync_channel::<Option<Event>>(2);
    std::thread::spawn(move || {
        let mut reader = BufReader::with_capacity(1 << 20, stdout);
        loop {
            match Event::read(&mut reader) {
                Ok(Some(event)) => {
                    if tx.send(Some(event)).is_err() {
                        return;
                    }
                }
                Ok(None) | Err(_) => {
                    let _ = tx.send(None);
                    return;
                }
            }
        }
    });
    for event in rx {
        match event {
            Some(Event::Hello { version, width, height }) => {
                if version != VERSION {
                    let _ = proxy.send_event(Message::Ended(format!("the desktop agent speaks version {version}, this viewer {VERSION}")));
                    return;
                }
                let mut d = desktop.lock().unwrap();
                *d = Desktop { width, height, pixels: vec![0; (width * height) as usize] };
            }
            Some(Event::Frame { tiles }) => {
                let mut d = desktop.lock().unwrap();
                let width = d.width;
                let _ = apply_tiles(as_bytes(&mut d.pixels), width, &tiles);
                drop(d);
                let _ = proxy.send_event(Message::Frame);
            }
            Some(Event::Cursor { hot_x, hot_y, width, height, rgba }) => {
                let _ = proxy.send_event(Message::Cursor {
                    hot_x: hot_x as u16,
                    hot_y: hot_y as u16,
                    width: width as u16,
                    height: height as u16,
                    rgba,
                });
            }
            Some(Event::Ended { reason }) => {
                let _ = proxy.send_event(Message::Ended(reason));
                return;
            }
            None => {
                let _ = proxy.send_event(Message::Ended(String::new()));
                return;
            }
        }
    }
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let mut command = Command::new(&args.command[0]);
    command.args(&args.command[1..]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // no console window for wsl.exe
    }
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("could not start {}: {e}", args.command[0]);
            std::process::exit(1);
        }
    };
    let input = InputSink::new(child.stdin.take().unwrap());
    let stdout = child.stdout.take().unwrap();
    let desktop = Arc::new(Mutex::new(Desktop { width: 0, height: 0, pixels: Vec::new() }));

    let event_loop = EventLoop::<Message>::with_user_event().build().expect("event loop");
    let proxy = event_loop.create_proxy();
    {
        let desktop = desktop.clone();
        let proxy = proxy.clone();
        std::thread::spawn(move || read_events(stdout, desktop, proxy));
    }
    hook::install(input.clone());

    let mut app = App {
        args,
        window: None,
        surface: None,
        desktop,
        input,
        child,
        held: HashSet::new(),
        ctrl: false,
        alt: false,
        placement: (0.0, 0.0, 1.0),
        ended: None,
        pictured: false,
        proxy,
        hint_until: None,
    };
    let _ = event_loop.run_app(&mut app);
    let _ = app.child.kill();
    if let Some(reason) = app.ended.as_ref().filter(|r| !r.is_empty()) {
        eprintln!("{reason}");
    }
    // Exit code 3: the session ended before it showed anything (it did not start).
    if app.ended.is_some() && !app.pictured {
        std::process::exit(3);
    }
}
