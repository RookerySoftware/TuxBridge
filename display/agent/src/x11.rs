//! An X11 desktop (Xfce, Cinnamon) on a virtual X server (Xvfb). The Damage extension says which
//! part of the screen changed, and only that part is read back; XFixes hands over the pointer's
//! shape; XTest takes the input.

use std::error::Error;
use std::time::Instant;

use easywsl_protocol::{Event, Input};
use x11rb::connection::Connection;
use x11rb::protocol::damage::{self, ConnectionExt as _};
use x11rb::protocol::xfixes::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{self, ConnectionExt as _};
use x11rb::protocol::xtest::ConnectionExt as _;

use crate::sender::{Frame, Sender};

type Result<T> = std::result::Result<T, Box<dyn Error>>;


pub fn run() -> Result<()> {
    let (conn, screen) = x11rb::connect(None).map_err(|e| format!("no X display (is the desktop session running?): {e}"))?;
    let root = conn.setup().roots[screen].root;
    let (width, height) = {
        let s = &conn.setup().roots[screen];
        (s.width_in_pixels as u32, s.height_in_pixels as u32)
    };
    if conn.setup().roots[screen].root_depth != 24 {
        return Err("the X screen must have a depth of 24 bits".into());
    }
    conn.damage_query_version(1, 1)?.reply().map_err(|e| format!("the X server has no Damage extension: {e}"))?;
    conn.xfixes_query_version(5, 0)?.reply().map_err(|e| format!("the X server has no XFixes extension: {e}"))?;

    let damage = conn.generate_id()?;
    conn.damage_create(damage, root, damage::ReportLevel::NON_EMPTY)?;
    let region = conn.generate_id()?;
    conn.xfixes_create_region(region, &[])?;
    conn.xfixes_select_cursor_input(root, xfixes::CursorNotifyMask::DISPLAY_CURSOR)?;
    conn.flush()?;
    eprintln!("easywsl-agent: X display {width}x{height}");

    let sender = Sender::start();
    // The pointer is drawn by the viewer, from the shapes sent here.
    if let Some(cursor) = cursor_shape(&conn) {
        sender.cursor(cursor);
    }
    input_thread(root, sender.clone())?;

    let stride = width as usize * 4;
    let mut screen_pixels = read_area(&conn, root, 0, 0, width as u16, height as u16)?;
    sender.frame(Frame { width, height, stride, pixels: screen_pixels.clone() });

    let mut last = Instant::now();
    loop {
        let mut damaged = false;
        let mut event = Some(conn.wait_for_event()?);
        while let Some(e) = event {
            match e {
                x11rb::protocol::Event::DamageNotify(_) => damaged = true,
                x11rb::protocol::Event::Error(e) => eprintln!("easywsl-agent: X error {e:?}"),
                x11rb::protocol::Event::XfixesCursorNotify(_) => {
                    if let Some(cursor) = cursor_shape(&conn) {
                        sender.cursor(cursor);
                    }
                }
                _ => {}
            }
            event = conn.poll_for_event()?;
        }
        if !damaged {
            continue;
        }
        let wait = crate::frame_interval().saturating_sub(last.elapsed());
        if !wait.is_zero() {
            std::thread::sleep(wait);
        }
        last = Instant::now();
        // Take what changed so far (and reset it), and read back the area around it.
        conn.damage_subtract(damage, x11rb::NONE, region)?;
        let extents = conn.xfixes_fetch_region(region)?.reply()?.extents;
        if crate::debug() {
            eprintln!("easywsl-agent: damage {extents:?}");
        }
        let (x, y) = (extents.x.max(0) as u32, extents.y.max(0) as u32);
        let w = (extents.width as u32).min(width.saturating_sub(x));
        let h = (extents.height as u32).min(height.saturating_sub(y));
        if w == 0 || h == 0 {
            continue;
        }
        let area = read_area(&conn, root, x as i16, y as i16, w as u16, h as u16)?;
        let row = w as usize * 4;
        for r in 0..h as usize {
            let start = (y as usize + r) * stride + x as usize * 4;
            screen_pixels[start..start + row].copy_from_slice(&area[r * row..(r + 1) * row]);
        }
        sender.frame(Frame { width, height, stride, pixels: screen_pixels.clone() });
    }
}

