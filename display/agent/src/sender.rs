//! Sending frames to the viewer. Capture hands over each new frame; a thread of its own compares
//! it with the one sent before and writes only the changed tiles. When the viewer is slower than
//! the desktop, frames in between are skipped rather than queued, so it always gets the newest.

use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};

use easywsl_protocol::{Event, VERSION, changed_tiles};

/// A frame as captured: BGRA (or BGRx) rows of `stride` bytes.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub pixels: Vec<u8>,
}

#[derive(Default)]
struct Slot {
    newest: Option<Frame>,
    /// The pointer's newest shape (an Event::Cursor), sent before any frame.
    cursor: Option<Event>,
    /// Send everything with the next frame (the viewer asked, or the size changed).
    full: bool,
}

#[derive(Clone)]
pub struct Sender(Arc<(Mutex<Slot>, Condvar)>);

impl Sender {
    /// Starts the writer thread on stdout.
    pub fn start() -> Sender {
        let sender = Sender(Arc::new((Mutex::new(Slot { full: true, ..Default::default() }), Condvar::new())));
        let shared = sender.0.clone();
        std::thread::spawn(move || write_loop(shared));
        sender
    }

    /// Hands over a new frame, replacing one not sent yet.
    pub fn frame(&self, frame: Frame) {
        let (lock, cv) = &*self.0;
        lock.lock().unwrap().newest = Some(frame);
        cv.notify_one();
    }

    /// Hands over a new pointer shape.
    pub fn cursor(&self, cursor: Event) {
        let (lock, cv) = &*self.0;
        lock.lock().unwrap().cursor = Some(cursor);
        cv.notify_one();
    }

    /// Makes the next frame a complete one.
    pub fn refresh(&self) {
        self.0.0.lock().unwrap().full = true;
    }
}

fn write_loop(shared: Arc<(Mutex<Slot>, Condvar)>) {
    // Writing a frame where most of a 4K screen changed takes longer than compressing it, so a
    // thread of its own writes while the next frame is compressed.
    let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<Event>>(1);
    std::thread::spawn(move || {
        let stdout = std::io::stdout();
        let mut out = std::io::BufWriter::with_capacity(1 << 20, stdout.lock());
        for events in rx {
            let started = std::time::Instant::now();
            for event in &events {
                if event.write(&mut out).is_err() {
                    // The viewer is gone.
                    std::process::exit(0);
                }
            }
            if out.flush().is_err() {
                std::process::exit(0);
            }
            if crate::debug() {
                eprintln!("easywsl-agent: wrote in {:?}", started.elapsed());
            }
        }
    });
    let mut previous: Option<Frame> = None;
    loop {
        let (frame, cursor, full) = {
            let (lock, cv) = &*shared;
            let mut slot = lock.lock().unwrap();
            while slot.newest.is_none() && slot.cursor.is_none() {
                slot = cv.wait(slot).unwrap();
            }
            let full = slot.newest.is_some() && std::mem::take(&mut slot.full);
            (slot.newest.take(), slot.cursor.take(), full)
        };
        let mut events = Vec::new();
        events.extend(cursor);
        if let Some(frame) = frame {
            let resized = previous.as_ref().is_none_or(|p| p.width != frame.width || p.height != frame.height || p.stride != frame.stride);
            if resized {
                events.push(Event::Hello { version: VERSION, width: frame.width, height: frame.height });
            }
            let started = std::time::Instant::now();
            let base = if full || resized { None } else { previous.as_ref().map(|p| p.pixels.as_slice()) };
            let tiles = changed_tiles(&frame.pixels, base, frame.width, frame.height, frame.stride);
            if crate::debug() {
                let bytes: usize = tiles.iter().map(|t| t.1.len()).sum();
                eprintln!("easywsl-agent: {} tiles ({} KB) compressed in {:?}", tiles.len(), bytes / 1024, started.elapsed());
            }
            if !tiles.is_empty() {
                events.push(Event::Frame { tiles });
            }
            previous = Some(frame);
        }
        if !events.is_empty() && tx.send(events).is_err() {
            return;
        }
    }
}
