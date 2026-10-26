//! Ways into and out of full screen besides Ctrl+Alt+Enter, which nobody would guess: an item in
//! the window's menu (its icon, or Alt+Space), and a hint drawn over the picture for a few seconds
//! after full screen starts, saying how to leave it.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicIsize, Ordering};

use winit::event_loop::EventLoopProxy;
use winit::window::Window;

use crate::Message;

/// The menu item's command; Windows keeps the low four bits of system commands for itself.
const FULL_SCREEN_ITEM: usize = 0x0010;
const HINT: &str = "Full screen. Ctrl+Alt+Enter goes back to a window.";

static PROXY: OnceLock<EventLoopProxy<Message>> = OnceLock::new();
static OLD_PROC: AtomicIsize = AtomicIsize::new(0);

fn hwnd(window: &Window) -> Option<isize> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get()),
        _ => None,
    }
}

/// Adds "Full screen" to the window's menu; choosing it sends Message::ToggleFullscreen.
#[cfg(windows)]
pub fn add_menu_item(window: &Window, proxy: EventLoopProxy<Message>) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{AppendMenuW, GWLP_WNDPROC, GetSystemMenu, MF_SEPARATOR, MF_STRING, SetWindowLongPtrW};
    let Some(hwnd) = hwnd(window) else { return };
    let _ = PROXY.set(proxy);
    let label: Vec<u16> = "Full screen\tCtrl+Alt+Enter\0".encode_utf16().collect();
    unsafe {
        let menu = GetSystemMenu(hwnd as _, 0);
        if menu.is_null() {
            return;
        }
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(menu, MF_STRING, FULL_SCREEN_ITEM, label.as_ptr());
        // The window's own procedure is winit's; this one sees the menu command first.
        let old = SetWindowLongPtrW(hwnd as _, GWLP_WNDPROC, window_proc as *const () as isize);
        OLD_PROC.store(old, Ordering::SeqCst);
    }
}

#[cfg(not(windows))]
pub fn add_menu_item(_: &Window, _: EventLoopProxy<Message>) {}

#[cfg(windows)]
unsafe extern "system" fn window_proc(hwnd: windows_sys::Win32::Foundation::HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    use windows_sys::Win32::UI::WindowsAndMessaging::{CallWindowProcW, WM_SYSCOMMAND, WNDPROC};
    if msg == WM_SYSCOMMAND && wparam & 0xFFF0 == FULL_SCREEN_ITEM {
        if let Some(proxy) = PROXY.get() {
            let _ = proxy.send_event(Message::ToggleFullscreen);
        }
        return 0;
    }
    unsafe {
        let old: WNDPROC = std::mem::transmute(OLD_PROC.load(Ordering::SeqCst));
        CallWindowProcW(old, hwnd, msg, wparam, lparam)
    }
}

/// Draws the hint over the top middle of the window: white text on a dark box, sized for the
/// display's scaling. Drawn after each picture while it shows.
#[cfg(windows)]
pub fn draw_hint(window: &Window) {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateFontW, CreateSolidBrush, DT_CALCRECT, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject, DrawTextW, FW_SEMIBOLD,
        FillRect, GetDC, ReleaseDC, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
    };
    let Some(hwnd) = hwnd(window) else { return };
    let scale = window.scale_factor();
    let size = window.inner_size();
    let text: Vec<u16> = HINT.encode_utf16().collect();
    let face: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    unsafe {
        let dc = GetDC(hwnd as _);
        if dc.is_null() {
            return;
        }
        let font = CreateFontW(-((18.0 * scale) as i32), 0, 0, 0, FW_SEMIBOLD as i32, 0, 0, 0, 0, 0, 0, 5, 0, face.as_ptr());
        let old_font = SelectObject(dc, font);
        let mut measured = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        DrawTextW(dc, text.as_ptr(), text.len() as i32, &mut measured, DT_CALCRECT | DT_SINGLELINE);
        let (pad_x, pad_y) = ((20.0 * scale) as i32, (10.0 * scale) as i32);
        let w = measured.right + 2 * pad_x;
        let h = measured.bottom + 2 * pad_y;
        let left = (size.width as i32 - w) / 2;
        let top = (24.0 * scale) as i32;
        let mut rect = RECT { left, top, right: left + w, bottom: top + h };
        let brush = CreateSolidBrush(0x0030_2a28); // dark grey (0x00BBGGRR)
        FillRect(dc, &rect, brush);
        DeleteObject(brush);
        SetBkMode(dc, TRANSPARENT as i32);
        SetTextColor(dc, 0x00ff_ffff);
        DrawTextW(dc, text.as_ptr(), text.len() as i32, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        SelectObject(dc, old_font);
        DeleteObject(font);
        ReleaseDC(hwnd as _, dc);
    }
}

#[cfg(not(windows))]
pub fn draw_hint(_: &Window) {}
