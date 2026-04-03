#![windows_subsystem = "windows"]

use std::{
    cell::RefCell,
    fmt,
    ptr,
    rc::Rc,
    sync::mpsc,
};

use serde::Deserialize;
use windows::{
    core::*,
    Win32::{
        Foundation::{E_POINTER, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM},
        Graphics::Gdi,
        System::{
            Com::*,
            LibraryLoader,
            WinRT::EventRegistrationToken,
        },
        UI::{
            HiDpi,
            Input::KeyboardAndMouse,
            WindowsAndMessaging::{self, MSG, WNDCLASSW},
        },
    },
};

use webview2_com::{Microsoft::Web::WebView2::Win32::*, *};

// ─── Error Handling ────────────────────────────────────────────────────────
#[derive(Debug)]
enum AppError {
    WebView2(webview2_com::Error),
    Windows(windows::core::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::WebView2(e) => write!(f, "WebView2: {e:?}"),
            AppError::Windows(e) => write!(f, "Windows: {e}"),
        }
    }
}

impl From<webview2_com::Error> for AppError {
    fn from(e: webview2_com::Error) -> Self { AppError::WebView2(e) }
}

impl From<windows::core::Error> for AppError {
    fn from(e: windows::core::Error) -> Self { AppError::Windows(e) }
}

type AppResult<T> = std::result::Result<T, AppError>;

// Height of the UI shell (toolbar + tab bar) in pixels
const UI_HEIGHT: i32 = 80;

// ─── IPC Messages from UI shell JS → Rust ─────────────────────────────────
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum UiMessage {
    #[serde(rename = "navigate")]
    Navigate { url: String },
    #[serde(rename = "back")]
    Back,
    #[serde(rename = "forward")]
    Forward,
    #[serde(rename = "reload")]
    Reload,
    #[serde(rename = "home")]
    Home,
    #[serde(rename = "toggle_theme")]
    ToggleTheme,
}

// ─── App State ─────────────────────────────────────────────────────────────
struct AppState {
    hwnd: HWND,
    size: SIZE,
    ui_controller: Option<ICoreWebView2Controller>,
    ui_webview: Option<ICoreWebView2>,
    content_controller: Option<ICoreWebView2Controller>,
    content_webview: Option<ICoreWebView2>,
    dark_mode: bool,
}

thread_local! {
    static APP: RefCell<Option<Rc<RefCell<AppState>>>> = RefCell::new(None);
}

fn main() -> AppResult<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }
    set_process_dpi_awareness()?;

    let hwnd = create_window();
    let size = get_window_size(hwnd);

    let state = Rc::new(RefCell::new(AppState {
        hwnd,
        size,
        ui_controller: None,
        ui_webview: None,
        content_controller: None,
        content_webview: None,
        dark_mode: true,
    }));

    APP.with(|app| {
        *app.borrow_mut() = Some(state.clone());
    });

    // Create the WebView2 environment
    let env = create_environment()?;

    // Create UI shell webview (top portion of window)
    create_ui_webview(&env, hwnd, &state)?;

    // Create content webview (below the UI shell)
    create_content_webview(&env, hwnd, &state)?;

    // Show window
    unsafe {
        let _ = WindowsAndMessaging::ShowWindow(hwnd, WindowsAndMessaging::SW_SHOW);
        let _ = Gdi::UpdateWindow(hwnd);
        let _ = KeyboardAndMouse::SetFocus(hwnd);
    }

    // Message loop
    let mut msg = MSG::default();
    loop {
        let result = unsafe { WindowsAndMessaging::GetMessageW(&mut msg, HWND::default(), 0, 0).0 };
        match result {
            -1 => break Err(windows::core::Error::from_win32().into()),
            0 => break Ok(()),
            _ => unsafe {
                let _ = WindowsAndMessaging::TranslateMessage(&msg);
                WindowsAndMessaging::DispatchMessageW(&msg);
            },
        }
    }
}

// ─── Window Creation ───────────────────────────────────────────────────────
fn create_window() -> HWND {
    let window_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: w!("CXBrowser"),
        hCursor: unsafe {
            WindowsAndMessaging::LoadCursorW(None, WindowsAndMessaging::IDC_ARROW).unwrap_or_default()
        },
        hbrBackground: unsafe { Gdi::CreateSolidBrush(windows::Win32::Foundation::COLORREF(0x0023190F)) }, // #0F1923 in BGR
        ..Default::default()
    };

    unsafe {
        WindowsAndMessaging::RegisterClassW(&window_class);
        WindowsAndMessaging::CreateWindowExW(
            Default::default(),
            w!("CXBrowser"),
            w!("CX Browser"),
            WindowsAndMessaging::WS_OVERLAPPEDWINDOW,
            WindowsAndMessaging::CW_USEDEFAULT,
            WindowsAndMessaging::CW_USEDEFAULT,
            1280,
            800,
            None,
            None,
            LibraryLoader::GetModuleHandleW(None).unwrap_or_default(),
            None,
        )
        .unwrap_or_default()
    }
}

