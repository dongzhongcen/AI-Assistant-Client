#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::ffi::c_void;
use std::fs;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, IPersistFile,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetDlgItem, GetMessageW,
    LoadCursorW, MessageBoxW, PostQuitMessage, RegisterClassW, SetWindowTextW,
    ShowWindow, TranslateMessage, CREATESTRUCTW, CW_USEDEFAULT, HMENU, IDC_ARROW, MB_ICONERROR,
    IDYES, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO, MSG, SW_SHOW, WINDOW_EX_STYLE,
    WM_COMMAND, WM_CREATE, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CAPTION,
    WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_CONTROLPARENT, WS_OVERLAPPED,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

const APP_EXE: &[u8] = include_bytes!("../../src-tauri/target/x86_64-pc-windows-msvc/release/ai_assistant_client.exe");
const APP_NAME: &str = "AI Assistant Client";
const APP_EXE_NAME: &str = "AI-Assistant-Client.exe";
const VERSION: &str = "1.0.6";
const ID_INSTALL: isize = 1001;
const ID_LAUNCH: isize = 1002;
const ID_UNINSTALL: isize = 1003;
const ID_CLEAN: isize = 1004;
const ID_CLOSE: isize = 1005;
const ID_STATUS: isize = 1010;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "/?") {
        print_help();
        return;
    }
    if args.iter().any(|arg| arg == "--uninstall") {
        if !confirm("Uninstall AI Assistant Client?\n\nThis removes the app and shortcuts. Local chat data is kept.") {
            return;
        }
        if let Err(error) = uninstall(false) {
            show_error(&format!("Uninstall failed:\n{error}"));
            log_error(&format!("Uninstall failed: {error}"));
            std::process::exit(1);
        }
        show_info("AI Assistant Client has been uninstalled.");
        log_info(&format!("{APP_NAME} uninstalled."));
        return;
    }
    if args.iter().any(|arg| arg == "--uninstall-clean") {
        if !confirm("Clean uninstall AI Assistant Client?\n\nThis removes the app, shortcuts, and local data under %LOCALAPPDATA%\\AI-Assistant-Client.") {
            return;
        }
        if let Err(error) = uninstall(true) {
            show_error(&format!("Clean uninstall failed:\n{error}"));
            log_error(&format!("Clean uninstall failed: {error}"));
            std::process::exit(1);
        }
        show_info("AI Assistant Client and local data have been removed.");
        log_info(&format!("{APP_NAME} uninstalled and data cleaned."));
        return;
    }

    if let Err(error) = run_setup_panel() {
        show_error(&format!("Setup panel failed:\n{error}"));
        log_error(&format!("Install failed: {error}"));
        std::process::exit(1);
    }
}

fn print_help() {
    #[cfg(not(debug_assertions))]
    {
        return;
    }
    #[cfg(debug_assertions)]
    {
    println!("{APP_NAME} Setup {VERSION}");
    println!("Usage:");
    println!("  AI-Assistant-Client-Setup.exe              Install for current user");
    println!("  AI-Assistant-Client-Setup.exe --uninstall  Remove app files and shortcuts");
    println!("  AI-Assistant-Client-Setup.exe --uninstall-clean  Remove app and local data");
    }
}

fn install() -> std::io::Result<()> {
    let install_dir = install_dir();
    fs::create_dir_all(&install_dir)?;

    let app_path = install_dir.join(APP_EXE_NAME);
    write_file(&app_path, APP_EXE)?;

    create_shortcut(
        &start_menu_dir().join(format!("{APP_NAME}.lnk")),
        &app_path,
        &install_dir,
    );
    create_shortcut(
        &desktop_dir().join(format!("{APP_NAME}.lnk")),
        &app_path,
        &install_dir,
    );

    write_uninstall_registry(&install_dir, &app_path);
    Ok(())
}

