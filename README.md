# CX Browser

CX Browser is a lightweight web browser for Windows built in Rust. It uses the WebView2 COM API directly rather than a wrapper library, which gives it proper per-tab isolation, built-in ad and tracker blocking at the network request level, and fine-grained control over memory and GPU settings.

The goal is a browser that stays out of your way, does not collect anything, and does not eat your RAM.

---

## What It Does

- Tabbed browsing with tab groups — color coded, collapsible, persist across sessions
- Built-in ad and tracker blocking — EasyList and EasyPrivacy compatible, runs in Rust before requests are made
- Ghost Mode — fully isolated private browsing using a separate WebView2 profile, wiped on close
- Per-tab memory limits and automatic tab hibernation when limits are approached
- Hard memory ceiling via Windows Job Objects — the browser cannot exceed it
- Dark and light themes with customizable accent colors
- Application layouts — side by side, focus mode, research view, dashboard grid
- No telemetry, no accounts, no phoning home

---

## Status

Early development. Core browsing and the basic UI shell are in place. Most of the modules described in DESIGN.md are still being built out.

Current source is a single main.rs while the architecture is being established. Module splits into tabs.rs, blocker.rs, ipc.rs, resources.rs etc are coming as each piece solidifies.

---

## Tech

- Rust
- webview2-com (direct WebView2 COM API — not wry)
- windows crate (Win32, Job Objects, GDI, DWM)
- serde + serde_json for config and IPC

---

## Building

Requires the WebView2 runtime to be installed (ships with Windows 11, available for Windows 10).

```
cargo build --release
```

Binary lands at target/release/cx_browser.exe.

---

## Design

The full design document covering architecture, privacy model, resource management, theme system, and development phases is in DESIGN.md.

---

## License

Apache 2.0 — see LICENSE for details.