# CX Browser — Design Document

## Overview

CX Browser is a lightweight, standalone web browser for Windows 11 built in Rust using the **webview2-com** crate (direct WebView2 COM API). It prioritizes speed, low memory usage, privacy, and built-in ad/tracker blocking.

**Why webview2-com over wry?** Direct COM access gives us multiple webviews per window, per-tab WebView2 profiles for Ghost Mode, `WebResourceRequested` interception for ad blocking, and fine-grained control over GPU flags and resource policies.

---

## Features

### Core Browsing
- URL navigation with smart address bar (auto-prefix `https://`, fallback to Google search)
- Back / Forward / Reload / Home controls
- Tabbed browsing (create, close, switch tabs)
- Page title display per tab
- Status bar with loading state

### Tracking & Ad Blocking
- **Built-in content blocker** — blocks ads, trackers, and fingerprinting scripts
- Filter lists loaded at startup from bundled EasyList / EasyPrivacy-compatible rules
- Custom blocklist support — users can add their own filter list URLs
- **Do Not Track** header sent on all requests
- **Third-party cookie blocking** by default
- **Referrer trimming** — strips full referrer to origin-only on cross-site requests
- Toggle on/off per-site via shield icon in toolbar
- Block counter badge shows number of blocked requests per page
- WebRTC IP leak prevention (disable non-proxied UDP)

### Themes
- **Dark theme** (default) — slate & lake palette
- **Light theme** — marble palette
- Theme toggle button in the toolbar
- **User-customizable accent colors** — four accent slots editable in settings
- Theme + accent preferences persisted to local config file (`%APPDATA%/CXBrowser/settings.json`)

### Bookmarks
- Add / remove bookmarks from toolbar
- Bookmark bar (toggleable)
- Stored in local JSON file

### History
- Per-session browsing history for back/forward
- Persistent history log with timestamps
- Clear history option

### Downloads
- Basic download handling via WebView2
- Default save-to location with prompt

### Find in Page
- `Ctrl+F` find bar overlay
- Match highlighting and navigation

### Ghost Mode (Private Browsing)
- Open a private window via menu or `Ctrl+Shift+N`
- Visual indicator — toolbar tinted with ghost icon, distinct from normal windows
- **No history saved** — browsing history, search entries, and form data are discarded on close
- **No cookies persisted** — all cookies and site data cleared when the ghost window closes
- **No cache retained** — page cache lives only in memory for the session
- **Downloads kept** — downloaded files remain on disk (user is notified)
- **Bookmarks allowed** — user can still save bookmarks from ghost mode
- **Ad/tracker blocking stays active** — filters still apply in ghost mode
- **All ghost tabs isolated** — separate WebView2 profile/user data folder (in-memory or temp directory, wiped on close)

### Tab Groups
- Right-click a tab → **Add to Group** → choose existing group or create new
- Each group has a user-chosen **name** and **color** (from accent palette or custom)
- Groups are visually separated by a colored bar/divider in the tab strip
- **Collapse/expand** a group by clicking its label — collapsed groups show only the label and tab count
- Drag tabs between groups, or drag a tab out of a group to ungroup it
- Close entire group at once (right-click group label → Close Group)
- Groups persist across sessions (saved in `settings.json`)
- Ghost mode windows have their own independent group space

### Application Layouts
Predefined and custom window/tab arrangements for different workflows.

| Layout         | Description                                                    |
|----------------|----------------------------------------------------------------|
| **Single**     | One tab, full width (default)                                  |
| **Side by Side** | Two tabs split 50/50 horizontally                            |
| **Focus**      | Single tab, toolbar auto-hides, distraction-free               |
| **Research**   | Three-column: bookmarks sidebar + main content + secondary tab |
| **Dashboard**  | 2×2 grid of four tabs                                          |

- Switch layouts via toolbar dropdown or `Ctrl+Shift+L`
- **Custom layouts** — user can save current tab arrangement as a named layout
- Layouts remember which tab groups are assigned to which position
- Layout selection persisted in `settings.json`

