//! A GNOME Shell session, through Mutter's own remote-desktop interfaces (the ones GNOME's
//! remote desktop server uses): a RemoteDesktop session for input, a ScreenCast session tied to
//! it for frames, which arrive as a PipeWire video stream.

use std::collections::HashMap;
use std::error::Error;

use easywsl_protocol::{Event, Input};
use pipewire as pw;
use pw::spa;
use pw::spa::pod::Pod;
use zbus::blocking::{Connection, MessageIterator};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::sender::{Frame, Sender};

const RD: &str = "org.gnome.Mutter.RemoteDesktop";
const RD_SESSION: &str = "org.gnome.Mutter.RemoteDesktop.Session";
const SC: &str = "org.gnome.Mutter.ScreenCast";
const SC_SESSION: &str = "org.gnome.Mutter.ScreenCast.Session";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn run(connector: &str) -> Result<()> {
    let conn = Connection::session().map_err(|e| format!("no session bus (is the desktop session running?): {e}"))?;

    // The RemoteDesktop session carries input; the ScreenCast session tied to it, the picture.
    let rd_session: OwnedObjectPath = conn
        .call_method(Some(RD), "/org/gnome/Mutter/RemoteDesktop", Some(RD), "CreateSession", &())
        .map_err(|e| format!("GNOME's remote desktop interface is not available: {e}"))?
        .body()
        .deserialize()?;
    let id: OwnedValue = conn
        .call_method(Some(RD), &rd_session, Some("org.freedesktop.DBus.Properties"), "Get", &(RD_SESSION, "SessionId"))?
        .body()
        .deserialize()?;
    let id = String::try_from(id)?;

    let mut options: HashMap<&str, Value> = HashMap::new();
    options.insert("remote-desktop-session-id", Value::from(id.as_str()));
    let sc_session: OwnedObjectPath = conn
        .call_method(Some(SC), "/org/gnome/Mutter/ScreenCast", Some(SC), "CreateSession", &(options,))?
        .body()
        .deserialize()?;
    // cursor-mode 2: the pointer comes as metadata, not drawn into the frames. The viewer shows
    // it as its own pointer, so moving the mouse needs no round trip and changes no pixels.
    let mut options: HashMap<&str, Value> = HashMap::new();
    options.insert("cursor-mode", Value::U32(2));
    let stream_path: OwnedObjectPath = conn
        .call_method(Some(SC), &sc_session, Some(SC_SESSION), "RecordMonitor", &(connector, options))
        .map_err(|e| format!("recording monitor {connector}: {e}"))?
        .body()
        .deserialize()?;

    let rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .interface("org.gnome.Mutter.ScreenCast.Stream")?
        .member("PipeWireStreamAdded")?
        .path(stream_path.as_str())?
        .build();
    let mut added = MessageIterator::for_match_rule(rule, &conn, Some(4))?;
    let closed_rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .interface(RD_SESSION)?
        .member("Closed")?
        .path(rd_session.as_str())?
        .build();
    let closed = MessageIterator::for_match_rule(closed_rule, &conn, Some(4))?;

    conn.call_method(Some(RD), &rd_session, Some(RD_SESSION), "Start", &())?;
    let node: u32 = match added.next() {
        Some(message) => message?.body().deserialize()?,
        None => return Err("GNOME did not start the screen stream".into()),
    };
    eprintln!("easywsl-agent: GNOME stream {stream_path} on PipeWire node {node}");

    let sender = Sender::start();

    // The session ends (logout, power off): the viewer sees the end of the stream and closes.
    std::thread::spawn(move || {
        let mut closed = closed;
        let _ = closed.next();
        eprintln!("easywsl-agent: the GNOME session closed");
        std::process::exit(0);
    });

    // Input from the viewer, into the RemoteDesktop session.
    {
        let conn = conn.clone();
        let rd_session = rd_session.clone();
        let stream = stream_path.to_string();
        let sender = sender.clone();
        std::thread::spawn(move || {
            let mut stdin = std::io::BufReader::new(std::io::stdin().lock());
            loop {
                let input = match Input::read(&mut stdin) {
                    Ok(Some(input)) => input,
                    // The viewer closed: end the session and leave.
                    _ => {
                        let _ = conn.call_method(Some(RD), &rd_session, Some(RD_SESSION), "Stop", &());
                        std::process::exit(0);
                    }
                };
                let result = match input {
                    Input::Key { code, pressed } => {
                        conn.call_method(Some(RD), &rd_session, Some(RD_SESSION), "NotifyKeyboardKeycode", &(code, pressed)).map(drop)
                    }
                    Input::PointerMove { x, y } => conn
                        .call_method(Some(RD), &rd_session, Some(RD_SESSION), "NotifyPointerMotionAbsolute", &(stream.as_str(), x, y))
                        .map(drop),
                    Input::Button { code, pressed } => conn
                        .call_method(Some(RD), &rd_session, Some(RD_SESSION), "NotifyPointerButton", &(code as i32, pressed))
                        .map(drop),
                    Input::Scroll { vertical, horizontal } => {
                        let mut r = Ok(());
                        if vertical != 0 {
                            r = conn.call_method(Some(RD), &rd_session, Some(RD_SESSION), "NotifyPointerAxisDiscrete", &(0u32, vertical)).map(drop);
                        }
                        if horizontal != 0 {
                            r = conn.call_method(Some(RD), &rd_session, Some(RD_SESSION), "NotifyPointerAxisDiscrete", &(1u32, horizontal)).map(drop);
                        }
                        r
                    }
                    Input::Refresh => {
                        sender.refresh();
                        Ok(())
                    }
                    // The virtual monitor's size is set when the session starts.
                    Input::Resize { .. } => Ok(()),
                };
                if let Err(e) = result {
                    eprintln!("easywsl-agent: input: {e}");
                }
            }
        });
    }

    stream_frames(node, sender)
}

