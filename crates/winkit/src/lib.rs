//! Small generic Win32 helpers shared by the UI crates: the primary display
//! scale, `HICON` loading, and cross-process window lookup/raise. Kept
//! dependency-light so `traykit`, `websurface` and `browserhost` can all build
//! on it.
//!
//! ```no_run
//! let _ = winkit::set_app_user_model_id("My.App");
//! winkit::enable_per_monitor_dpi();
//! let _scale = winkit::dpi_scale();
//! let _icon = winkit::from_ico(&[], 32);
//! if let Some(hwnd) = winkit::window_for_command_line("my-profile") {
//!     winkit::raise_window(hwnd);
//! }
//! ```

mod appid;
mod dpi;
mod icon;
mod window;

pub use appid::set_app_user_model_id;
pub use dpi::{dpi_scale, enable_per_monitor_dpi, primary_work_area};
pub use icon::{from_ico, set_window_icon};
pub use window::{process_command_line, raise_window, window_for_command_line};
