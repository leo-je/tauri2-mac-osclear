use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    ActivationPolicy, AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;
use walkdir::WalkDir;

pub struct PasswordState {
    password: Mutex<Option<String>>,
}

impl PasswordState {
    pub fn new() -> Self {
        PasswordState {
            password: Mutex::new(None),
        }
    }
}

fn start_system_monitor(app_handle: AppHandle, tray: TrayIcon) {
    thread::spawn(move || {
        loop {
            if let Ok(status) = get_system_status_internal() {
                let _ = app_handle.emit("system-update", status.clone());
                let text = format!("c {:>3}%  m {:>3}%", status.cpu_usage as u32, status.memory_usage as u32);
                let _ = tray.set_title(Some(text));
                let _ = tray.set_tooltip(Some(format!("cpu: {:.0}%\nmem: {:.0}%", status.cpu_usage, status.memory_usage)));
            }
            if let Ok(memory_info) = get_memory_info_internal() {
                let _ = app_handle.emit("memory-update", memory_info);
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

#[derive(Debug, Serialize, Clone)]
pub struct MemoryInfo {
    total: u64,
    used: u64,
    available: u64,
    free: u64,
    reclaimable: u64,
    usage: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemStatus {
    cpu_usage: f32,
    memory_usage: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProcessMemoryInfo {
    pid: u32,
    name: String,
    memory_bytes: u64,
    cpu_usage: f32,
}

#[derive(Debug, Serialize)]
pub struct JunkItem {
    path: String,
    size: u64,
    category: String,
}

#[derive(Debug, Serialize)]
pub struct JunkScanResult {
    items: Vec<JunkItem>,
    total_size: u64,
    categories: Vec<(String, u64)>,
}

#[derive(Debug, Deserialize)]
pub struct ScanJunkRequest {
    target_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CleanRequest {
    paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CleanResult {
    cleaned_count: usize,
    cleaned_size: u64,
    errors: Vec<String>,
}

struct JunkScanTargetSpec {
    id: &'static str,
    category: &'static str,
    path: PathBuf,
}

#[tauri::command]
async fn get_memory_info() -> Result<MemoryInfo, String> {
    get_memory_info_internal()
}

#[tauri::command]
async fn get_process_memory_list() -> Result<Vec<ProcessMemoryInfo>, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes: Vec<ProcessMemoryInfo> = sys
        .processes()
        .iter()
        .filter(|(_, process)| process.memory() > 1024 * 1024)
        .map(|(pid, process)| ProcessMemoryInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            memory_bytes: process.memory(),
            cpu_usage: process.cpu_usage(),
        })
        .collect();

    processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));

    Ok(processes)
}

#[derive(Debug, Serialize)]
pub struct FreeMemoryResult {
    freed_bytes: u64,
    before_usage: f64,
    after_usage: f64,
}

#[tauri::command]
async fn setup_passwordless_purge() -> Result<String, String> {
    let username = Command::new("whoami")
        .output()
        .map_err(|e| format!("获取用户名失败: {}", e))?;

    let username = String::from_utf8_lossy(&username.stdout).trim().to_string();

    let sudoers_entry = format!("{} ALL=(ALL) NOPASSWD: /usr/sbin/purge", username);

    let script = format!(
        r#"do shell script "echo '{}' >> /etc/sudoers.d/macos-cleaner""#,
        sudoers_entry.replace("'", "'\\''")
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("配置失败: {}", e))?;

    if output.status.success() {
        Ok("已配置免密清理内存".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("User canceled") {
            return Err("用户取消了授权".to_string());
        }
        Err(format!("配置失败: {}", stderr))
    }
}

#[tauri::command]
async fn free_memory(password_state: State<'_, PasswordState>) -> Result<FreeMemoryResult, String> {
    let before = get_memory_info_internal()?;

    let mut success = false;

    let no_password_output = Command::new("sudo")
        .args(["-n", "purge"])
        .output();

    if let Ok(ref output) = no_password_output {
        if output.status.success() {
            success = true;
        }
    }

    if !success {
        let cached_password = {
            let guard = password_state.password.lock().unwrap();
            guard.clone()
        };

        if let Some(ref password) = cached_password {
            let sudo_output = Command::new("sudo")
                .args(["-S", "purge"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(format!("{}\n", password).as_bytes());
                    }
                    child.wait_with_output()
                });

            if let Ok(ref output) = sudo_output {
                if output.status.success() {
                    success = true;
                }
            }
        }
    }

    if !success {
        let script = r#"
            try
                set result to display dialog "请输入管理员密码以清理内存:" default answer "" with hidden answer with title "macOS Cleaner"
                return text returned of result
            on error errMsg
                if errMsg contains "User canceled" or errMsg contains "-128" then
                    return "canceled"
                end if
                return "error: " & errMsg
            end try
        "#;

        let password_output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| format!("获取密码失败: {}", e))?;

        let password_result = String::from_utf8_lossy(&password_output.stdout).trim().to_string();

        if password_result == "canceled" {
            return Err("用户取消了授权".to_string());
        }

        if password_result.starts_with("error:") {
            return Err(format!("获取密码失败: {}", password_result));
        }

        let sudo_output = Command::new("sudo")
            .args(["-S", "purge"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(format!("{}\n", password_result).as_bytes());
                }
                child.wait_with_output()
            });

        match sudo_output {
            Ok(output) if output.status.success() => {
                success = true;
                let mut guard = password_state.password.lock().unwrap();
                *guard = Some(password_result);
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("密码错误或清理失败: {}", stderr));
            }
            Err(e) => {
                return Err(format!("执行清理失败: {}", e));
            }
        }
    }

    if !success {
        return Err("清理失败".to_string());
    }

    thread::sleep(Duration::from_millis(500));

    let after = get_memory_info_internal()?;

    let freed_bytes = if after.used < before.used {
        before.used - after.used
    } else {
        0
    };

    Ok(FreeMemoryResult {
        freed_bytes,
        before_usage: before.usage,
        after_usage: after.usage,
    })
}

fn get_memory_info_internal() -> Result<MemoryInfo, String> {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = sys.available_memory();
    let free = sys.free_memory();
    let reclaimable = available.saturating_sub(free);

    let usage = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(MemoryInfo {
        total,
        used,
        available,
        free,
        reclaimable,
        usage,
    })
}

fn get_system_status_internal() -> Result<SystemStatus, String> {
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    thread::sleep(Duration::from_millis(200));
    sys.refresh_cpu_usage();

    let cpu_usage = sys.global_cpu_usage();

    let memory_info = get_memory_info_internal()?;

    Ok(SystemStatus {
        cpu_usage,
        memory_usage: memory_info.usage,
    })
}

#[tauri::command]
async fn scan_junk_files(request: Option<ScanJunkRequest>) -> Result<JunkScanResult, String> {
    let mut items: Vec<JunkItem> = Vec::new();
    let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;

    let selected_target_ids = request
        .map(|scan_request| scan_request.target_ids)
        .unwrap_or_default();

    let junk_paths = get_selected_junk_scan_targets(&home, &selected_target_ids);

    for target in junk_paths {
        let path = target.path;
        if !path.exists() {
            continue;
        }

        if path.is_dir() {
            for entry in WalkDir::new(&path)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let metadata = entry.metadata();
                if let Ok(meta) = metadata {
                    let size = meta.len();
                    if size > 1024 * 1024 {
                        items.push(JunkItem {
                            path: entry.path().to_string_lossy().to_string(),
                            size,
                            category: target.category.to_string(),
                        });
                    }
                }
            }
        } else if path.is_file() {
            let metadata = path.metadata();
            if let Ok(meta) = metadata {
                items.push(JunkItem {
                    path: path.to_string_lossy().to_string(),
                    size: meta.len(),
                    category: target.category.to_string(),
                });
            }
        }
    }

    let mut category_sizes: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    let mut total_size = 0u64;

    for item in &items {
        total_size += item.size;
        *category_sizes.entry(item.category.clone()).or_insert(0) += item.size;
    }

    let mut categories: Vec<(String, u64)> = category_sizes.into_iter().collect();
    categories.sort_by(|a, b| b.1.cmp(&a.1));

    items.sort_by(|a, b| b.size.cmp(&a.size));
    items.truncate(100);

    Ok(JunkScanResult {
        items,
        total_size,
        categories,
    })
}

fn get_selected_junk_scan_targets(home: &str, selected_ids: &[String]) -> Vec<JunkScanTargetSpec> {
    let all_targets = vec![
        JunkScanTargetSpec {
            id: "user_cache",
            category: "用户缓存",
            path: PathBuf::from(format!("{}/Library/Caches", home)),
        },
        JunkScanTargetSpec {
            id: "logs",
            category: "日志文件",
            path: PathBuf::from(format!("{}/Library/Logs", home)),
        },
        JunkScanTargetSpec {
            id: "tmp",
            category: "临时文件",
            path: PathBuf::from("/tmp"),
        },
        JunkScanTargetSpec {
            id: "system_cache",
            category: "系统缓存",
            path: PathBuf::from("/Library/Caches"),
        },
        JunkScanTargetSpec {
            id: "xcode_derived_data",
            category: "Xcode 派生数据",
            path: PathBuf::from(format!("{}/Library/Developer/Xcode/DerivedData", home)),
        },
        JunkScanTargetSpec {
            id: "application_support",
            category: "应用支持文件",
            path: PathBuf::from(format!("{}/Library/Application Support", home)),
        },
        JunkScanTargetSpec {
            id: "downloads",
            category: "下载文件",
            path: PathBuf::from(format!("{}/Downloads", home)),
        },
        JunkScanTargetSpec {
            id: "trash",
            category: "废纸篓",
            path: PathBuf::from(format!("{}/.Trash", home)),
        },
    ];

    if selected_ids.is_empty() {
        return all_targets;
    }

    all_targets
        .into_iter()
        .filter(|target| selected_ids.iter().any(|selected_id| selected_id == target.id))
        .collect()
}

#[tauri::command]
async fn clean_junk_files(request: CleanRequest) -> Result<CleanResult, String> {
    let mut cleaned_count = 0;
    let mut cleaned_size = 0u64;
    let mut errors = Vec::new();

    for path_str in request.paths {
        let path = PathBuf::from(&path_str);
        if !path.exists() {
            errors.push(format!("Path does not exist: {}", path_str));
            continue;
        }

        let size = if path.is_file() {
            path.metadata().map(|m| m.len()).unwrap_or(0)
        } else {
            let mut dir_size = 0u64;
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                if let Ok(meta) = entry.metadata() {
                    dir_size += meta.len();
                }
            }
            dir_size
        };

        match move_to_trash(&path) {
            Ok(_) => {
                cleaned_count += 1;
                cleaned_size += size;
            }
            Err(e) => {
                errors.push(format!("Failed to move {} to trash: {}", path_str, e));
            }
        }
    }

    Ok(CleanResult {
        cleaned_count,
        cleaned_size,
        errors,
    })
}