### Keyboard Shortcuts
| Shortcut         | Action              |
|------------------|---------------------|
| `Ctrl+T`         | New tab             |
| `Ctrl+W`         | Close tab           |
| `Ctrl+Tab`       | Next tab            |
| `Ctrl+Shift+Tab` | Previous tab        |
| `Ctrl+Shift+N`   | New ghost window    |
| `Ctrl+Shift+L`   | Switch layout       |
| `Ctrl+G`         | Group selected tabs |
| `Ctrl+L`         | Focus address bar   |
| `Ctrl+R` / `F5`  | Reload              |
| `Alt+Left`       | Back                |
| `Alt+Right`      | Forward             |
| `Ctrl+F`         | Find in page        |
| `Ctrl+,`         | Open settings       |
| `F11`            | Toggle fullscreen   |

---

## Security

### Content Security
- **WebView2 sandbox** — all web content runs in the OS-provided WebView2 sandbox (Chromium-based process isolation)
- **No custom JS injection into page content** — UI layer is fully separated from browsed pages
- IPC messages between UI shell and Rust backend are validated and typed (serde JSON with strict deserialization)
- No `eval()` or dynamic script execution from external sources

### Network Security
- HTTPS enforced by default; HTTP connections show a visible warning in the address bar
- Certificate errors surfaced to the user (no silent bypass)
- No custom certificate store — relies on Windows system trust store

### Privacy & Anti-Tracking
- No telemetry or analytics
- No third-party service calls from the browser itself
- History and bookmarks stored locally only
- Option to clear all browsing data on exit ("session mode")
- **Built-in tracker blocking** — EasyPrivacy-compatible filter engine runs in Rust
- **Third-party cookies blocked** by default
- **Referrer policy** — `strict-origin-when-cross-origin` enforced
- **Do Not Track / GPC headers** sent on every request
- **WebRTC leak prevention** — non-proxied UDP disabled
- **Fingerprint resistance** — common fingerprinting vectors (canvas, font enumeration) limited where WebView2 allows

### Input Validation
- URL bar input sanitized before navigation (prevent `javascript:` and `data:` URI attacks)
- IPC message payloads validated against known message types; unknown messages are dropped
- File paths for downloads/config use safe path construction (no path traversal)

### Update Security
- (Future) Signed binary updates only
- No auto-update without user consent

---

## Themes

### Brand Accent Colors (User-Customizable)
These four accents are used across both themes for buttons, links, active states, and highlights. Users can change them in Settings.

| Slot       | Default   | Usage                          |
|------------|-----------|--------------------------------|
| Accent 1   | `#3E54D3` | Primary buttons, active tab    |
| Accent 2   | `#4F80E2` | Links, URL bar focus ring      |
| Accent 3   | `#15CDCA` | Success states, shield icon    |
| Accent 4   | `#4FE0B6` | Hover highlights, badge counts |

### Dark Theme — "Slate Lake" (Default)
Deep blue-grey slate tones inspired by a lake at dusk.

| Element        | Color     |
|----------------|-----------|
| Background     | `#0F1923` |
| Surface        | `#152231` |
| Overlay        | `#1E3044` |
| Border         | `#2A4057` |
| Text           | `#D4DDE8` |
| Subtext        | `#7A8FA6` |
| Accent 1       | `#3E54D3` |
| Accent 2       | `#4F80E2` |
| Accent 3       | `#15CDCA` |
| Accent 4       | `#4FE0B6` |
| Error/Warning  | `#E85A6E` |
| Success        | `#4FE0B6` |

### Light Theme — "Marble"
Clean white-grey tones with subtle warmth, like polished marble.

| Element        | Color     |
|----------------|-----------|
| Background     | `#F5F5F7` |
| Surface        | `#EAEAEF` |
| Overlay        | `#DDDDE5` |
| Border         | `#C8C8D4` |
| Text           | `#1E1E2E` |
| Subtext        | `#5C5C72` |
| Accent 1       | `#3E54D3` |
| Accent 2       | `#4F80E2` |
| Accent 3       | `#15CDCA` |
| Accent 4       | `#4FE0B6` |
| Error/Warning  | `#C62839` |
| Success        | `#1AA67E` |

### Customization
- Users open **Settings > Appearance** to:
  - Toggle dark / light mode
  - Edit each of the 4 accent color slots via color picker
  - Reset accents to defaults
- All customizations saved to `settings.json`