// ─── WebView2 Environment ──────────────────────────────────────────────────
fn create_environment() -> webview2_com::Result<ICoreWebView2Environment> {
    let (tx, rx) = mpsc::channel();

    CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
        Box::new(|handler| unsafe {
            CreateCoreWebView2Environment(&handler)
                .map_err(webview2_com::Error::WindowsError)
        }),
        Box::new(move |error_code, environment| {
            error_code?;
            tx.send(environment.ok_or_else(|| windows::core::Error::from(E_POINTER)))
                .expect("send over mpsc channel");
            Ok(())
        }),
    )?;

    rx.recv()
        .map_err(|_| webview2_com::Error::SendError)?
        .map_err(webview2_com::Error::WindowsError)
}

// ─── UI Shell WebView ──────────────────────────────────────────────────────
fn create_ui_webview(
    env: &ICoreWebView2Environment,
    hwnd: HWND,
    state: &Rc<RefCell<AppState>>,
) -> webview2_com::Result<()> {
    let (tx, rx) = mpsc::channel();
    let env = env.clone();

    CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
        Box::new(move |handler| unsafe {
            env.CreateCoreWebView2Controller(hwnd, &handler)
                .map_err(webview2_com::Error::WindowsError)
        }),
        Box::new(move |error_code, controller| {
            error_code?;
            tx.send(controller.ok_or_else(|| windows::core::Error::from(E_POINTER)))
                .expect("send over mpsc channel");
            Ok(())
        }),
    )?;

    let controller = rx
        .recv()
        .map_err(|_| webview2_com::Error::SendError)?
        .map_err(webview2_com::Error::WindowsError)?;

    let size = get_window_size(hwnd);
    unsafe {
        controller.SetBounds(RECT {
            left: 0,
            top: 0,
            right: size.cx,
            bottom: UI_HEIGHT,
        })?;
        controller.SetIsVisible(true)?;
    }

    let webview = unsafe { controller.CoreWebView2()? };

    // Disable context menus and devtools in the UI shell
    unsafe {
        let settings = webview.Settings()?;
        settings.SetAreDefaultContextMenusEnabled(false)?;
        settings.SetAreDevToolsEnabled(false)?;
        settings.SetIsStatusBarEnabled(false)?;
        settings.SetIsZoomControlEnabled(false)?;
    }

    // Set up IPC handler for messages from UI shell JS
    let state_clone = state.clone();
    unsafe {
        let mut token = EventRegistrationToken::default();
        webview.add_WebMessageReceived(
            &WebMessageReceivedEventHandler::create(Box::new(move |_wv, args| {
                if let Some(args) = args {
                    let mut message = PWSTR(ptr::null_mut());
                    if args.WebMessageAsJson(&mut message).is_ok() {
                        let message = CoTaskMemPWSTR::from(message);
                        let msg_str = message.to_string();
                        if let Ok(msg) = serde_json::from_str::<UiMessage>(&msg_str) {
                            handle_ui_message(&state_clone, msg);
                        }
                    }
                }
                Ok(())
            })),
            &mut token,
        )?;
    }

    // Load the UI HTML
    let ui_html = include_str!("ui.html");
    unsafe {
        let html = CoTaskMemPWSTR::from(ui_html);
        webview.NavigateToString(*html.as_ref().as_pcwstr())?;
    }

    // Store in state
    {
        let mut s = state.borrow_mut();
        s.ui_controller = Some(controller);
        s.ui_webview = Some(webview);
    }

    Ok(())
}

