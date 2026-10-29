//! The Windows key belongs to Linux while the viewer has focus. Windows handles it before any
//! window sees it (the Start menu), so a low-level keyboard hook takes it first, sends it to the
//! desktop and swallows it. Only while the viewer's window is focused; Ctrl+Alt+Del and Win+L stay
//! Windows' (the system does not let any program have them).

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::Sender;

use easywsl_protocol::Input;

use crate::InputSink;
use crate::keys::{KEY_LEFTMETA, KEY_RIGHTMETA};

static FOCUSED: AtomicBool = AtomicBool::new(false);
static WINDOW: AtomicIsize = AtomicIsize::new(0);
/// Where the keys go: the viewer's input queue, which never makes the hook wait.
static KEYS: OnceLock<InputSink> = OnceLock::new();
/// Which Windows keys (left, right) Linux has been told are down. Windows repeats a held key's
/// key-down, and Linux's keyboard state counts every one, so a single key-up would leave Super
/// held there (arrows becoming Super+arrows, shortcuts misfiring): only the first goes through.
static DOWN: [AtomicBool; 2] = [AtomicBool::new(false), AtomicBool::new(false)];

pub fn set_focused(focused: bool) {
    FOCUSED.store(focused, Ordering::SeqCst);
}

pub fn set_window(window: &winit::window::Window) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(h) = handle.as_raw() {
            WINDOW.store(h.hwnd.get(), Ordering::SeqCst);
        }
    }
}

/// Tells Linux the Windows keys are up, when the window loses focus with one held.
pub fn release() {
    for (down, code) in DOWN.iter().zip([KEY_LEFTMETA, KEY_RIGHTMETA]) {
        if down.swap(false, Ordering::SeqCst) {
            if let Some(keys) = KEYS.get() {
                keys.send(Input::Key { code, pressed: false });
            }
        }
    }
}

/// What the hook did with the first presses, for the desktop's log (written off the hook's thread,
/// so the hook never waits on it).
static NOTES: OnceLock<Sender<String>> = OnceLock::new();
static NOTED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn note(line: String) {
    if NOTED.fetch_add(1, Ordering::SeqCst) < 40 {
        if let Some(notes) = NOTES.get() {
            let _ = notes.send(line);
        }
    }
}

fn start_sending(sink: InputSink) {
    let (notes, lines) = std::sync::mpsc::channel::<String>();
    if NOTES.set(notes).is_ok() {
        std::thread::spawn(move || lines.into_iter().for_each(|line| eprintln!("{line}")));
    }
    let _ = KEYS.set(sink);
}

#[cfg(windows)]
pub fn install(sink: InputSink) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, SetWindowsHookExW, WH_KEYBOARD_LL};
    start_sending(sink);
    std::thread::spawn(|| unsafe {
        let module = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), module, 0);
        if hook.is_null() {
            eprintln!("viewer: the Windows key stays with Windows (keyboard hook refused, error {})", windows_sys::Win32::Foundation::GetLastError());
            return;
        }
        eprintln!("viewer: keyboard hook installed");
        // A low-level hook runs on the thread that installed it, which needs a message loop.
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {}
    });
}

#[cfg(not(windows))]
pub fn install(sink: InputSink) {
    start_sending(sink);
}

#[cfg(windows)]
unsafe extern "system" fn keyboard_hook(code: i32, wparam: usize, lparam: isize) -> isize {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{VK_LWIN, VK_RWIN};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetForegroundWindow, HC_ACTION, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };
    unsafe {
        if code == HC_ACTION as i32 {
            let info = &*(lparam as *const KBDLLHOOKSTRUCT);
            let (index, evdev) = match info.vkCode as u16 {
                VK_LWIN => (0, KEY_LEFTMETA),
                VK_RWIN => (1, KEY_RIGHTMETA),
                _ => return CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam),
            };
            let pressed = matches!(wparam as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
            let released = matches!(wparam as u32, WM_KEYUP | WM_SYSKEYUP);
            if let (Some(keys), true) = (KEYS.get(), pressed || released) {
                // Windows' own word on which window is in front; winit's focus events alone can
                // miss the window being active from the start.
                let ours = GetForegroundWindow() as isize == WINDOW.load(Ordering::SeqCst);
                if pressed && ours {
                    if !DOWN[index].swap(true, Ordering::SeqCst) {
                        note(format!("viewer: Windows key down: focused {}, sent to Linux", FOCUSED.load(Ordering::SeqCst)));
                        keys.send(Input::Key { code: evdev, pressed: true });
                    }
                    return 1;
                }
                // A key Linux has down is released there, wherever the window is by then.
                if released && DOWN[index].swap(false, Ordering::SeqCst) {
                    note("viewer: Windows key up: sent to Linux".into());
                    keys.send(Input::Key { code: evdev, pressed: false });
                    return 1;
                }
                if pressed {
                    note(format!("viewer: Windows key down: viewer not in front (focused {}), left to Windows", FOCUSED.load(Ordering::SeqCst)));
                }
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
    }
}
