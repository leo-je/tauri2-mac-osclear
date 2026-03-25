use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Serialize, Clone)]
pub struct UninstalledAppJunkItem {
    path: String,
    size: u64,
    category: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct UninstalledAppJunkApp {
    app_id: String,
    app_name: String,
    identifier: Option<String>,
    total_size: u64,
    items: Vec<UninstalledAppJunkItem>,
}

#[derive(Debug, Serialize)]
pub struct UninstalledAppJunkScanResult {
    apps: Vec<UninstalledAppJunkApp>,
    total_size: u64,
    total_items: usize,
}

#[derive(Debug, Deserialize)]
pub struct ScanJunkRequest {
    target_ids: Vec<String>,
    downloads_min_age_days: Option<u64>,
    downloads_min_size_mb: Option<u64>,
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
    is_downloads: bool,
}

struct InstalledAppsIndex {
    bundle_ids: HashSet<String>,
    normalized_names: HashSet<String>,
    apps: Vec<InstalledAppRecord>,
}

#[derive(Clone)]
struct InstalledAppRecord {
    normalized_name: String,
    base_normalized_name: String,
    version_key: Option<String>,
}

struct AppResidualCandidate {
    app_id: String,
    app_name: String,
    identifier: Option<String>,
    source_key: String,
    item: UninstalledAppJunkItem,
    is_old_version: bool,
}

struct AppResidualGroupAccumulator {
    app_id: String,
    app_name: String,
    identifier: Option<String>,
    total_size: u64,
    source_keys: HashSet<String>,
    items: Vec<UninstalledAppJunkItem>,
    has_old_version_items: bool,
}

const MIN_APP_RESIDUAL_ITEM_SIZE: u64 = 16 * 1024;
const MIN_APP_RESIDUAL_GROUP_SIZE: u64 = 2 * 1024 * 1024;

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

fn normalize_app_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn normalize_version_key(value: &str) -> Option<String> {
    let normalized: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect();

    if normalized.chars().any(|ch| ch.is_ascii_digit()) {
        Some(normalized)
    } else {
        None
    }
}

fn is_version_like(value: &str) -> bool {
    !value.is_empty()
        && value.chars().any(|ch| ch.is_ascii_digit())
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '(' | ')'))
}

fn extract_name_and_version(value: &str) -> (String, Option<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }

    for separator in [' ', '-', '_', '('] {
        if let Some(index) = trimmed.rfind(separator) {
            let suffix = trimmed[index + 1..].trim_matches(|ch: char| {
                ch.is_ascii_whitespace() || matches!(ch, '-' | '_' | '(' | ')')
            });
            if is_version_like(suffix) {
                let base = trimmed[..index].trim().trim_end_matches('(').trim();
                return (base.to_string(), normalize_version_key(suffix));
            }
        }
    }

    (trimmed.to_string(), None)
}

fn is_bundle_identifier(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').filter(|part| !part.is_empty()).collect();
    if parts.len() < 2 {
        return false;
    }

    parts.iter().all(|part| {
        part.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    })
}

fn prettify_label(value: &str) -> String {
    let cleaned = value
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.is_empty() {
        value.to_string()
    } else {
        cleaned
    }
}

fn guess_app_name_from_identifier(identifier: &str) -> String {
    let last_segment = identifier.rsplit('.').next().unwrap_or(identifier);
    prettify_label(last_segment)
}

fn get_path_size(path: &Path) -> u64 {
    if path.is_file() {
        return path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
    }

    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .map(|metadata| metadata.len())
        .sum()
}

