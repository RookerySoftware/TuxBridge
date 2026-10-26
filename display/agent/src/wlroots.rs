//! A wlroots-style Wayland compositor (Hyprland). Frames come through wlr-screencopy, which
//! waits for the screen to change before it copies; input goes in through a virtual pointer and
//! a virtual keyboard. The compositor draws the pointer into the frames, so the viewer hides its
//! own.

use std::error::Error;
use std::os::fd::{AsFd, FromRawFd, OwnedFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use easywsl_protocol::{Event, Input};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{wl_buffer, wl_output, wl_pointer, wl_registry, wl_seat, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle, WEnum, delegate_noop};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1};
use wayland_protocols_wlr::screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1};
use wayland_protocols_wlr::virtual_pointer::v1::client::{zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1};
use xkbcommon::xkb;

use crate::sender::{Frame, Sender};

type Result<T> = std::result::Result<T, Box<dyn Error>>;


#[derive(Default)]
struct State {
    /// The buffer the compositor wants for the next copy: format, width, height, stride.
    offer: Option<(wl_shm::Format, u32, u32, u32)>,
    /// All buffer offers are in (screencopy version 3 and later).
    offers_done: bool,
    y_invert: bool,
    ready: bool,
    failed: bool,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(_: &mut Self, _: &wl_registry::WlRegistry, _: wl_registry::Event, _: &GlobalListContents, _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use zwlr_screencopy_frame_v1::Event as E;
        match event {
            E::Buffer { format: WEnum::Value(format), width, height, stride } => state.offer = Some((format, width, height, stride)),
            E::BufferDone => state.offers_done = true,
            E::Flags { flags: WEnum::Value(flags) } => {
                state.y_invert = flags.contains(zwlr_screencopy_frame_v1::Flags::YInvert)
            }
            E::Ready { .. } => state.ready = true,
            E::Failed => state.failed = true,
            _ => {}
        }
    }
}

delegate_noop!(State: ignore wl_output::WlOutput);
delegate_noop!(State: ignore wl_seat::WlSeat);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_buffer::WlBuffer);
delegate_noop!(State: wl_shm_pool::WlShmPool);
delegate_noop!(State: zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1);
delegate_noop!(State: zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1);
delegate_noop!(State: zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1);
delegate_noop!(State: zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1);
delegate_noop!(State: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1);

/// A shared-memory buffer the compositor copies the screen into.
struct ShmBuffer {
    buffer: wl_buffer::WlBuffer,
    map: *mut u8,
    len: usize,
    key: (wl_shm::Format, u32, u32, u32),
}

impl ShmBuffer {
    fn new(shm: &wl_shm::WlShm, qh: &QueueHandle<State>, key: (wl_shm::Format, u32, u32, u32)) -> Result<ShmBuffer> {
        let (format, width, height, stride) = key;
        let len = stride as usize * height as usize;
        let fd = memfd(len)?;
        let map = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, std::os::fd::AsRawFd::as_raw_fd(&fd), 0) };
        if map == libc::MAP_FAILED {
            return Err(format!("mapping the screen buffer: {}", std::io::Error::last_os_error()).into());
        }
        let pool = shm.create_pool(fd.as_fd(), len as i32, qh, ());
        let buffer = pool.create_buffer(0, width as i32, height as i32, stride as i32, format, qh, ());
        pool.destroy();
        Ok(ShmBuffer { buffer, map: map as *mut u8, len, key })
    }

    fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.map, self.len) }
    }
}

impl Drop for ShmBuffer {
    fn drop(&mut self) {
        self.buffer.destroy();
        unsafe { libc::munmap(self.map as *mut libc::c_void, self.len) };
    }
}

