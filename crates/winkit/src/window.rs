//! Cross-process window helpers: raise a window owned by another process, and
//! find a process's window by its command line.

use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, RECT};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentThreadId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, IsWindowVisible,
    SetForegroundWindow, SetWindowPos, HWND_TOP, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
};

/// `ProcessCommandLineInformation`.
const PROCESS_COMMAND_LINE_INFORMATION: u32 = 60;

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtQueryInformationProcess(
        process: HANDLE,
        class: u32,
        info: *mut core::ffi::c_void,
        len: u32,
        ret: *mut u32,
    ) -> i32;
}

/// Raises a window to the top of the z-order from this process.
///
/// The window may belong to another process, and when it was opened from a
/// window that is still foreground, this process is a background process: the
/// foreground lock would refuse both the activation and a `SetWindowPos(HWND_TOP)`
/// that outranks the foreground window. Attaching to the foreground thread's
/// input queue suspends that lock for the duration of the call, which is what
/// lets us raise (and focus) the window.
pub fn raise_window(hwnd: isize) {
    let hwnd = HWND(hwnd as *mut core::ffi::c_void);
    unsafe {
        let fg = GetForegroundWindow();
        let our_thread = GetCurrentThreadId();
        let fg_thread = if fg.0.is_null() {
            0
        } else {
            GetWindowThreadProcessId(fg, None)
        };
        let attached = fg_thread != 0
            && fg_thread != our_thread
            && AttachThreadInput(our_thread, fg_thread, true).as_bool();
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
        let _ = SetForegroundWindow(hwnd);
        if attached {
            let _ = AttachThreadInput(our_thread, fg_thread, false);
        }
    }
}

/// The handle of the largest visible top-level window owned by a process whose
/// command line contains `needle`. A process can own several visible top-level
/// windows (a popup, a docked devtools, …), and its main window is the largest,
/// so the largest match is the main window.
pub fn window_for_command_line(needle: &str) -> Option<isize> {
    let needle = needle.to_lowercase();
    if needle.is_empty() {
        return None;
    }
    struct Ctx<'a> {
        needle: &'a str,
        best: Option<(i64, isize)>,
    }
    unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = unsafe { &mut *(lparam.0 as *mut Ctx) };
        if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
            return BOOL(1);
        }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if let Some(cmd) = process_command_line(pid) {
            if cmd.to_lowercase().contains(ctx.needle) {
                let mut r = RECT::default();
                if unsafe { GetWindowRect(hwnd, &mut r) }.is_ok() {
                    let area = (r.right - r.left) as i64 * (r.bottom - r.top) as i64;
                    if ctx.best.is_none_or(|(a, _)| area > a) {
                        ctx.best = Some((area, hwnd.0 as isize));
                    }
                }
            }
        }
        BOOL(1)
    }
    let mut ctx = Ctx {
        needle: &needle,
        best: None,
    };
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut ctx as *mut Ctx as isize));
    }
    ctx.best.map(|(_, hwnd)| hwnd)
}

/// The full command line of `pid`, or `None` if it cannot be read.
pub fn process_command_line(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut size: u32 = 0;
        // First probe to learn the required buffer size.
        let _ = NtQueryInformationProcess(
            handle,
            PROCESS_COMMAND_LINE_INFORMATION,
            std::ptr::null_mut(),
            0,
            &mut size,
        );
        if size == 0 {
            size = 8192;
        }
        let mut buf = vec![0u8; size as usize];
        let status = NtQueryInformationProcess(
            handle,
            PROCESS_COMMAND_LINE_INFORMATION,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            buf.len() as u32,
            &mut size,
        );
        let result = if status >= 0 {
            // UNICODE_STRING: u16 length (bytes), u16 max, pad, then buffer
            // pointer at offset 8 on 64-bit.
            let length = *(buf.as_ptr() as *const u16) as usize;
            let buffer = *(buf.as_ptr().add(8) as *const *const u16);
            if length > 0 && !buffer.is_null() && length <= size as usize {
                Some(String::from_utf16_lossy(std::slice::from_raw_parts(
                    buffer,
                    length / 2,
                )))
            } else {
                None
            }
        } else {
            None
        };
        let _ = CloseHandle(handle);
        result
    }
}
