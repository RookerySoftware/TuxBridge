//! A scripted, windowless client for testing: starts the agent command, waits for the first
//! picture, plays a script of input, and saves the picture as a BMP.
//!
//!   probe OUT.bmp "SCRIPT" -- COMMAND [ARGS...]
//!
//! SCRIPT is space-separated steps: `wait:MS`, `move:X,Y`, `click`, `key:CODE` (press and release
//! an evdev code), `type:TEXT` (lower-case letters, digits and spaces).

use std::io::{BufReader, BufWriter, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use easywsl_protocol::{Event, Input, apply_tile};

fn letter(c: char) -> Option<u32> {
    let row = |s: &str, start: u32| s.find(c).map(|i| start + i as u32);
    row("qwertyuiop", 16).or_else(|| row("asdfghjkl", 30)).or_else(|| row("zxcvbnm", 44)).or_else(|| match c {
        '1'..='9' => Some(2 + c as u32 - '1' as u32),
        '0' => Some(11),
        ' ' => Some(57),
        _ => None,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sep = args.iter().position(|a| a == "--").expect("probe OUT.bmp SCRIPT -- COMMAND");
    let (out, script) = (args[1].clone(), args[2].clone());
    let mut child = Command::new(&args[sep + 1])
        .args(&args[sep + 2..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start");
    let mut input = BufWriter::new(child.stdin.take().unwrap());
    let stdout = child.stdout.take().unwrap();
    let picture: Arc<Mutex<(u32, u32, Vec<u8>, u32)>> = Arc::new(Mutex::new((0, 0, Vec::new(), 0)));
    let received = Arc::new(std::sync::atomic::AtomicU64::new(0));
    {
        let picture = picture.clone();
        let received = received.clone();
        std::thread::spawn(move || {
            let mut r = BufReader::new(stdout);
            while let Ok(Some(event)) = Event::read(&mut r) {
                let mut p = picture.lock().unwrap();
                match event {
                    Event::Hello { width, height, .. } => *p = (width, height, vec![0; (width * height * 4) as usize], 0),
                    Event::Frame { tiles } => {
                        let bytes: usize = tiles.iter().map(|(_, d)| d.len() + 20).sum();
                        received.fetch_add(bytes as u64, std::sync::atomic::Ordering::Relaxed);
                        let w = p.0;
                        for (rect, data) in &tiles {
                            let _ = apply_tile(&mut p.2, w, rect, data);
                        }
                        p.3 += 1;
                    }
                    Event::Ended { .. } => break,
                }
            }
        });
    }
    for _ in 0..600 {
        if picture.lock().unwrap().3 > 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let start = std::time::Instant::now();
    let mut send = |i: Input| {
        i.write(&mut input).unwrap();
        input.flush().unwrap();
    };
    for step in script.split_whitespace() {
        let (op, arg) = step.split_once(':').unwrap_or((step, ""));
        match op {
            "wait" => std::thread::sleep(Duration::from_millis(arg.parse().unwrap())),
            "move" => {
                let (x, y) = arg.split_once(',').unwrap();
                send(Input::PointerMove { x: x.parse().unwrap(), y: y.parse().unwrap() });
            }
            "click" => {
                send(Input::Button { code: 0x110, pressed: true });
                send(Input::Button { code: 0x110, pressed: false });
            }
            "down" | "up" => send(Input::Key { code: arg.parse().unwrap(), pressed: op == "down" }),
            "fps" => {
                let secs: f64 = arg.parse().unwrap();
                let (f0, b0) = (picture.lock().unwrap().3, received.load(std::sync::atomic::Ordering::Relaxed));
                std::thread::sleep(Duration::from_secs_f64(secs));
                let (f1, b1) = (picture.lock().unwrap().3, received.load(std::sync::atomic::Ordering::Relaxed));
                println!("{:.1} updates/s, {:.2} MB/s", (f1 - f0) as f64 / secs, (b1 - b0) as f64 / 1e6 / secs);
            }
            // Moves the pointer back and forth for SECONDS at 120 moves a second, measuring updates.
            "sweep" => {
                let secs: f64 = arg.parse().unwrap();
                let (f0, b0) = (picture.lock().unwrap().3, received.load(std::sync::atomic::Ordering::Relaxed));
                let steps = (secs * 120.0) as u32;
                for i in 0..steps {
                    let t = (i % 240) as f64 / 240.0;
                    let x = 200.0 + 1200.0 * if t < 0.5 { t * 2.0 } else { 2.0 - t * 2.0 };
                    send(Input::PointerMove { x, y: 450.0 });
                    std::thread::sleep(Duration::from_micros(8333));
                }
                let (f1, b1) = (picture.lock().unwrap().3, received.load(std::sync::atomic::Ordering::Relaxed));
                println!("pointer sweep: {:.1} updates/s, {:.2} MB/s", (f1 - f0) as f64 / secs, (b1 - b0) as f64 / 1e6 / secs);
            }
            "key" => {
                let code = arg.parse().unwrap();
                send(Input::Key { code, pressed: true });
                send(Input::Key { code, pressed: false });
            }
            "type" => {
                for c in arg.chars().map(|c| if c == '_' { ' ' } else { c }) {
                    if let Some(code) = letter(c) {
                        send(Input::Key { code, pressed: true });
                        send(Input::Key { code, pressed: false });
                        std::thread::sleep(Duration::from_millis(30));
                    }
                }
            }
            _ => panic!("unknown step {step}"),
        }
    }
    let (w, h, pixels, frames) = picture.lock().unwrap().clone();
    println!("{w}x{h}, {frames} updates, script took {:?}", start.elapsed());
    // A 32-bit BMP, bottom-up rows: BGRA is exactly its pixel order.
    let mut bmp = Vec::with_capacity(54 + pixels.len());
    let size = 54 + pixels.len() as u32;
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&size.to_le_bytes());
    bmp.extend_from_slice(&[0, 0, 0, 0, 54, 0, 0, 0, 40, 0, 0, 0]);
    bmp.extend_from_slice(&(w as i32).to_le_bytes());
    bmp.extend_from_slice(&(h as i32).to_le_bytes());
    bmp.extend_from_slice(&[1, 0, 32, 0, 0, 0, 0, 0]);
    bmp.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
    bmp.extend_from_slice(&[0; 16]);
    for row in pixels.chunks_exact(w as usize * 4).rev() {
        bmp.extend_from_slice(row);
    }
    std::fs::write(&out, bmp).unwrap();
    let _ = child.kill();
}