fn collect_installed_apps_from_root(root: &Path, index: &mut InstalledAppsIndex) {
    if !root.exists() {
        return;
    }

    for entry in WalkDir::new(root)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        let is_app_bundle = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("app"))
            .unwrap_or(false);

        if !is_app_bundle {
            continue;
        }

        let fallback_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();

        let mut app_name = fallback_name.clone();
        let mut version_key = None;

        if !fallback_name.is_empty() {
            index
                .normalized_names
                .insert(normalize_app_name(&fallback_name));
        }

        let info_plist_path = path.join("Contents/Info.plist");
        if let Ok(plist::Value::Dictionary(dict)) = plist::Value::from_file(&info_plist_path) {
            if let Some(bundle_identifier) = dict
                .get("CFBundleIdentifier")
                .and_then(|value| value.as_string())
                .map(|value| value.to_lowercase())
            {
                index.bundle_ids.insert(bundle_identifier);
            }

            for key in ["CFBundleDisplayName", "CFBundleName"] {
                if let Some(plist_app_name) = dict.get(key).and_then(|value| value.as_string()) {
                    index.normalized_names.insert(normalize_app_name(plist_app_name));
                    if !plist_app_name.trim().is_empty() {
                        app_name = plist_app_name.to_string();
                    }
                }
            }

            version_key = dict
                .get("CFBundleShortVersionString")
                .and_then(|value| value.as_string())
                .and_then(normalize_version_key)
                .or_else(|| {
                    dict.get("CFBundleVersion")
                        .and_then(|value| value.as_string())
                        .and_then(normalize_version_key)
                });
        }

        let normalized_name = normalize_app_name(&app_name);
        let (base_name, embedded_version) = extract_name_and_version(&app_name);
        let base_normalized_name = normalize_app_name(&base_name);

        if !normalized_name.is_empty() {
            index.apps.push(InstalledAppRecord {
                normalized_name,
                base_normalized_name: if base_normalized_name.is_empty() {
                    normalize_app_name(&fallback_name)
                } else {
                    base_normalized_name
                },
                version_key: version_key.or(embedded_version),
            });
        }
    }
}

fn get_installed_apps_index(home: &str) -> InstalledAppsIndex {
    let mut index = InstalledAppsIndex {
        bundle_ids: HashSet::new(),
        normalized_names: HashSet::new(),
        apps: Vec::new(),
    };

    for root in [
        PathBuf::from("/Applications"),
        PathBuf::from(format!("{}/Applications", home)),
    ] {
        collect_installed_apps_from_root(&root, &mut index);
    }

    index
}

fn build_residual_candidate(
    path: &Path,
    category: &str,
    source_key: &str,
    strip_suffix: Option<&str>,
    installed_apps: &InstalledAppsIndex,
) -> Option<AppResidualCandidate> {
    let file_name = path.file_name()?.to_str()?.to_string();
    let logical_name = strip_suffix
        .and_then(|suffix| file_name.strip_suffix(suffix))
        .unwrap_or(&file_name)
        .trim();

    if logical_name.is_empty() {
        return None;
    }

    let bundle_identifier = if is_bundle_identifier(logical_name) {
        Some(logical_name.to_string())
    } else {
        None
    };

    if let Some(identifier) = &bundle_identifier {
        if installed_apps
            .bundle_ids
            .contains(&identifier.to_lowercase())
        {
            return None;
        }
    }

    let normalized_name = normalize_app_name(logical_name);
    if normalized_name.is_empty() || installed_apps.normalized_names.contains(&normalized_name) {
        return None;
    }

    let (base_name, candidate_version_key) = extract_name_and_version(logical_name);
    let base_normalized_name = normalize_app_name(&base_name);
    let matching_installed_apps: Vec<&InstalledAppRecord> = installed_apps
        .apps
        .iter()
        .filter(|app| {
            (!base_normalized_name.is_empty() && app.base_normalized_name == base_normalized_name)
                || app.normalized_name == normalized_name
        })
        .collect();

    let mut is_old_version = false;
    if !matching_installed_apps.is_empty() {
        if candidate_version_key.is_none() {
            return None;
        }

        let same_version_exists = matching_installed_apps.iter().any(|app| {
            app.normalized_name == normalized_name
                || app.version_key.as_ref() == candidate_version_key.as_ref()
        });

        if same_version_exists {
            return None;
        }

        is_old_version = true;
    }

    let size = get_path_size(path);
    if size < MIN_APP_RESIDUAL_ITEM_SIZE {
        return None;
    }

    let app_name = if is_old_version && !base_name.is_empty() {
        prettify_label(&base_name)
    } else {
        bundle_identifier
            .as_deref()
            .map(guess_app_name_from_identifier)
            .unwrap_or_else(|| prettify_label(logical_name))
    };

    let category_label = if is_old_version {
        format!("{}（旧版本）", category)
    } else {
        category.to_string()
    };

    Some(AppResidualCandidate {
        app_id: bundle_identifier
            .as_ref()
            .map(|identifier| identifier.to_lowercase())
            .unwrap_or_else(|| {
                if is_old_version && !base_normalized_name.is_empty() {
                    base_normalized_name.clone()
                } else {
                    normalized_name.clone()
                }
            }),
        app_name,
        identifier: bundle_identifier,
        source_key: source_key.to_string(),
        item: UninstalledAppJunkItem {
            path: path.to_string_lossy().to_string(),
            size,
            category: category_label,
        },
        is_old_version,
    })
}