/// An anonymous file of `len` bytes, to share with the compositor.
fn memfd(len: usize) -> Result<OwnedFd> {
    let fd = unsafe { libc::memfd_create(c"easywsl-agent".as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(format!("memfd_create: {}", std::io::Error::last_os_error()).into());
    }
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    if unsafe { libc::ftruncate(std::os::fd::AsRawFd::as_raw_fd(&fd), len as libc::off_t) } < 0 {
        return Err(format!("ftruncate: {}", std::io::Error::last_os_error()).into());
    }
    Ok(fd)
}

pub fn run() -> Result<()> {
    let conn = Connection::connect_to_env().map_err(|e| format!("no Wayland display (is the desktop session running?): {e}"))?;
    let (globals, mut queue) = registry_queue_init::<State>(&conn)?;
    let qh = queue.handle();
    let shm: wl_shm::WlShm = globals.bind(&qh, 1..=1, ())?;
    let output: wl_output::WlOutput = globals.bind(&qh, 1..=4, ())?;
    let seat: wl_seat::WlSeat = globals.bind(&qh, 1..=7, ())?;
    let screencopy: zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1 =
        globals.bind(&qh, 1..=3, ()).map_err(|e| format!("the compositor cannot share its screen (wlr-screencopy): {e}"))?;
    let pointers: zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1 =
        globals.bind(&qh, 1..=2, ()).map_err(|e| format!("the compositor takes no virtual pointer: {e}"))?;
    let keyboards: zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1 =
        globals.bind(&qh, 1..=1, ()).map_err(|e| format!("the compositor takes no virtual keyboard: {e}"))?;
    let pointer = pointers.create_virtual_pointer(Some(&seat), &qh, ());
    let keyboard = keyboards.create_virtual_keyboard(&seat, &qh, ());

    let sender = Sender::start();
    // The compositor draws the pointer into the frames.
    sender.cursor(Event::Cursor { hot_x: 0, hot_y: 0, width: 0, height: 0, rgba: Vec::new() });

    let size = Arc::new((AtomicU32::new(0), AtomicU32::new(0)));
    let input = Keyboard::new()?;
    input.send_keymap(&keyboard)?;
    conn.flush()?;
    input_thread(conn.clone(), pointer, keyboard, input, size.clone(), sender.clone());

    let mut buffer: Option<ShmBuffer> = None;
    let mut first = true;
    let mut last = Instant::now();
    loop {
        let mut state = State::default();
        let frame = screencopy.capture_output(1, &output, &qh, ());
        // Version 3 lists every buffer kind and ends with BufferDone; older ones offer one shm buffer.
        while state.offer.is_none() || (frame.version() >= 3 && !state.offers_done) {
            if state.failed {
                break;
            }
            queue.blocking_dispatch(&mut state)?;
        }
        let Some(key) = state.offer.filter(|_| !state.failed) else {
            frame.destroy();
            return Err("the compositor could not share its screen".into());
        };
        if buffer.as_ref().is_none_or(|b| b.key != key) {
            buffer = Some(ShmBuffer::new(&shm, &qh, key)?);
            first = true;
        }
        let shm_buffer = buffer.as_ref().unwrap();
        let t_copy = Instant::now();
        // After the first copy, wait for the screen to change.
        if first || frame.version() < 2 {
            frame.copy(&shm_buffer.buffer);
        } else {
            frame.copy_with_damage(&shm_buffer.buffer);
        }
        first = false;
        while !state.ready && !state.failed {
            queue.blocking_dispatch(&mut state)?;
        }
        frame.destroy();
        if state.failed {
            continue;
        }

        let t_ready = Instant::now();
        let (format, width, height, stride) = key;
        size.0.store(width, Ordering::Relaxed);
        size.1.store(height, Ordering::Relaxed);
        let mut pixels = shm_buffer.bytes().to_vec();
        if state.y_invert {
            let row = stride as usize;
            let rows: Vec<Vec<u8>> = pixels.chunks_exact(row).rev().map(<[u8]>::to_vec).collect();
            pixels = rows.concat();
        }
        // The viewer takes BGRA, which is what (A|X)RGB8888 is in memory; swap the others.
        if matches!(format, wl_shm::Format::Abgr8888 | wl_shm::Format::Xbgr8888) {
            for p in pixels.chunks_exact_mut(4) {
                p.swap(0, 2);
            }
        }
        sender.frame(Frame { width, height, stride: stride as usize, pixels });
        if crate::debug() {
            eprintln!("easywsl-agent: copy {:?}, process {:?}", t_ready - t_copy, t_ready.elapsed());
        }

        let wait = crate::frame_interval().saturating_sub(last.elapsed());
        if !wait.is_zero() {
            std::thread::sleep(wait);
        }
        last = Instant::now();
    }
}

/// The keyboard layout given to the compositor, and the state of the modifier keys in it.
struct Keyboard {
    keymap: String,
    state: xkb::State,
}

// The xkb state is used by the input thread only.
unsafe impl Send for Keyboard {}

impl Keyboard {
    fn new() -> Result<Keyboard> {
        let layout = std::env::var("EASYWSL_KEYBOARD_LAYOUT").unwrap_or_else(|_| "us".into());
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(&context, "", "", layout.as_str(), "", None, xkb::KEYMAP_COMPILE_NO_FLAGS)
            .ok_or_else(|| format!("no keyboard layout '{layout}'"))?;
        Ok(Keyboard { keymap: keymap.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1), state: xkb::State::new(&keymap) })
    }

    fn send_keymap(&self, keyboard: &zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1) -> Result<()> {
        let mut text = self.keymap.clone().into_bytes();
        text.push(0);
        let fd = memfd(text.len())?;
        let mut file = std::fs::File::from(fd);
        std::io::Write::write_all(&mut file, &text)?;
        keyboard.keymap(xkb::KEYMAP_FORMAT_TEXT_V1, file.as_fd(), text.len() as u32);
        Ok(())
    }

    /// Updates the modifier state with a key and returns it, when it changed.
    fn key(&mut self, code: u32, pressed: bool) -> Option<(u32, u32, u32, u32)> {
        let direction = if pressed { xkb::KeyDirection::Down } else { xkb::KeyDirection::Up };
        let changed = self.state.update_key(xkb::Keycode::new(code + 8), direction);
        (changed != 0).then(|| {
            (
                self.state.serialize_mods(xkb::STATE_MODS_DEPRESSED),
                self.state.serialize_mods(xkb::STATE_MODS_LATCHED),
                self.state.serialize_mods(xkb::STATE_MODS_LOCKED),
                self.state.serialize_layout(xkb::STATE_LAYOUT_EFFECTIVE),
            )
        })
    }
}

