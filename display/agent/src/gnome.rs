//! A GNOME Shell session, through Mutter's own remote-desktop interfaces (the ones GNOME's
//! remote desktop server uses): a RemoteDesktop session for input, a ScreenCast session tied to
//! it for frames, which arrive as a PipeWire video stream.

use std::collections::HashMap;
use std::error::Error;

use easywsl_protocol::Input;
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
    // cursor-mode 1: the pointer is drawn into the frames.
    let mut options: HashMap<&str, Value> = HashMap::new();
    options.insert("cursor-mode", Value::U32(1));
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
}

/// Receives the PipeWire video stream and hands each frame to the sender.
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

    let _listener = stream
        .add_local_listener_with_user_data(StreamData { format: Default::default(), sender })
        .param_changed(|_, data, id, param| {
            let Some(param) = param else { return };
            if id != spa::param::ParamType::Format.as_raw() {
                return;
            }
            if data.format.parse(param).is_ok() {
                let size = data.format.size();
                eprintln!("easywsl-agent: {}x{} {:?}", size.width, size.height, data.format.format());
            }
        })
        .process(|stream, data| {
            let Some(mut buffer) = stream.dequeue_buffer() else { return };
            let size = data.format.size();
            let datas = buffer.datas_mut();
            let Some(plane) = datas.first_mut() else { return };
            let chunk = plane.chunk();
            let (offset, stride) = (chunk.offset() as usize, chunk.stride().max(0) as usize);
            let stride = if stride == 0 { size.width as usize * 4 } else { stride };
            let length = stride * size.height as usize;
            let Some(bytes) = plane.data() else { return };
            if size.width == 0 || bytes.len() < offset + length {
                return;
            }
            data.sender.frame(Frame { width: size.width, height: size.height, stride, pixels: bytes[offset..offset + length].to_vec() });
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