// ─── Content WebView ───────────────────────────────────────────────────────
fn create_content_webview(
    env: &ICoreWebView2Environment,
    hwnd: HWND,
    state: &Rc<RefCell<AppState>>,
) -> webview2_com::Result<()> {
    let (tx, rx) = mpsc::channel();
    let env = env.clone();

    CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
        Box::new(move |handler| unsafe {
            env.CreateCoreWebView2Controller(hwnd, &handler)
                .map_err(webview2_com::Error::WindowsError)
        }),
        Box::new(move |error_code, controller| {
            error_code?;
            tx.send(controller.ok_or_else(|| windows::core::Error::from(E_POINTER)))
                .expect("send over mpsc channel");
            Ok(())
        }),
    )?;

    let controller = rx
        .recv()
        .map_err(|_| webview2_com::Error::SendError)?
        .map_err(webview2_com::Error::WindowsError)?;

    let size = get_window_size(hwnd);
    unsafe {
        controller.SetBounds(RECT {
            left: 0,
            top: UI_HEIGHT,
            right: size.cx,
            bottom: size.cy,
        })?;
        controller.SetIsVisible(true)?;
    }

    let webview = unsafe { controller.CoreWebView2()? };

    // Set up navigation events to update URL bar in UI shell
    let state_clone = state.clone();
    unsafe {
        let mut token = EventRegistrationToken::default();
        webview.add_NavigationStarting(
            &NavigationStartingEventHandler::create(Box::new(move |_wv, args| {
                if let Some(args) = args {
                    let mut uri = PWSTR(ptr::null_mut());
                    if args.Uri(&mut uri).is_ok() {
                        let uri = CoTaskMemPWSTR::from(uri);
                        let uri_str = uri.to_string();
                        update_ui_url(&state_clone, &uri_str);
                        update_ui_status(&state_clone, &format!("Loading {}...", &uri_str));
                    }
                }
                Ok(())
            })),
            &mut token,
        )?;
    }

    let state_clone = state.clone();
    unsafe {
        let mut token = EventRegistrationToken::default();
        webview.add_NavigationCompleted(
            &NavigationCompletedEventHandler::create(Box::new(move |_wv, _args| {
                update_ui_status(&state_clone, "Ready");
                Ok(())
            })),
            &mut token,
        )?;
    }

    // Update page title on document title change
    let state_clone = state.clone();
    unsafe {
        let mut token = EventRegistrationToken::default();
        webview.add_DocumentTitleChanged(
            &DocumentTitleChangedEventHandler::create(Box::new(move |wv, _args| {
                if let Some(wv) = wv {
                    let mut title = PWSTR(ptr::null_mut());
                    if wv.DocumentTitle(&mut title).is_ok() {
                        let title = CoTaskMemPWSTR::from(title);
                        let title_str = title.to_string();
                        update_window_title(&state_clone, &title_str);
                    }
                }
                Ok(())
            })),
            &mut token,
        )?;
    }

    // Navigate to new tab page
    let new_tab_html = new_tab_page_html(true);
    unsafe {
        let html = CoTaskMemPWSTR::from(new_tab_html.as_str());
        webview.NavigateToString(*html.as_ref().as_pcwstr())?;
    }

    // Store in state
    {
        let mut s = state.borrow_mut();
        s.content_controller = Some(controller);
        s.content_webview = Some(webview);
    }

    Ok(())
}

// ─── IPC Message Handler ───────────────────────────────────────────────────
fn handle_ui_message(state: &Rc<RefCell<AppState>>, msg: UiMessage) {
    let s = state.borrow();
    match msg {
        UiMessage::Navigate { url } => {
            if let Some(ref wv) = s.content_webview {
                // Security: block javascript: and data: URIs
                let url_lower = url.to_lowercase();
                if url_lower.starts_with("javascript:") || url_lower.starts_with("data:") {
                    drop(s);
                    update_ui_status(state, "Blocked: unsafe URL scheme");
                    return;
                }
                unsafe {
                    let url = CoTaskMemPWSTR::from(url.as_str());
                    let _ = wv.Navigate(*url.as_ref().as_pcwstr());
                }
            }
        }
        UiMessage::Back => {
            if let Some(ref wv) = s.content_webview {
                unsafe {
                    let _ = wv.GoBack();
                }
            }
        }
        UiMessage::Forward => {
            if let Some(ref wv) = s.content_webview {
                unsafe {
                    let _ = wv.GoForward();
                }
            }
        }
        UiMessage::Reload => {
            if let Some(ref wv) = s.content_webview {
                unsafe {
                    let _ = wv.Reload();
                }
            }
        }
        UiMessage::Home => {
            if let Some(ref wv) = s.content_webview {
                let dark = s.dark_mode;
                let html = new_tab_page_html(dark);
                unsafe {
                    let html = CoTaskMemPWSTR::from(html.as_str());
                    let _ = wv.NavigateToString(*html.as_ref().as_pcwstr());
                }
            }
        }
        UiMessage::ToggleTheme => {
            drop(s);
            let mut s = state.borrow_mut();
            s.dark_mode = !s.dark_mode;
            let dark = s.dark_mode;
            // Update UI shell theme
            if let Some(ref wv) = s.ui_webview {
                let js = format!("window.setTheme({})", dark);
                unsafe {
                    let js_pwstr = CoTaskMemPWSTR::from(js.as_str());
                    let _ = wv.ExecuteScript(
                        *js_pwstr.as_ref().as_pcwstr(),
                        &ExecuteScriptCompletedHandler::create(Box::new(|_, _| Ok(()))),
                    );
                }
            }
        }
    }
}

