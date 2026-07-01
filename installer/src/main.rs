#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use windows::core::{Interface, PCWSTR};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, IPersistFile,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO,
};

const APP_EXE: &[u8] = include_bytes!("../../src-tauri/target/x86_64-pc-windows-msvc/release/ai_assistant_client.exe");
const APP_NAME: &str = "AI Assistant Client";
const APP_EXE_NAME: &str = "AI-Assistant-Client.exe";
const VERSION: &str = "1.0.6";

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

    let install_dir = install_dir();
    let prompt = if install_dir.join(APP_EXE_NAME).exists() {
        format!(
            "AI Assistant Client is already installed.\n\nUpdate or repair the installation at:\n{}",
            install_dir.display()
        )
    } else {
        format!(
            "Install AI Assistant Client for the current Windows user?\n\nInstall path:\n{}",
            install_dir.display()
        )
    };
    if !confirm(&prompt) {
        return;
    }

    if let Err(error) = install() {
        show_error(&format!("Install failed:\n{error}"));
        log_error(&format!("Install failed: {error}"));
        std::process::exit(1);
    }
    if confirm("Installation complete.\n\nLaunch AI Assistant Client now?") {
        let _ = Command::new(install_dir.join(APP_EXE_NAME)).spawn();
    }
    log_info(&format!("{APP_NAME} installed."));
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
