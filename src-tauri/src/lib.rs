use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Emitter, Manager,
};
use walkdir::WalkDir;

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

#[derive(Debug, Serialize)]
pub struct FreeMemoryResult {
    freed_bytes: u64,
    before_usage: f64,
    after_usage: f64,
}

#[tauri::command]
async fn free_memory() -> Result<FreeMemoryResult, String> {
    let before = get_memory_info_internal()?;
    
    let script = "do shell script \"purge\" with administrator privileges";
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("执行清理失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("User canceled") || stderr.contains("-128") {
            return Err("用户取消了授权".to_string());
        }
        return Err(format!("清理失败: {}", stderr));
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

fn start_system_monitor(app_handle: AppHandle, tray: TrayIcon) {
    thread::spawn(move || loop {
        if let Ok(status) = get_system_status_internal() {
            let _ = app_handle.emit("system-update", status.clone());
            let text = format!("CPU: {:.0}%\nMEM: {:.0}%", status.cpu_usage, status.memory_usage);
            let _ = tray.set_title(Some(text));
        }
        if let Ok(memory_info) = get_memory_info_internal() {
            let _ = app_handle.emit("memory-update", memory_info);
        }
        thread::sleep(Duration::from_secs(1));
    });
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
        .setup(|app| {
            let app_handle = app.handle().clone();

            let clean_item = MenuItem::with_id(app, "clean", "清理内存", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&clean_item, &quit_item])?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .title("CPU: 0%\nMEM: 0%")
                .build(app)?;

            start_system_monitor(app_handle, tray);

            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "clean" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    if let Ok(result) = free_memory().await {
                        let msg = if result.freed_bytes > 0 {
                            format!("已释放 {} 内存", format_size_display(result.freed_bytes))
                        } else {
                            "当前内存状态良好，无需清理".to_string()
                        };
                        let _ = app_handle.emit("tray-clean-result", msg);
                    }
                });
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_memory_info,
            free_memory,
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
