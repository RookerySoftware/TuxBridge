//! The messages between the EasyWSL display agent, which runs inside Linux next to the desktop,
//! and the viewer, a window on Windows. They travel over the agent's stdin and stdout, which
//! wsl.exe connects to the viewer through WSL's own Hyper-V socket: no network, no ports.
//!
//! Every message is a 1-byte type, a 4-byte little-endian payload length, then the payload.
//! Pixels are BGRA, 4 bytes each, rows top to bottom; only changed tiles are sent, each
//! compressed with LZ4.

use std::io::{self, Read, Write};

/// Side of the square tiles the agent compares frames in.
pub const TILE: u32 = 64;

/// Bumped when a message changes shape; the viewer refuses an agent with another version.
pub const VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Agent to viewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// First message, and again whenever the desktop's size changes. The next Frame repaints
    /// everything.
    Hello { version: u32, width: u32, height: u32 },
    /// Changed areas of the desktop: each tile's BGRA pixels, LZ4-compressed.
    Frame { tiles: Vec<(Rect, Vec<u8>)> },
    /// The desktop session ended (logged out, powered off).
    Ended { reason: String },
    /// The pointer's shape changed. The viewer shows it as its own pointer, so moving the mouse
    /// needs no round trip. RGBA, `width` x `height`; zero size hides the pointer.
    Cursor { hot_x: u32, hot_y: u32, width: u32, height: u32, rgba: Vec<u8> },
}

/// Viewer to agent. Keys are Linux evdev key codes (KEY_A = 30, ...); buttons are evdev button
/// codes (BTN_LEFT = 0x110, BTN_RIGHT = 0x111, BTN_MIDDLE = 0x112).
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Key { code: u32, pressed: bool },
    PointerMove { x: f64, y: f64 },
    Button { code: u32, pressed: bool },
    /// Wheel steps: positive is down / right.
    Scroll { vertical: i32, horizontal: i32 },
    /// The viewer wants a different desktop size (its window was resized).
    Resize { width: u32, height: u32 },
    /// Repaint everything (after the viewer lost its picture).
    Refresh,
}

mod kind {
    pub const HELLO: u8 = 1;
    pub const FRAME: u8 = 2;
    pub const ENDED: u8 = 3;
    pub const CURSOR: u8 = 4;
    pub const KEY: u8 = 16;
    pub const POINTER_MOVE: u8 = 17;
    pub const BUTTON: u8 = 18;
    pub const SCROLL: u8 = 19;
    pub const RESIZE: u8 = 20;
    pub const REFRESH: u8 = 21;
}

struct Buf(Vec<u8>);

impl Buf {
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f64(&mut self, v: f64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.u32(v.len() as u32);
        self.0.extend_from_slice(v);
    }
}

struct Cursor<'a>(&'a [u8]);

impl Cursor<'_> {
    fn take(&mut self, n: usize) -> io::Result<&[u8]> {
        if self.0.len() < n {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "message too short"));
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f64(&mut self) -> io::Result<f64> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn bytes(&mut self) -> io::Result<Vec<u8>> {
        let n = self.u32()? as usize;
        Ok(self.take(n)?.to_vec())
    }
}

fn write_message(w: &mut impl Write, kind: u8, payload: &[u8]) -> io::Result<()> {
    let mut head = [0u8; 5];
    head[0] = kind;
    head[1..].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    w.write_all(&head)?;
    w.write_all(payload)
}

/// Reads one message: its type and payload. None at the end of the stream.
fn read_message(r: &mut impl Read) -> io::Result<Option<(u8, Vec<u8>)>> {
    let mut head = [0u8; 5];
    match r.read_exact(&mut head) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_le_bytes(head[1..].try_into().unwrap()) as usize;
    if len > 512 << 20 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
    }
    let mut payload = vec![0u8; len];
    r.read_exact(&mut payload)?;
    Ok(Some((head[0], payload)))
}