fn uninstall(clean_data: bool) -> std::io::Result<()> {
    remove_shortcut(&start_menu_dir().join(format!("{APP_NAME}.lnk")));
    remove_shortcut(&desktop_dir().join(format!("{APP_NAME}.lnk")));
    remove_shortcut(&start_menu_dir().join("AI Assistant Client.cmd"));
    remove_shortcut(&desktop_dir().join("AI Assistant Client.cmd"));
    delete_uninstall_registry();

    let install_dir = install_dir();
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }

    if clean_data {
        let data = data_dir();
        if data.exists() {
            fs::remove_dir_all(data)?;
        }
    }

    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn create_shortcut(shortcut: &Path, target: &Path, workdir: &Path) {
    if let Some(parent) = shortcut.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let shortcut_w = wide(shortcut);
    let target_w = wide(target);
    let workdir_w = wide(workdir);

    unsafe {
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return;
        }
        let result = (|| -> windows::core::Result<()> {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
            link.SetPath(PCWSTR(target_w.as_ptr()))?;
            link.SetWorkingDirectory(PCWSTR(workdir_w.as_ptr()))?;
            link.SetIconLocation(PCWSTR(target_w.as_ptr()), 0)?;
            let file: IPersistFile = link.cast()?;
            file.Save(PCWSTR(shortcut_w.as_ptr()), true)?;
            Ok(())
        })();
        let _ = result;
        CoUninitialize();
    }
}

fn remove_shortcut(shortcut: &Path) {
    let _ = fs::remove_file(shortcut);
}

fn write_uninstall_registry(install_dir: &Path, app_path: &Path) {
    let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\AI Assistant Client";
    let setup = env::current_exe().unwrap_or_else(|_| app_path.to_path_buf());
    let uninstall = format!("\"{}\" --uninstall", setup.display());
    let clean_uninstall = format!("\"{}\" --uninstall-clean", setup.display());

    let values = [
        ("DisplayName", APP_NAME),
        ("DisplayVersion", VERSION),
        ("Publisher", "dongzhongcen"),
        ("InstallLocation", &install_dir.display().to_string()),
        ("DisplayIcon", &format!("{},0", app_path.display())),
        ("UninstallString", &uninstall),
        ("QuietUninstallString", &clean_uninstall),
    ];

    for (name, value) in values {
        let _ = Command::new("reg")
            .args(["add", key, "/v", name, "/d", value, "/f"])
            .status();
    }
}

fn delete_uninstall_registry() {
    let _ = Command::new("reg")
        .args([
            "delete",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\AI Assistant Client",
            "/f",
        ])
        .status();
}

fn run_setup_panel() -> windows::core::Result<()> {
    unsafe {
        let module = GetModuleHandleW(None)?;
        let instance = HINSTANCE(module.0);
        let class_name = wide_text("AiAssistantClientSetupPanel");
        let cursor = LoadCursorW(None, IDC_ARROW)?;
        let wc = WNDCLASSW {
            hCursor: cursor,
            hInstance: instance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            lpfnWndProc: Some(setup_wnd_proc),
            ..Default::default()
        };
        RegisterClassW(&wc);

        let title = wide_text("AI Assistant Client Setup");
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            560,
            380,
            None,
            None,
            Some(instance),
            None,
        )?;
        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

extern "system" fn setup_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                create_setup_controls(hwnd, lparam);
                refresh_setup_status(hwnd, "Ready.");
                LRESULT(0)
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xffff) as isize;
                handle_setup_command(hwnd, id);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn create_setup_controls(hwnd: HWND, lparam: LPARAM) {
    let createstruct = lparam.0 as *const CREATESTRUCTW;
    let instance = if createstruct.is_null() {
        HINSTANCE(std::ptr::null_mut())
    } else {
        (*createstruct).hInstance
    };

    create_label(hwnd, instance, 24, 20, 500, 28, "AI Assistant Client");
    create_label(hwnd, instance, 24, 54, 500, 22, &format!("Version {VERSION} · Windows GUI setup panel"));
    create_label(hwnd, instance, 24, 92, 500, 42, &format!("Install path:\r\n{}", install_dir().display()));
    create_label(hwnd, instance, 24, 138, 500, 42, &format!("Data path:\r\n{}", data_dir().display()));

    create_button(hwnd, instance, ID_INSTALL, 24, 204, 116, 38, "Install / Repair");
    create_button(hwnd, instance, ID_LAUNCH, 152, 204, 92, 38, "Launch");
    create_button(hwnd, instance, ID_UNINSTALL, 256, 204, 96, 38, "Uninstall");
    create_button(hwnd, instance, ID_CLEAN, 364, 204, 132, 38, "Clean Uninstall");
    create_button(hwnd, instance, ID_CLOSE, 404, 292, 92, 34, "Close");
    create_label(hwnd, instance, 24, 260, 472, 24, "");
}

