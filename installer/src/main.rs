#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const APP_EXE: &[u8] = include_bytes!("../../src-tauri/target/x86_64-pc-windows-msvc/release/ai_assistant_client.exe");
const APP_NAME: &str = "AI Assistant Client";
const APP_EXE_NAME: &str = "AI-Assistant-Client.exe";
const VERSION: &str = "1.0.5";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "/?") {
        print_help();
        return;
    }
    if args.iter().any(|arg| arg == "--uninstall") {
        if let Err(error) = uninstall(false) {
            eprintln!("Uninstall failed: {error}");
            std::process::exit(1);
        }
        println!("{APP_NAME} uninstalled.");
        return;
    }
    if args.iter().any(|arg| arg == "--uninstall-clean") {
        if let Err(error) = uninstall(true) {
            eprintln!("Clean uninstall failed: {error}");
            std::process::exit(1);
        }
        println!("{APP_NAME} uninstalled and data cleaned.");
        return;
    }

    if let Err(error) = install() {
        eprintln!("Install failed: {error}");
        std::process::exit(1);
    }
    println!("{APP_NAME} installed.");
}

fn print_help() {
    println!("{APP_NAME} Setup {VERSION}");
    println!("Usage:");
    println!("  AI-Assistant-Client-Setup.exe              Install for current user");
    println!("  AI-Assistant-Client-Setup.exe --uninstall  Remove app files and shortcuts");
    println!("  AI-Assistant-Client-Setup.exe --uninstall-clean  Remove app and local data");
}

fn install() -> std::io::Result<()> {
    let install_dir = install_dir();
    fs::create_dir_all(&install_dir)?;

    let app_path = install_dir.join(APP_EXE_NAME);
    write_file(&app_path, APP_EXE)?;

    let uninstall_cmd = install_dir.join("Uninstall-AI-Assistant-Client.cmd");
    write_text(
        &uninstall_cmd,
        &format!(
            "@echo off\r\n\"{}\" --uninstall-clean\r\npause\r\n",
            env::current_exe()?.display()
        ),
    )?;

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

fn write_text(path: &Path, text: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

fn create_shortcut(shortcut: &Path, target: &Path, workdir: &Path) {
    if let Some(parent) = shortcut.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let script = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.WorkingDirectory='{}';$s.IconLocation='{},0';$s.Save()",
        ps_escape(&shortcut.display().to_string()),
        ps_escape(&target.display().to_string()),
        ps_escape(&workdir.display().to_string()),
        ps_escape(&target.display().to_string()),
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .status();
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

fn ps_escape(value: &str) -> String {
    value.replace('\'', "''")
}