### Logo
- **Dark mode** → `images/CATX_Dark_Mode.png` (light pastel CXT mark — visible on dark backgrounds)
- **Light mode** → `images/CATX_Light_Mode.png` (black CXT mark — visible on light backgrounds)
- Logo displayed in the new-tab page and as the window icon
- Swaps automatically when theme changes

---

## Settings Layout

Settings open in a dedicated tab (`cx://settings`) with a sidebar navigation.

```
┌─────────────────────────────────────────────────┐
│  CX Browser Settings                      [×]   │
├──────────────┬──────────────────────────────────┤
│              │                                  │
│  ▸ General   │  [Currently selected section]    │
│  ▸ Appearance│                                  │
│  ▸ Privacy   │  Form controls, toggles, and     │
│  ▸ Security  │  sliders rendered here            │
│  ▸ Content   │                                  │
│    Blocking  │                                  │
│  ▸ Performance│                                 │
│  ▸ Downloads │                                  │
│  ▸ Shortcuts │                                  │
│  ▸ About     │                                  │
│              │                                  │
└──────────────┴──────────────────────────────────┘
```

### General
- Home page URL
- Default search engine (Google, DuckDuckGo, Brave Search, custom)
- Startup behavior (restore last session / new tab / home page)
- Language

### Appearance
- Theme toggle (dark / light)
- Accent color pickers (4 slots with hex input + color wheel)
- Reset to defaults button
- Font size override
- Tab bar position (top / side — future)
- Show/hide bookmark bar

### Privacy
- **Tracking protection level**: Standard / Strict / Custom
  - Standard: block known trackers + third-party cookies
  - Strict: add fingerprint resistance, block all cross-site cookies
  - Custom: per-toggle control of each feature
- Clear browsing data (history, cookies, cache, downloads — individually selectable)
- Clear data on exit toggle
- Do Not Track / GPC toggle
- WebRTC policy (default / disable non-proxied UDP / disable entirely)
- Block `javascript:` and `data:` URLs toggle
- Per-site permissions list (camera, mic, location, notifications — with deny-by-default)

### Security
- HTTPS-Only mode toggle (warn / block on HTTP)
- Certificate error behavior (warn / hard block)
- Safe browsing (future: integration with threat lists)

### Content Blocking
- Enable/disable content blocker globally
- Manage filter lists (enable/disable bundled lists)
- Add custom filter list URLs
- Per-site allow list (whitelist sites from blocking)
- View blocked request count stats

### Performance
- **Graphics acceleration**: enable / disable / auto-detect
- **Resource limits**:
  - Max memory per tab (slider: 128 MB – 2 GB, default: 512 MB)
  - Max total browser memory (slider: 512 MB – 8 GB, default: 2 GB)
  - CPU throttle for background tabs (aggressive / balanced / off)
  - Max concurrent tabs loading (1 – 10, default: 3)
- Tab hibernation (auto-suspend inactive tabs after N minutes)
- Preload / prefetch toggle

### Downloads
- Default download directory
- Always ask where to save toggle
- Auto-open safe file types toggle

### Shortcuts
- Full list of keyboard shortcuts (read-only reference)
- Custom shortcut remapping (future)

### About
- CX Browser version, build info
- WebView2 runtime version
- Check for updates button
- Licenses / credits

---

## Graphics Acceleration

- Leverage WebView2's built-in **GPU-accelerated rendering** (Direct3D 11/12 on Windows)
- Settings toggle: **Auto** (default) / **Force enabled** / **Disabled**
- When disabled, falls back to software rasterizer (WARP) for compatibility
- GPU acceleration benefits: smooth scrolling, CSS animations, WebGL, video decode (H.264/VP9 hardware decode)
- Rust backend passes WebView2 environment flags:
  - `--enable-gpu-rasterization` (when force-enabled)
  - `--disable-gpu` (when disabled)
- GPU info displayed in **Settings > About** for diagnostics
- Auto-detect disables GPU if crash-on-launch is detected (crash counter in config)

---

## Resource Management

Users can cap the browser's resource consumption in **Settings > Performance**.