fn move_to_trash(path: &PathBuf) -> Result<(), String> {
    let path_str = path.to_string_lossy();

    let script = format!(
        "tell application \"Finder\" to delete POSIX file \"{}\"",
        path_str.replace("\"", "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to execute osascript: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("AppleScript failed: {}", stderr))
    }
}

#[tauri::command]
async fn get_system_info() -> Result<serde_json::Value, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let info = serde_json::json!({
        "os_name": System::name().unwrap_or_else(|| "Unknown".to_string()),
        "os_version": System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        "hostname": System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        "cpu_count": sys.cpus().len(),
        "total_memory": sys.total_memory(),
    });

    Ok(info)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(PasswordState::new())
        .setup(|app| {
            let app_handle = app.handle().clone();

            let clean_item = MenuItem::with_id(app, "clean", "清理内存", true, None::<&str>)?;
            let show_item = MenuItem::with_id(app, "show_main", "显示主界面", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&clean_item, &show_item, &quit_item])?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .build(app)?;

            start_system_monitor(app_handle.clone(), tray);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                #[cfg(target_os = "macos")]
                {
                    let _ = window.app_handle().set_dock_visibility(false);
                    let _ = window.app_handle().set_activation_policy(ActivationPolicy::Accessory);
                }
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "clean" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let password_state = app_handle.state::<PasswordState>();
                    match free_memory(password_state).await {
                        Ok(result) => {
                            let msg = if result.freed_bytes > 0 {
                                format!("已释放 {} 内存", format_size_display(result.freed_bytes))
                            } else {
                                "当前内存状态良好，无需清理".to_string()
                            };
                            match app_handle.notification()
                                .builder()
                                .title("macOS Cleaner")
                                .body(&msg)
                                .show() {
                                Ok(_) => println!("通知发送成功"),
                                Err(e) => println!("通知发送失败: {:?}", e),
                            }
                        }
                        Err(e) => {
                            if e != "用户取消了授权" {
                                match app_handle.notification()
                                    .builder()
                                    .title("macOS Cleaner")
                                    .body(&e)
                                    .show() {
                                    Ok(_) => println!("通知发送成功"),
                                    Err(e) => println!("通知发送失败: {:?}", e),
                                }
                            }
                        }
                    }
                });
            }
            "show_main" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                    #[cfg(target_os = "macos")]
                    {
                        let _ = app.set_dock_visibility(true);
                        let _ = app.set_activation_policy(ActivationPolicy::Regular);
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_memory_info,
            get_process_memory_list,
            free_memory,
            setup_passwordless_purge,
            scan_junk_files,
            clean_junk_files,
            get_system_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn format_size_display(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log(k).floor() as usize;
    let size = bytes as f64 / k.powi(i as i32);
    format!("{:.2} {}", size, sizes[i.min(4)])
}