// ─── Helper: Update UI shell URL bar ───────────────────────────────────────
fn update_ui_url(state: &Rc<RefCell<AppState>>, url: &str) {
    let s = state.borrow();
    if let Some(ref wv) = s.ui_webview {
        let escaped = url.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!("window.updateUrl('{}')", escaped);
        unsafe {
            let js_pwstr = CoTaskMemPWSTR::from(js.as_str());
            let _ = wv.ExecuteScript(
                *js_pwstr.as_ref().as_pcwstr(),
                &ExecuteScriptCompletedHandler::create(Box::new(|_, _| Ok(()))),
            );
        }
    }
}

fn update_ui_status(state: &Rc<RefCell<AppState>>, status: &str) {
    let s = state.borrow();
    if let Some(ref wv) = s.ui_webview {
        let escaped = status.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!("window.updateStatus('{}')", escaped);
        unsafe {
            let js_pwstr = CoTaskMemPWSTR::from(js.as_str());
            let _ = wv.ExecuteScript(
                *js_pwstr.as_ref().as_pcwstr(),
                &ExecuteScriptCompletedHandler::create(Box::new(|_, _| Ok(()))),
            );
        }
    }
}

fn update_window_title(state: &Rc<RefCell<AppState>>, title: &str) {
    let s = state.borrow();
    let display_title = if title.is_empty() {
        "CX Browser".to_string()
    } else {
        format!("{} — CX Browser", title)
    };
    let title_pwstr = CoTaskMemPWSTR::from(display_title.as_str());
    unsafe {
        let _ = WindowsAndMessaging::SetWindowTextW(s.hwnd, *title_pwstr.as_ref().as_pcwstr());
    }
}

// ─── New Tab Page ──────────────────────────────────────────────────────────
fn new_tab_page_html(dark: bool) -> String {
    let (bg, text, sub) = if dark {
        ("#0F1923", "#D4DDE8", "#7A8FA6")
    } else {
        ("#F5F5F7", "#1E1E2E", "#5C5C72")
    };

    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>
body {{
    margin: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: {bg};
    color: {text};
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}}
h1 {{ font-size: 48px; font-weight: 300; margin: 0; }}
p {{ color: {sub}; font-size: 14px; margin-top: 8px; }}
</style></head><body>
<h1>CX Browser</h1>
<p>Start typing in the address bar to search or enter a URL</p>
</body></html>"#
    )
}

// ─── Window Proc ───────────────────────────────────────────────────────────
extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WindowsAndMessaging::WM_SIZE => {
            APP.with(|app| {
                if let Some(ref state) = *app.borrow() {
                    let mut s = state.borrow_mut();
                    let size = get_window_size(hwnd);
                    s.size = size;

                    // Resize UI shell
                    if let Some(ref ctrl) = s.ui_controller {
                        unsafe {
                            let _ = ctrl.SetBounds(RECT {
                                left: 0,
                                top: 0,
                                right: size.cx,
                                bottom: UI_HEIGHT,
                            });
                        }
                    }

                    // Resize content area
                    if let Some(ref ctrl) = s.content_controller {
                        unsafe {
                            let _ = ctrl.SetBounds(RECT {
                                left: 0,
                                top: UI_HEIGHT,
                                right: size.cx,
                                bottom: size.cy,
                            });
                        }
                    }
                }
            });
            LRESULT::default()
        }

        WindowsAndMessaging::WM_CLOSE => {
            unsafe {
                let _ = WindowsAndMessaging::DestroyWindow(hwnd);
            }
            LRESULT::default()
        }

        WindowsAndMessaging::WM_DESTROY => {
            APP.with(|app| {
                if let Some(state) = app.borrow_mut().take() {
                    let mut s = state.borrow_mut();
                    // Close controllers to release WebView2
                    if let Some(ref ctrl) = s.content_controller {
                        unsafe { let _ = ctrl.Close(); }
                    }
                    if let Some(ref ctrl) = s.ui_controller {
                        unsafe { let _ = ctrl.Close(); }
                    }
                    s.ui_controller = None;
                    s.ui_webview = None;
                    s.content_controller = None;
                    s.content_webview = None;
                }
            });
            unsafe {
                WindowsAndMessaging::PostQuitMessage(0);
            }
            LRESULT::default()
        }

        _ => unsafe { WindowsAndMessaging::DefWindowProcW(hwnd, msg, w_param, l_param) },
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────
fn get_window_size(hwnd: HWND) -> SIZE {
    let mut rect = RECT::default();
    unsafe { let _ = WindowsAndMessaging::GetClientRect(hwnd, &mut rect); }
    SIZE {
        cx: rect.right - rect.left,
        cy: rect.bottom - rect.top,
    }
}

fn set_process_dpi_awareness() -> AppResult<()> {
    unsafe { HiDpi::SetProcessDpiAwareness(HiDpi::PROCESS_PER_MONITOR_DPI_AWARE)? };
    Ok(())
}