#[tauri::command]
async fn scan_uninstalled_app_junk() -> Result<UninstalledAppJunkScanResult, String> {
    let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
    let installed_apps = get_installed_apps_index(&home);

    let scan_roots: Vec<(&str, String, Option<&str>)> = vec![
        ("应用支持文件", format!("{}/Library/Application Support", home), None),
        ("缓存文件", format!("{}/Library/Caches", home), None),
        ("日志文件", format!("{}/Library/Logs", home), None),
        ("偏好设置", format!("{}/Library/Preferences", home), Some(".plist")),
        (
            "保存的应用状态",
            format!("{}/Library/Saved Application State", home),
            Some(".savedState"),
        ),
        ("WebKit 数据", format!("{}/Library/WebKit", home), None),
        ("HTTP 缓存", format!("{}/Library/HTTPStorages", home), None),
        ("应用容器", format!("{}/Library/Containers", home), None),
        ("群组容器", format!("{}/Library/Group Containers", home), None),
    ];

    let mut grouped_apps: HashMap<String, AppResidualGroupAccumulator> = HashMap::new();

    for (category, root, strip_suffix) in scan_roots {
        let root_path = PathBuf::from(&root);
        if !root_path.exists() {
            continue;
        }

        let Ok(entries) = fs::read_dir(&root_path) else {
            continue;
        };

        for entry in entries.filter_map(|entry| entry.ok()) {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            if file_type.is_symlink() {
                continue;
            }

            let entry_path = entry.path();
            let Some(candidate) = build_residual_candidate(
                &entry_path,
                category,
                &root,
                strip_suffix,
                &installed_apps,
            ) else {
                continue;
            };

            let group = grouped_apps
                .entry(candidate.app_id.clone())
                .or_insert_with(|| AppResidualGroupAccumulator {
                    app_id: candidate.app_id.clone(),
                    app_name: candidate.app_name.clone(),
                    identifier: candidate.identifier.clone(),
                    total_size: 0,
                    source_keys: HashSet::new(),
                    items: Vec::new(),
                    has_old_version_items: false,
                });

            if group.identifier.is_none() && candidate.identifier.is_some() {
                group.identifier = candidate.identifier.clone();
            }

            if candidate.identifier.is_none()
                && !candidate.app_name.is_empty()
                && candidate.app_name.len() <= group.app_name.len()
            {
                group.app_name = candidate.app_name.clone();
            }

            group.total_size += candidate.item.size;
            group.source_keys.insert(candidate.source_key.clone());
            group.items.push(candidate.item);
            group.has_old_version_items |= candidate.is_old_version;
        }
    }

    let mut apps: Vec<UninstalledAppJunkApp> = grouped_apps
        .into_values()
        .filter(|group| {
            group.has_old_version_items
                || group.identifier.is_some()
                || group.source_keys.len() >= 2
                || group.total_size >= MIN_APP_RESIDUAL_GROUP_SIZE
        })
        .map(|mut group| {
            group
                .items
                .sort_by(|left, right| right.size.cmp(&left.size));

            UninstalledAppJunkApp {
                app_id: group.app_id,
                app_name: group.app_name,
                identifier: group.identifier,
                total_size: group.total_size,
                items: group.items,
            }
        })
        .collect();

    apps.sort_by(|left, right| right.total_size.cmp(&left.total_size));

    let total_size = apps.iter().map(|app| app.total_size).sum();
    let total_items = apps.iter().map(|app| app.items.len()).sum();

    Ok(UninstalledAppJunkScanResult {
        apps,
        total_size,
        total_items,
    })
}