unsafe fn create_label(hwnd: HWND, instance: HINSTANCE, x: i32, y: i32, width: i32, height: i32, text: &str) {
    let class = wide_text("STATIC");
    let value = wide_text(text);
    let id = if text.is_empty() { ID_STATUS } else { 0 };
    let _ = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        PCWSTR(class.as_ptr()),
        PCWSTR(value.as_ptr()),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        width,
        height,
        Some(hwnd),
        Some(HMENU(id as *mut c_void)),
        Some(instance),
        None,
    );
}

unsafe fn create_button(
    hwnd: HWND,
    instance: HINSTANCE,
    id: isize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    text: &str,
) {
    let class = wide_text("BUTTON");
    let value = wide_text(text);
    let _ = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        PCWSTR(class.as_ptr()),
        PCWSTR(value.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER,
        x,
        y,
        width,
        height,
        Some(hwnd),
        Some(HMENU(id as *mut c_void)),
        Some(instance),
        None,
    );
}

unsafe fn handle_setup_command(hwnd: HWND, id: isize) {
    match id {
        ID_INSTALL => match install() {
            Ok(()) => refresh_setup_status(hwnd, "Installed. Shortcuts point to the app, not setup."),
            Err(error) => refresh_setup_status(hwnd, &format!("Install failed: {error}")),
        },
        ID_LAUNCH => {
            let app = install_dir().join(APP_EXE_NAME);
            if app.exists() {
                match Command::new(app).spawn() {
                    Ok(_) => refresh_setup_status(hwnd, "Launched AI Assistant Client."),
                    Err(error) => refresh_setup_status(hwnd, &format!("Launch failed: {error}")),
                }
            } else {
                refresh_setup_status(hwnd, "App is not installed yet.");
            }
        }
        ID_UNINSTALL => {
            if confirm("Uninstall AI Assistant Client?\n\nLocal chat data will be kept.") {
                match uninstall(false) {
                    Ok(()) => refresh_setup_status(hwnd, "Uninstalled. Local data was kept."),
                    Err(error) => refresh_setup_status(hwnd, &format!("Uninstall failed: {error}")),
                }
            }
        }
        ID_CLEAN => {
            if confirm("Clean uninstall AI Assistant Client?\n\nThis removes the app and local data.") {
                match uninstall(true) {
                    Ok(()) => refresh_setup_status(hwnd, "Clean uninstall complete. App data removed."),
                    Err(error) => refresh_setup_status(hwnd, &format!("Clean uninstall failed: {error}")),
                }
            }
        }
        ID_CLOSE => {
            let _ = DestroyWindow(hwnd);
        }
        _ => {}
    }
}

unsafe fn refresh_setup_status(hwnd: HWND, text: &str) {
    if let Ok(status) = GetDlgItem(Some(hwnd), ID_STATUS as i32) {
        let value = wide_text(text);
        let _ = SetWindowTextW(status, PCWSTR(value.as_ptr()));
    }
}

fn confirm(message: &str) -> bool {
    message_box(message, MB_YESNO | MB_ICONQUESTION) == IDYES.0
}

fn show_info(message: &str) {
    let _ = message_box(message, MB_OK | MB_ICONINFORMATION);
}

fn show_error(message: &str) {
    let _ = message_box(message, MB_OK | MB_ICONERROR);
}

fn message_box(message: &str, style: windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE) -> i32 {
    let title = wide_text(&format!("{APP_NAME} Setup"));
    let body = wide_text(message);
    unsafe { MessageBoxW(None, PCWSTR(body.as_ptr()), PCWSTR(title.as_ptr()), style).0 }
}

fn install_dir() -> PathBuf {
    local_app_data().join("Programs").join("AI-Assistant-Client")
}

fn data_dir() -> PathBuf {
    local_app_data().join("AI-Assistant-Client")
}

fn start_menu_dir() -> PathBuf {
    env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| local_app_data())
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
}

fn desktop_dir() -> PathBuf {
    env::var("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| local_app_data())
        .join("Desktop")
}

fn local_app_data() -> PathBuf {
    env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|_| env::var("USERPROFILE").map(|home| PathBuf::from(home).join("AppData").join("Local")))
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
}

fn wide_text(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn log_info(_message: &str) {
    #[cfg(debug_assertions)]
    println!("{_message}");
}

fn log_error(_message: &str) {
    #[cfg(debug_assertions)]
    eprintln!("{_message}");
}