impl Event {
    pub fn write(&self, w: &mut impl Write) -> io::Result<()> {
        let mut b = Buf(Vec::new());
        let kind = match self {
            Event::Hello { version, width, height } => {
                b.u32(*version);
                b.u32(*width);
                b.u32(*height);
                kind::HELLO
            }
            Event::Frame { tiles } => {
                b.u32(tiles.len() as u32);
                for (r, data) in tiles {
                    b.u32(r.x);
                    b.u32(r.y);
                    b.u32(r.width);
                    b.u32(r.height);
                    b.bytes(data);
                }
                kind::FRAME
            }
            Event::Ended { reason } => {
                b.bytes(reason.as_bytes());
                kind::ENDED
            }
            Event::Cursor { hot_x, hot_y, width, height, rgba } => {
                b.u32(*hot_x);
                b.u32(*hot_y);
                b.u32(*width);
                b.u32(*height);
                b.bytes(rgba);
                kind::CURSOR
            }
        };
        write_message(w, kind, &b.0)
    }

    pub fn read(r: &mut impl Read) -> io::Result<Option<Event>> {
        let Some((kind, payload)) = read_message(r)? else { return Ok(None) };
        let mut c = Cursor(&payload);
        Ok(Some(match kind {
            kind::HELLO => Event::Hello { version: c.u32()?, width: c.u32()?, height: c.u32()? },
            kind::FRAME => {
                let n = c.u32()? as usize;
                let mut tiles = Vec::with_capacity(n);
                for _ in 0..n {
                    let rect = Rect { x: c.u32()?, y: c.u32()?, width: c.u32()?, height: c.u32()? };
                    tiles.push((rect, c.bytes()?));
                }
                Event::Frame { tiles }
            }
            kind::ENDED => Event::Ended { reason: String::from_utf8_lossy(&c.bytes()?).into_owned() },
            kind::CURSOR => Event::Cursor { hot_x: c.u32()?, hot_y: c.u32()?, width: c.u32()?, height: c.u32()?, rgba: c.bytes()? },
            other => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown event {other}"))),
        }))
    }
}

impl Input {
    pub fn write(&self, w: &mut impl Write) -> io::Result<()> {
        let mut b = Buf(Vec::new());
        let kind = match self {
            Input::Key { code, pressed } => {
                b.u32(*code);
                b.u32(*pressed as u32);
                kind::KEY
            }
            Input::PointerMove { x, y } => {
                b.f64(*x);
                b.f64(*y);
                kind::POINTER_MOVE
            }
            Input::Button { code, pressed } => {
                b.u32(*code);
                b.u32(*pressed as u32);
                kind::BUTTON
            }
            Input::Scroll { vertical, horizontal } => {
                b.i32(*vertical);
                b.i32(*horizontal);
                kind::SCROLL
            }
            Input::Resize { width, height } => {
                b.u32(*width);
                b.u32(*height);
                kind::RESIZE
            }
            Input::Refresh => kind::REFRESH,
        };
        write_message(w, kind, &b.0)
    }

    pub fn read(r: &mut impl Read) -> io::Result<Option<Input>> {
        let Some((kind, payload)) = read_message(r)? else { return Ok(None) };
        let mut c = Cursor(&payload);
        Ok(Some(match kind {
            kind::KEY => Input::Key { code: c.u32()?, pressed: c.u32()? != 0 },
            kind::POINTER_MOVE => Input::PointerMove { x: c.f64()?, y: c.f64()? },
            kind::BUTTON => Input::Button { code: c.u32()?, pressed: c.u32()? != 0 },
            kind::SCROLL => Input::Scroll { vertical: c.i32()?, horizontal: c.i32()? },
            kind::RESIZE => Input::Resize { width: c.u32()?, height: c.u32()? },
            kind::REFRESH => Input::Refresh,
            other => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown input {other}"))),
        }))
    }
}