#[tauri::command]
async fn scan_junk_files(request: Option<ScanJunkRequest>) -> Result<JunkScanResult, String> {
    let mut items: Vec<JunkItem> = Vec::new();
    let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;

    let selected_target_ids = request
        .as_ref()
        .map(|scan_request| scan_request.target_ids.clone())
        .unwrap_or_default();

    let downloads_min_age_days = request
        .as_ref()
        .and_then(|r| r.downloads_min_age_days)
        .unwrap_or(30);

    let downloads_min_size_bytes = request
        .as_ref()
        .and_then(|r| r.downloads_min_size_mb)
        .unwrap_or(100) * 1024 * 1024;

    let now = std::time::SystemTime::now();
    let min_age_duration = std::time::Duration::from_secs(downloads_min_age_days * 24 * 60 * 60);

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

                    if target.is_downloads {
                        if size < downloads_min_size_bytes {
                            continue;
                        }

                        if let Ok(modified) = meta.modified() {
                            if let Ok(age) = now.duration_since(modified) {
                                if age < min_age_duration {
                                    continue;
                                }
                            }
                        }
                    } else if size <= 1024 * 1024 {
                        continue;
                    }

                    items.push(JunkItem {
                        path: entry.path().to_string_lossy().to_string(),
                        size,
                        category: target.category.to_string(),
                    });
                }
            }
        } else if path.is_file() {
            let metadata = path.metadata();
            if let Ok(meta) = metadata {
                let size = meta.len();

                if target.is_downloads {
                    if size < downloads_min_size_bytes {
                        continue;
                    }

                    if let Ok(modified) = meta.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age < min_age_duration {
                                continue;
                            }
                        }
                    }
                }

                items.push(JunkItem {
                    path: path.to_string_lossy().to_string(),
                    size,
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
            is_downloads: false,
        },
        JunkScanTargetSpec {
            id: "logs",
            category: "日志文件",
            path: PathBuf::from(format!("{}/Library/Logs", home)),
            is_downloads: false,
        },
        JunkScanTargetSpec {
            id: "tmp",
            category: "临时文件",
            path: PathBuf::from("/tmp"),
            is_downloads: false,
        },
        JunkScanTargetSpec {
            id: "system_cache",
            category: "系统缓存",
            path: PathBuf::from("/Library/Caches"),
            is_downloads: false,
        },
        JunkScanTargetSpec {
            id: "xcode_derived_data",
            category: "Xcode 派生数据",
            path: PathBuf::from(format!("{}/Library/Developer/Xcode/DerivedData", home)),
            is_downloads: false,
        },
        JunkScanTargetSpec {
            id: "application_support",
            category: "应用支持文件",
            path: PathBuf::from(format!("{}/Library/Application Support", home)),
            is_downloads: false,
        },
        JunkScanTargetSpec {
            id: "downloads",
            category: "下载文件",
            path: PathBuf::from(format!("{}/Downloads", home)),
            is_downloads: true,
        },
        JunkScanTargetSpec {
            id: "trash",
            category: "废纸篓",
            path: PathBuf::from(format!("{}/.Trash", home)),
            is_downloads: false,
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
            scan_uninstalled_app_junk,
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
