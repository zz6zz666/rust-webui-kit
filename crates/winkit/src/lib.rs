//! Small generic Win32 helpers shared by the UI crates: the primary display
//! scale and `HICON` loading. Kept dependency-light so `traykit` and
//! `websurface` can both build on it.
//!
//! ```no_run
//! let _ = winkit::set_app_user_model_id("My.App");
//! winkit::enable_per_monitor_dpi();
//! let _scale = winkit::dpi_scale();
//! let _icon = winkit::from_ico(&[], 32);
//! ```

mod appid;
mod dpi;
mod icon;

pub use appid::set_app_user_model_id;
pub use dpi::{dpi_scale, enable_per_monitor_dpi};
pub use icon::{from_ico, set_window_icon};