struct StreamData {
    format: spa::param::video::VideoInfoRaw,
    sender: Sender,
    /// The buffer and metadata (pointer) requests, sent once the format is settled.
    params: Vec<Vec<u8>>,
}

/// Room for the pointer's metadata: the cursor and bitmap headers, and up to 384x384 pixels
/// (what GNOME offers).
const CURSOR_META_SIZE: usize =
    std::mem::size_of::<spa::sys::spa_meta_cursor>() + std::mem::size_of::<spa::sys::spa_meta_bitmap>() + 384 * 384 * 4;

const fn cursor_meta_size(side: usize) -> usize {
    std::mem::size_of::<spa::sys::spa_meta_cursor>() + std::mem::size_of::<spa::sys::spa_meta_bitmap>() + side * side * 4
}

/// The pointer's shape from a buffer's cursor metadata, as RGBA, when it carries a new one.
unsafe fn cursor_shape(buffer: *mut spa::sys::spa_buffer) -> Option<Event> {
    use spa::sys::*;
    unsafe {
        for i in 0..(*buffer).n_metas as usize {
            let meta = &*(*buffer).metas.add(i);
            if meta.type_ != SPA_META_Cursor || meta.data.is_null() || (meta.size as usize) < std::mem::size_of::<spa_meta_cursor>() {
                continue;
            }
            let cursor = &*(meta.data as *const spa_meta_cursor);
            // id 0: no cursor information; bitmap_offset 0: no new shape (only a position).
            if cursor.id == 0 || (cursor.bitmap_offset as usize) < std::mem::size_of::<spa_meta_cursor>() {
                return None;
            }
            let bitmap_ptr = (meta.data as *const u8).add(cursor.bitmap_offset as usize);
            let bitmap = &*(bitmap_ptr as *const spa_meta_bitmap);
            if bitmap.format == 0 {
                return None;
            }
            let (w, h) = (bitmap.size.width, bitmap.size.height);
            if bitmap.offset == 0 || w == 0 || h == 0 {
                return Some(Event::Cursor { hot_x: 0, hot_y: 0, width: 0, height: 0, rgba: Vec::new() });
            }
            let stride = if bitmap.stride > 0 { bitmap.stride as usize } else { w as usize * 4 };
            let pixels = bitmap_ptr.add(bitmap.offset as usize);
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for y in 0..h as usize {
                let row = std::slice::from_raw_parts(pixels.add(y * stride), w as usize * 4);
                for p in row.chunks_exact(4) {
                    let (r, g, b, a) = match bitmap.format {
                        SPA_VIDEO_FORMAT_RGBA => (p[0], p[1], p[2], p[3]),
                        SPA_VIDEO_FORMAT_BGRA => (p[2], p[1], p[0], p[3]),
                        SPA_VIDEO_FORMAT_ARGB => (p[1], p[2], p[3], p[0]),
                        SPA_VIDEO_FORMAT_ABGR => (p[3], p[2], p[1], p[0]),
                        _ => return None,
                    };
                    rgba.extend_from_slice(&[r, g, b, a]);
                }
            }
            return Some(Event::Cursor {
                hot_x: cursor.hotspot.x.max(0) as u32,
                hot_y: cursor.hotspot.y.max(0) as u32,
                width: w,
                height: h,
                rgba,
            });
        }
        None
    }
}