### Memory Limits
- **Per-tab limit** — when a tab exceeds its memory budget, it is flagged in the tab bar with a warning icon; user can choose to hibernate or close
- **Total browser limit** — when total memory approaches the cap, oldest background tabs are auto-hibernated
- Hibernated tabs show a "sleeping" indicator and reload on click
- Memory usage visible in status bar (optional toggle)

### CPU Throttling
- **Background tab throttling** — tabs not in view have timers and requestAnimationFrame reduced
  - *Aggressive*: 1 wake-up per 60s, no rAF
  - *Balanced*: 1 wake-up per 10s (default)
  - *Off*: no throttling
- Audio-playing tabs are exempt from throttling
- Optional CPU usage display per tab (right-click tab → View resource usage)

### Network Limits (Future)
- Per-tab bandwidth cap
- Global bandwidth limit for metered connections

### Implementation — Hard Limits via Windows Job Objects
- **Job Object** created at browser startup via `CreateJobObjectW`
- Browser process + all WebView2 child processes assigned to the Job via `AssignProcessToJobObject`
- `SetInformationJobObject` with `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`:
  - `JOB_OBJECT_LIMIT_JOB_MEMORY` — hard ceiling on total committed memory for all processes in the Job
  - When exceeded, the OS kills the offending process (a renderer tab), not the whole browser
- **Monitoring loop** (Resource Monitor thread):
  1. Every 2 seconds, call `QueryInformationJobObject` to read current memory usage
  2. At 80% of limit → start hibernating oldest background tabs (soft response)
  3. At 95% of limit → force-hibernate all background tabs (aggressive)
  4. If a renderer process is killed by the OS → show "Tab crashed — out of memory" page, offer reload
- **Per-tab soft limits** — `MemoryUsageTargetLevel::Low` set on background tabs via WebView2 API; this hints the renderer to reduce caches but is not a hard cap
- **CPU throttle** — `put_IsVisible(false)` + `TryClearBrowserCache` on hibernated tabs; WebView2 natively throttles invisible webviews
- **Minimum floors enforced in settings UI**: 256 MB total minimum, 64 MB per-tab minimum — prevents users from making the browser unusable

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│  Rust Process (Win32 + webview2-com)             │
│                                                  │
│  ┌──────────┐ ┌──────────┐ ┌───────────────────┐│
│  │ Win32    │ │ Config   │ │ Job Object        ││
│  │ Window   │ │ Store    │ │ (memory hard cap) ││
│  └────┬─────┘ └──────────┘ └───────────────────┘│
│       │                                          │
│  ┌────▼──────────────────────────────────┐       │
│  │  UI WebView (shell)                   │       │
│  │  toolbar, tabs bar, theme, status     │       │
│  │  Positioned at top of window          │       │
│  └────┬──────────────────────────────────┘       │
│       │ IPC (JSON via web_message)                │
│  ┌────▼──────────────────────────────────┐       │
│  │  Content WebView (per tab)            │       │
│  │  Fills remaining window area          │       │
│  │  Each has own CoreWebView2Controller  │       │
│  │  Ghost tabs use isolated profile      │       │
│  └────┬──────────────────────────────────┘       │
│       │                                          │
│  ┌────▼──────────────────────────────────┐       │
│  │  WebResourceRequested handler         │       │
│  │  Runs Rust filter engine per request  │       │
│  │  Blocks ads/trackers before loading   │       │
│  └───────────────────────────────────────┘       │
│                                                  │
│  ┌───────────────────────────────────────┐       │
│  │  Resource Monitor thread              │       │
│  │  Polls process memory via Job Object  │       │
│  │  Hibernates/kills tabs at limits      │       │
│  └───────────────────────────────────────┘       │
└──────────────────────────────────────────────────┘
```

### Components
- **Win32 Window** — created via `windows` crate (`CreateWindowExW`), hosts all webviews as child HWNDs
- **UI WebView** — renders toolbar, tab bar, theme; communicates with Rust via `add_WebMessageReceived`
- **Content WebViews** — one `CoreWebView2Controller` per tab, positioned below the UI webview; visibility toggled on tab switch
- **Ghost Mode Profiles** — separate `CoreWebView2Environment` with temp user data folder, deleted on window close
- **WebResourceRequested** — registered on each content webview; Rust filter engine checks URL against block lists and returns `Deny` for matches
- **Job Object** — Win32 Job Object wrapping all browser + WebView2 renderer processes for hard memory caps
- **Resource Monitor** — background thread polling `QueryInformationJobObject` for memory usage; triggers hibernation before hard kill
- **Settings WebView** — `cx://settings` rendered as a special content tab, communicates with backend via same IPC
- **Config Store** — reads/writes JSON files in `%APPDATA%/CXBrowser/`

