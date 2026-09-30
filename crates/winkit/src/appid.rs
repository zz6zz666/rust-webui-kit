//! The process AppUserModelID, which the shell uses to group an app's windows
//! and attribute toast notifications.

use windows::core::PCWSTR;
use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

/// Sets the process AppUserModelID. Call it before creating any window so the
/// shell groups the app's windows (and its WebView2 processes) under it and
/// attributes notifications to the app rather than to a generic host.
pub fn set_app_user_model_id(id: &str) -> windows::core::Result<()> {
    let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { SetCurrentProcessExplicitAppUserModelID(PCWSTR(wide.as_ptr())) }
}
