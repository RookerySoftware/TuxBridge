//! Windows specifics: running a program elevated (the "Yes" prompt), checking a file's digital
//! signature, and facts about the PC that need no WMI.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::Security::WinTrust::{
    WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_FILE_INFO, WTD_CHOICE_FILE,
    WTD_REVOKE_NONE, WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY, WTD_UI_NONE, WinVerifyTrust,
};
use windows::Win32::System::Threading::{GetExitCodeProcess, INFINITE, WaitForSingleObject};
use windows::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::core::PCWSTR;

fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(std::iter::once(0)).collect()
}

/// Runs a program as administrator and waits for it. Windows shows its "Yes" prompt; a "No"
/// there is an error. Returns the program's exit code.
pub fn run_elevated(program: &Path, args: &str) -> anyhow::Result<u32> {
    let verb = wide("runas".as_ref());
    let file = wide(program.as_os_str());
    let params = wide(args.as_ref());
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: HWND::default(),
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    unsafe {
        ShellExecuteExW(&mut info).map_err(|_| anyhow::anyhow!("permission was not given"))?;
        let process: HANDLE = info.hProcess;
        anyhow::ensure!(!process.is_invalid(), "the program did not start");
        WaitForSingleObject(process, INFINITE);
        let mut code = 0u32;
        let _ = GetExitCodeProcess(process, &mut code);
        let _ = CloseHandle(process);
        Ok(code)
    }
}

/// Whether a file carries a valid Authenticode signature (as Get-AuthenticodeSignature's "Valid").
pub fn signature_valid(path: &Path) -> bool {
    let file = wide(path.as_os_str());
    let mut file_info = WINTRUST_FILE_INFO {
        cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: PCWSTR(file.as_ptr()),
        ..Default::default()
    };
    let mut data = WINTRUST_DATA {
        cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 { pFile: &mut file_info },
        dwStateAction: WTD_STATEACTION_VERIFY,
        ..Default::default()
    };
    let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    unsafe {
        let result = WinVerifyTrust(HWND(-1isize as _), &mut action, &mut data as *mut _ as *mut _);
        data.dwStateAction = WTD_STATEACTION_CLOSE;
        WinVerifyTrust(HWND(-1isize as _), &mut action, &mut data as *mut _ as *mut _);
        result == 0
    }
}

/// Whether a hypervisor runs (CPUID leaf 1, ECX bit 31). With Windows' own hypervisor running,
/// Windows reports the firmware's virtualization switch as off, so this is asked first.
pub fn hypervisor_present() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        let r = std::arch::x86_64::__cpuid(1);
        (r.ecx >> 31) & 1 == 1
    }
    #[cfg(not(target_arch = "x86_64"))]
    true
}

/// Whether virtualization is switched on in the PC's firmware (BIOS/UEFI).
pub fn virtualization_firmware_enabled() -> bool {
    use windows::Win32::System::Threading::{IsProcessorFeaturePresent, PF_VIRT_FIRMWARE_ENABLED};
    unsafe { IsProcessorFeaturePresent(PF_VIRT_FIRMWARE_ENABLED).as_bool() }
}