---

## File Structure

```
CX Browser/
├── Cargo.toml
├── DESIGN.md
├── images/
│   ├── CATX_Dark_Mode.png   # Logo for dark theme (light/pastel mark)
│   └── CATX_Light_Mode.png  # Logo for light theme (black mark)
├── src/
│   ├── main.rs          # Entry point, Win32 window + WebView2 setup
│   ├── webview.rs       # WebView2 COM wrapper helpers
│   ├── ui.html          # Browser shell UI (toolbar, tabs)
│   ├── ipc.rs           # IPC message types and handlers
│   ├── tabs.rs          # Tab state management (one CoreWebView2Controller per tab)
│   ├── config.rs        # Settings persistence (theme, accents, bookmarks)
│   ├── security.rs      # URL validation, safe path helpers
│   ├── blocker.rs       # Ad & tracker filter engine (WebResourceRequested)
│   ├── layouts.rs       # Application layout management
│   ├── groups.rs        # Tab group state and persistence
│   ├── resources.rs     # Job Object setup, resource monitor thread
│   └── settings.html    # Settings page UI (cx://settings)
```

---

## Config Storage

Location: `%APPDATA%/CXBrowser/`

```
CXBrowser/
├── settings.json    # theme, accents, layouts, tab groups, resource limits, all prefs
├── bookmarks.json   # bookmark entries
├── history.json     # browsing history
├── layouts.json     # saved custom application layouts
└── filters/         # ad/tracker block lists
    ├── easylist.txt
    ├── easyprivacy.txt
    └── custom.txt   # user-added rules
```

---

## Build & Run

```sh
cargo build --release
# Binary: target/release/cx_browser.exe
```

---

## Development Phases

### v0.1 — Core Shell
- Win32 window with embedded WebView2 via `webview2-com`
- UI webview (toolbar: back, forward, reload, home, URL bar)
- Single content webview for page navigation
- Dark (Slate Lake) and Light (Marble) themes with brand accent colors
- Logo on new-tab page, swaps with theme
- Basic keyboard shortcuts (Ctrl+L, Ctrl+R, F5, F11)

### v0.2 — Tabs & Groups
- Multiple content webviews (one per tab)
- Tab bar in UI webview with create/close/switch
- Tab groups (color, name, collapse/expand)
- Ctrl+T, Ctrl+W, Ctrl+Tab shortcuts
- Session restore (reopen tabs from last session)

### v0.3 — Ad & Tracker Blocking
- `WebResourceRequested` handler on each content webview
- Rust filter engine parsing EasyList/EasyPrivacy-format rules
- Shield icon with blocked count badge
- Per-site toggle (allow list)
- Custom filter list support

### v0.4 — Ghost Mode & Privacy
- Ghost window with isolated `CoreWebView2Environment` (temp profile)
- Visual ghost indicator in toolbar
- Privacy settings: tracking protection levels (Standard/Strict/Custom)
- DNT/GPC headers, referrer trimming, WebRTC leak prevention
- Clear browsing data controls

### v0.5 — Settings, Resources & Layouts
- `cx://settings` page with full sidebar navigation
- GPU acceleration toggle (auto/force/disabled via environment flags)
- Job Object hard memory limits + Resource Monitor thread
- Tab hibernation (auto-suspend inactive tabs)
- Application layouts (Single, Side by Side, Focus, Research, Dashboard)
- Custom layout saving
- Bookmarks, history persistence, downloads handling

---

## Future Considerations
- Extensions / plugin system
- PDF viewer integration
- Reader mode
- Side tab bar option
- Custom new-tab page with bookmarks grid
- Signed auto-update system
- Per-tab bandwidth limiting (metered connections)
- Custom keyboard shortcut remapping
- Sync settings across devices (encrypted, opt-in)
- Built-in VPN / proxy support
