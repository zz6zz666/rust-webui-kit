# AGENTS.md

Guidance for working in this repository: a Cargo workspace of reusable Rust
Windows web-UI building blocks, plus the Orbit demo.

## Layout

- `crates/winkit` — Win32 helpers: process DPI awareness/scale, `HICON` loading.
- `crates/traykit` — resident tray icon with a custom-drawn popup menu.
- `crates/browserhost` — locate, launch and drive an installed Chromium over CDP.
- `crates/webmsg` — loopback HTTP bridge between a web page and its host.
- `crates/websurface` — multi-window host for web-technology UIs (WebView2 plus a
  borrowed-Chromium fallback).
- `crates/winres-embed` — build-time embedding of a Windows icon + version
  resource (used from `demo/build.rs`).
- `demo/` — `orbit-demo`, a playable multi-window tray app built from the crates.

Dependency direction: `websurface` → { `webmsg`, `browserhost`, `winkit` };
`traykit` → `winkit`; `demo` → all of them (`winres-embed` only as a
build-dependency).

## Commands

Prepend the toolchain to `PATH` in a fresh PowerShell session:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
```

- Check — must be warning-free:
  ```powershell
  cargo check --workspace --all-targets --message-format short
  ```
- Unit tests:
  ```powershell
  cargo test --workspace
  ```
- Run the demo:
  ```powershell
  cargo run -p orbit-demo
  ```
- Feature-combination checks:
  ```powershell
  cargo check -p websurface --no-default-features
  cargo check -p websurface --no-default-features --features webview2
  cargo check -p websurface --no-default-features --features borrowed-browser
  ```

## Conventions

- Do not add comments unless they explain *why*.
- Crates use English error messages and must not contain application-specific
  strings or policy; the hosting app maps errors to its own user-facing text.
- Keep `cargo check --workspace --all-targets` at zero warnings.