/// Splits a frame into tiles and returns the ones that differ from `previous` (all of them when
/// there is none), LZ4-compressed. `frame` and `previous` are BGRA with `stride` bytes per row.
pub fn changed_tiles(frame: &[u8], previous: Option<&[u8]>, width: u32, height: u32, stride: usize) -> Vec<(Rect, Vec<u8>)> {
    let mut tiles = Vec::new();
    let mut pixels = Vec::new();
    for ty in (0..height).step_by(TILE as usize) {
        for tx in (0..width).step_by(TILE as usize) {
            let w = TILE.min(width - tx);
            let h = TILE.min(height - ty);
            let row_bytes = w as usize * 4;
            let changed = match previous {
                None => true,
                Some(prev) => (ty..ty + h).any(|y| {
                    let start = y as usize * stride + tx as usize * 4;
                    frame[start..start + row_bytes] != prev[start..start + row_bytes]
                }),
            };
            if !changed {
                continue;
            }
            pixels.clear();
            for y in ty..ty + h {
                let start = y as usize * stride + tx as usize * 4;
                pixels.extend_from_slice(&frame[start..start + row_bytes]);
            }
            tiles.push((Rect { x: tx, y: ty, width: w, height: h }, lz4_flex::compress_prepend_size(&pixels)));
        }
    }
    tiles
}

/// Writes a received tile into a BGRA framebuffer of `width` pixels per row.
pub fn apply_tile(framebuffer: &mut [u8], width: u32, rect: &Rect, data: &[u8]) -> io::Result<()> {
    let pixels = lz4_flex::decompress_size_prepended(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let row_bytes = rect.width as usize * 4;
    if pixels.len() != row_bytes * rect.height as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "tile size mismatch"));
    }
    for row in 0..rect.height as usize {
        let dst = ((rect.y as usize + row) * width as usize + rect.x as usize) * 4;
        let Some(target) = framebuffer.get_mut(dst..dst + row_bytes) else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "tile outside the framebuffer"));
        };
        target.copy_from_slice(&pixels[row * row_bytes..(row + 1) * row_bytes]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_round_trip() {
        let events = vec![
            Event::Hello { version: VERSION, width: 1600, height: 900 },
            Event::Frame { tiles: vec![(Rect { x: 0, y: 64, width: 10, height: 2 }, vec![1, 2, 3])] },
            Event::Ended { reason: "logged out".into() },
            Event::Cursor { hot_x: 1, hot_y: 2, width: 1, height: 1, rgba: vec![1, 2, 3, 4] },
        ];
        let mut buf = Vec::new();
        for e in &events {
            e.write(&mut buf).unwrap();
        }
        let mut r = &buf[..];
        for e in &events {
            assert_eq!(&Event::read(&mut r).unwrap().unwrap(), e);
        }
        assert!(Event::read(&mut r).unwrap().is_none());

        let inputs = vec![
            Input::Key { code: 30, pressed: true },
            Input::PointerMove { x: 10.5, y: 20.0 },
            Input::Button { code: 0x110, pressed: false },
            Input::Scroll { vertical: -1, horizontal: 0 },
            Input::Resize { width: 1920, height: 1080 },
            Input::Refresh,
        ];
        let mut buf = Vec::new();
        for i in &inputs {
            i.write(&mut buf).unwrap();
        }
        let mut r = &buf[..];
        for i in &inputs {
            assert_eq!(&Input::read(&mut r).unwrap().unwrap(), i);
        }
    }

    #[test]
    fn only_changed_tiles_are_sent_and_they_apply() {
        let (w, h) = (130u32, 70u32);
        let stride = w as usize * 4;
        let a = vec![7u8; stride * h as usize];
        let mut b = a.clone();
        b[stride * 66 + 4 * 129] = 9; // one pixel in the bottom-right tile
        assert_eq!(changed_tiles(&a, None, w, h, stride).len(), 3 * 2);
        let tiles = changed_tiles(&b, Some(&a), w, h, stride);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].0, Rect { x: 128, y: 64, width: 2, height: 6 });
        let mut fb = a.clone();
        apply_tile(&mut fb, w, &tiles[0].0, &tiles[0].1).unwrap();
        assert_eq!(fb, b);
    }
}
