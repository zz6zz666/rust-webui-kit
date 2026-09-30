//! Native, console-free process management for our dedicated browser profiles.
//!
//! Chromium hands a new launch off to an existing process that is already using
//! the same `--user-data-dir`; that process then exits without a debugging
//! port, which would break us. Before launching we therefore terminate any
//! browser process bound to our profile. This is done with Win32 APIs only
//! (Toolhelp snapshot + command-line query), so nothing flashes a console.

use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

/// Terminates every browser process whose command line references `dir`.
/// Best-effort: failures (e.g. another user's/elevated process) are ignored.
pub fn kill_for_profile(dir: &str) {
    let needle = dir.to_lowercase();
    if needle.is_empty() {
        return;
    }
    let me = std::process::id();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return;
        };
        if snapshot == INVALID_HANDLE_VALUE {
            return;
        }
        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let pid = entry.th32ProcessID;
                // Match on the command line so any Chromium fork the user made
                // default (not just the well-known names) is reaped, while never
                // touching our own process.
                if pid != 0 && pid != me {
                    if let Some(cmd) = winkit::process_command_line(pid) {
                        if cmd.to_lowercase().contains(&needle) {
                            terminate(pid);
                        }
                    }
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
}

unsafe fn terminate(pid: u32) { unsafe {
    if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
        let _ = TerminateProcess(handle, 1);
        let _ = CloseHandle(handle);
    }
}}
