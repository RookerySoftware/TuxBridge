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
/// Keys on their way to the desktop. The hook only queues them: Windows quietly removes a hook
/// that keeps it waiting (about a second), and writing to Linux can wait that long when the
/// desktop is busy. The Windows key would then open the Start menu until the desktop reopened.
static KEYS: OnceLock<Sender<Input>> = OnceLock::new();

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

fn start_sending(sink: InputSink) {
    let (keys, queue) = std::sync::mpsc::channel();
    if KEYS.set(keys).is_ok() {
        std::thread::spawn(move || queue.into_iter().for_each(|input| sink.send(input)));
    }
}

#[cfg(windows)]
pub fn install(sink: InputSink) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, SetWindowsHookExW, WH_KEYBOARD_LL};
    start_sending(sink);
    std::thread::spawn(|| unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), std::ptr::null_mut(), 0);
        if hook.is_null() {
            return;
        }
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
        if code == HC_ACTION as i32 && FOCUSED.load(Ordering::SeqCst) && GetForegroundWindow() as isize == WINDOW.load(Ordering::SeqCst) {
            let info = &*(lparam as *const KBDLLHOOKSTRUCT);
            let evdev = match info.vkCode as u16 {
                VK_LWIN => Some(KEY_LEFTMETA),
                VK_RWIN => Some(KEY_RIGHTMETA),
                _ => None,
            };
            if let (Some(evdev), Some(keys)) = (evdev, KEYS.get()) {
                let pressed = matches!(wparam as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
                if pressed || matches!(wparam as u32, WM_KEYUP | WM_SYSKEYUP) {
                    let _ = keys.send(Input::Key { code: evdev, pressed });
                    return 1;
                }
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
    }
}