fn input_thread(
    conn: Connection,
    pointer: zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
    keyboard: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
    mut keys: Keyboard,
    size: Arc<(AtomicU32, AtomicU32)>,
    sender: Sender,
) {
    std::thread::spawn(move || {
        let start = Instant::now();
        let mut stdin = std::io::BufReader::new(std::io::stdin().lock());
        // The viewer closed: leave, which ends the session.
        while let Ok(Some(input)) = Input::read(&mut stdin) {
            let time = start.elapsed().as_millis() as u32;
            match input {
                Input::Key { code, pressed } => {
                    keyboard.key(time, code, pressed as u32);
                    if let Some((depressed, latched, locked, group)) = keys.key(code, pressed) {
                        keyboard.modifiers(depressed, latched, locked, group);
                    }
                }
                Input::PointerMove { x, y } => {
                    let (w, h) = (size.0.load(Ordering::Relaxed), size.1.load(Ordering::Relaxed));
                    if w > 0 && h > 0 {
                        pointer.motion_absolute(time, (x.max(0.0) as u32).min(w - 1), (y.max(0.0) as u32).min(h - 1), w, h);
                        pointer.frame();
                    }
                }
                Input::Button { code, pressed } => {
                    let state = if pressed { wl_pointer::ButtonState::Pressed } else { wl_pointer::ButtonState::Released };
                    pointer.button(time, code, state);
                    pointer.frame();
                }
                Input::Scroll { vertical, horizontal } => {
                    pointer.axis_source(wl_pointer::AxisSource::Wheel);
                    if vertical != 0 {
                        pointer.axis_discrete(time, wl_pointer::Axis::VerticalScroll, 15.0 * vertical as f64, vertical);
                    }
                    if horizontal != 0 {
                        pointer.axis_discrete(time, wl_pointer::Axis::HorizontalScroll, 15.0 * horizontal as f64, horizontal);
                    }
                    pointer.frame();
                }
                Input::Refresh => sender.refresh(),
                // The launcher sets the screen size when the session starts.
                Input::Resize { .. } => {}
            }
            if let Err(e) = conn.flush() {
                eprintln!("easywsl-agent: input: {e}");
                break;
            }
        }
        std::process::exit(0);
    });
}