/// Receives the PipeWire video stream and hands each frame and pointer shape to the sender.
fn stream_frames(node: u32, sender: Sender) -> Result<()> {
    pw::init();
    let mainloop = pw::main_loop::MainLoop::new(None)?;
    let context = pw::context::Context::new(&mainloop)?;
    let core = context.connect(None)?;
    let stream = pw::stream::Stream::new(
        &core,
        "easywsl-agent",
        pw::properties::properties! {
            *pw::keys::MEDIA_TYPE => "Video",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Screen",
        },
    )?;

    let meta = spa::pod::Value::Object(spa::pod::Object {
        type_: spa::sys::SPA_TYPE_OBJECT_ParamMeta,
        id: spa::sys::SPA_PARAM_Meta,
        properties: vec![
            spa::pod::Property {
                key: spa::sys::SPA_PARAM_META_type,
                flags: spa::pod::PropertyFlags::empty(),
                value: spa::pod::Value::Id(spa::utils::Id(spa::sys::SPA_META_Cursor)),
            },
            spa::pod::Property {
                key: spa::sys::SPA_PARAM_META_size,
                flags: spa::pod::PropertyFlags::empty(),
                // A range, as GNOME's own remote desktop server asks for it: up to 384x384.
                value: spa::pod::Value::Choice(spa::pod::ChoiceValue::Int(spa::utils::Choice(
                    spa::utils::ChoiceFlags::empty(),
                    spa::utils::ChoiceEnum::Range {
                        default: CURSOR_META_SIZE as i32,
                        min: cursor_meta_size(1) as i32,
                        max: CURSOR_META_SIZE as i32,
                    },
                ))),
            },
        ],
    });
    // Buffers in shared memory the agent can read directly (not GPU buffers).
    let buffers = spa::pod::Value::Object(spa::pod::Object {
        type_: spa::sys::SPA_TYPE_OBJECT_ParamBuffers,
        id: spa::sys::SPA_PARAM_Buffers,
        properties: vec![spa::pod::Property {
            key: spa::sys::SPA_PARAM_BUFFERS_dataType,
            flags: spa::pod::PropertyFlags::empty(),
            value: spa::pod::Value::Int((1 << spa::sys::SPA_DATA_MemFd | 1 << spa::sys::SPA_DATA_MemPtr) as i32),
        }],
    });
    let serialize = |v: &spa::pod::Value| -> Result<Vec<u8>> {
        Ok(spa::pod::serialize::PodSerializer::serialize(std::io::Cursor::new(Vec::new()), v)?.0.into_inner())
    };
    let header = spa::pod::Value::Object(spa::pod::Object {
        type_: spa::sys::SPA_TYPE_OBJECT_ParamMeta,
        id: spa::sys::SPA_PARAM_Meta,
        properties: vec![
            spa::pod::Property {
                key: spa::sys::SPA_PARAM_META_type,
                flags: spa::pod::PropertyFlags::empty(),
                value: spa::pod::Value::Id(spa::utils::Id(spa::sys::SPA_META_Header)),
            },
            spa::pod::Property {
                key: spa::sys::SPA_PARAM_META_size,
                flags: spa::pod::PropertyFlags::empty(),
                value: spa::pod::Value::Int(std::mem::size_of::<spa::sys::spa_meta_header>() as i32),
            },
        ],
    });
    let params = vec![serialize(&buffers)?, serialize(&header)?, serialize(&meta)?];

    let _listener = stream
        .add_local_listener_with_user_data(StreamData { format: Default::default(), sender, params })
        .param_changed(|stream, data, id, param| {
            if std::env::var_os("EASYWSL_AGENT_DEBUG").is_some() {
                eprintln!("easywsl-agent: param {id} {}", if param.is_some() { "set" } else { "cleared" });
            }
            let Some(param) = param else { return };
            if id != spa::param::ParamType::Format.as_raw() {
                return;
            }
            if data.format.parse(param).is_ok() {
                let size = data.format.size();
                eprintln!("easywsl-agent: {}x{} {:?}", size.width, size.height, data.format.format());
                let mut pods: Vec<&Pod> = data.params.iter().filter_map(|p| Pod::from_bytes(p)).collect();
                if let Err(e) = stream.update_params(&mut pods) {
                    eprintln!("easywsl-agent: buffer parameters: {e}");
                }
            }
        })
        .process(|stream, data| unsafe {
            let raw = stream.dequeue_raw_buffer();
            if raw.is_null() {
                return;
            }
            let buffer = (*raw).buffer;
            if std::env::var_os("EASYWSL_AGENT_DEBUG").is_some() {
                let metas: Vec<(u32, u32)> = (0..(*buffer).n_metas as usize).map(|i| { let m = &*(*buffer).metas.add(i); (m.type_, m.size) }).collect();
                eprintln!("easywsl-agent: buffer metas {metas:?}");
            }
            if let Some(cursor) = cursor_shape(buffer) {
                data.sender.cursor(cursor);
            }
            let size = data.format.size();
            if (*buffer).n_datas > 0 && size.width > 0 {
                let plane = &*(*buffer).datas;
                let chunk = &*plane.chunk;
                // Buffers that only move the pointer carry no picture.
                if !plane.data.is_null() && chunk.size > 0 && chunk.flags & spa::sys::SPA_CHUNK_FLAG_CORRUPTED as i32 == 0 {
                    let stride = if chunk.stride > 0 { chunk.stride as usize } else { size.width as usize * 4 };
                    let length = stride * size.height as usize;
                    let offset = chunk.offset as usize % plane.maxsize.max(1) as usize;
                    if offset + length <= plane.maxsize as usize {
                        let bytes = std::slice::from_raw_parts((plane.data as *const u8).add(offset), length);
                        data.sender.frame(Frame { width: size.width, height: size.height, stride, pixels: bytes.to_vec() });
                    }
                }
            }
            stream.queue_raw_buffer(raw);
        })
        .register()?;

    // BGRx or BGRA: the viewer's own byte order, so frames need no conversion on either side.
    let format = spa::pod::object!(
        spa::utils::SpaTypes::ObjectParamFormat,
        spa::param::ParamType::EnumFormat,
        spa::pod::property!(spa::param::format::FormatProperties::MediaType, Id, spa::param::format::MediaType::Video),
        spa::pod::property!(spa::param::format::FormatProperties::MediaSubtype, Id, spa::param::format::MediaSubtype::Raw),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            spa::param::video::VideoFormat::BGRx,
            spa::param::video::VideoFormat::BGRx,
            spa::param::video::VideoFormat::BGRA
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            spa::utils::Rectangle { width: 1920, height: 1080 },
            spa::utils::Rectangle { width: 1, height: 1 },
            spa::utils::Rectangle { width: 8192, height: 8192 }
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            spa::utils::Fraction { num: 60, denom: 1 },
            spa::utils::Fraction { num: 0, denom: 1 },
            spa::utils::Fraction { num: 240, denom: 1 }
        ),
    );
    let bytes: Vec<u8> = spa::pod::serialize::PodSerializer::serialize(std::io::Cursor::new(Vec::new()), &spa::pod::Value::Object(format))?
        .0
        .into_inner();
    let mut params = [Pod::from_bytes(&bytes).ok_or("bad format description")?];
    stream.connect(
        spa::utils::Direction::Input,
        Some(node),
        pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
        &mut params,
    )?;
    mainloop.run();
    Ok(())
}