/// Reads an area of the screen as BGRx rows of 4 bytes per pixel.
fn read_area(conn: &impl Connection, root: xproto::Window, x: i16, y: i16, w: u16, h: u16) -> Result<Vec<u8>> {
    let mut image = conn.get_image(xproto::ImageFormat::Z_PIXMAP, root, x, y, w, h, u32::MAX)?.reply()?;
    if image.data.len() != w as usize * h as usize * 4 {
        return Err(format!("the X server sent {} bytes for a {w}x{h} area", image.data.len()).into());
    }
    // The fourth byte is unused at depth 24, and a compositor drawing with OpenGL leaves
    // whatever alpha it had there; it would make unchanged pixels look changed.
    for p in image.data.chunks_exact_mut(4) {
        p[3] = 255;
    }
    Ok(image.data)
}

/// The pointer's current shape, as RGBA. X hands it over as premultiplied ARGB.
fn cursor_shape(conn: &impl Connection) -> Option<Event> {
    let image = conn.xfixes_get_cursor_image().ok()?.reply().ok()?;
    let mut rgba = Vec::with_capacity(image.cursor_image.len() * 4);
    for p in &image.cursor_image {
        let [b, g, r, a] = p.to_le_bytes();
        let unpremultiply = |c: u8| if a == 0 { 0 } else { ((c as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8 };
        rgba.extend_from_slice(&[unpremultiply(r), unpremultiply(g), unpremultiply(b), a]);
    }
    Some(Event::Cursor {
        hot_x: image.xhot as u32,
        hot_y: image.yhot as u32,
        width: image.width as u32,
        height: image.height as u32,
        rgba,
    })
}

/// Input from the viewer, through XTest on a connection of its own.
fn input_thread(root: xproto::Window, sender: Sender) -> Result<()> {
    let (conn, _) = x11rb::connect(None)?;
    conn.xtest_get_version(2, 2)?.reply().map_err(|e| format!("the X server has no XTEST extension: {e}"))?;
    std::thread::spawn(move || {
        let mut stdin = std::io::BufReader::new(std::io::stdin().lock());
        // The viewer closed: leave, which ends the session.
        while let Ok(Some(input)) = Input::read(&mut stdin) {
            let result = match input {
                // X key codes are the evdev codes plus 8.
                Input::Key { code, pressed } => {
                    let kind = if pressed { xproto::KEY_PRESS_EVENT } else { xproto::KEY_RELEASE_EVENT };
                    conn.xtest_fake_input(kind, (code + 8).min(255) as u8, 0, root, 0, 0, 0).map(drop)
                }
                Input::PointerMove { x, y } => {
                    conn.xtest_fake_input(xproto::MOTION_NOTIFY_EVENT, 0, 0, root, x as i16, y as i16, 0).map(drop)
                }
                Input::Button { code, pressed } => match button(code) {
                    Some(b) => {
                        let kind = if pressed { xproto::BUTTON_PRESS_EVENT } else { xproto::BUTTON_RELEASE_EVENT };
                        conn.xtest_fake_input(kind, b, 0, root, 0, 0, 0).map(drop)
                    }
                    None => Ok(()),
                },
                // X has no wheel: each step is a press and release of buttons 4 to 7.
                Input::Scroll { vertical, horizontal } => {
                    let mut r = Ok(());
                    for (steps, back, forward) in [(vertical, 4u8, 5u8), (horizontal, 6, 7)] {
                        let b = if steps < 0 { back } else { forward };
                        for _ in 0..steps.unsigned_abs() {
                            r = r
                                .and_then(|_| conn.xtest_fake_input(xproto::BUTTON_PRESS_EVENT, b, 0, root, 0, 0, 0).map(drop))
                                .and_then(|_| conn.xtest_fake_input(xproto::BUTTON_RELEASE_EVENT, b, 0, root, 0, 0, 0).map(drop));
                        }
                    }
                    r
                }
                Input::Refresh => {
                    sender.refresh();
                    Ok(())
                }
                // Xvfb's screen size is set when it starts.
                Input::Resize { .. } => Ok(()),
            };
            if let Err(e) = result.and_then(|_| conn.flush()) {
                eprintln!("easywsl-agent: input: {e}");
            }
        }
        std::process::exit(0);
    });
    Ok(())
}

/// X button numbers for evdev button codes.
fn button(code: u32) -> Option<u8> {
    match code {
        0x110 => Some(1), // left
        0x111 => Some(3), // right
        0x112 => Some(2), // middle
        0x113 => Some(8), // side (back)
        0x114 => Some(9), // extra (forward)
        _ => None,
    }
}
