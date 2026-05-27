use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Window};
use tauri::tray::TrayIconBuilder;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use chrono::{Local, Timelike, Datelike};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedAccount {
    id: String,
    label: String,
    api_key: String,
    last_used: String,
}

fn accounts_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".zhipu-monitor").join("api-keys.json")
}

fn load_accounts() -> Vec<SavedAccount> {
    let path = accounts_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_accounts(accounts: &[SavedAccount]) {
    if let Some(parent) = accounts_path().parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(accounts_path(), serde_json::to_string_pretty(accounts).unwrap_or_default());
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return key.to_string();
    }
    format!("{}****{}", &key[..4], &key[key.len() - 4..])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UsageData {
    hourly: QuotaInfo,
    weekly: QuotaInfo,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuotaInfo {
    percentage: String,
    #[serde(rename = "resetTime")]
    reset_time: String,
}

struct AppState {
    is_logged_in: Mutex<bool>,
    current_api_key: Mutex<Option<String>>,
    usage_data: Mutex<Option<UsageData>>,
    refresh_interval_secs: Mutex<u64>,
    card_switch_secs: Mutex<u64>,
    http_client: reqwest::blocking::Client,
}

fn calc_reset_times() -> (String, String) {
    let now = Local::now();
    let hour = now.hour();
    let minute = now.minute();

    let next_5h = {
        let next_hour = ((hour / 5) + 1) * 5;
        if next_hour >= 24 {
            format!("{:02}:00", next_hour - 24)
        } else {
            format!("{:02}:00", next_hour)
        }
    };

    let weekly = {
        let mut days_until_mon = (8 - now.weekday().num_days_from_sunday()) % 7;
        if days_until_mon == 0 {
            if hour != 0 || minute != 0 {
                days_until_mon = 7;
            }
        }
        let mut reset_date = now + chrono::Duration::days(days_until_mon as i64);
        reset_date = reset_date
            .with_hour(0)
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .unwrap_or(reset_date);
        format!(
            "{:04}-{:02}-{:02} 00:00",
            reset_date.year(),
            reset_date.month(),
            reset_date.day()
        )
    };

    (next_5h, weekly)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiLimit {
    #[serde(rename = "type")]
    limit_type: String,
    unit: Option<u64>,
    number: Option<u64>,
    usage: Option<f64>,
    #[serde(rename = "currentValue")]
    current_value: Option<f64>,
    percentage: Option<f64>,
    #[serde(rename = "nextResetTime")]
    next_reset_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiData {
    limits: Option<Vec<ApiLimit>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse {
    code: Option<u64>,
    msg: Option<String>,
    success: Option<bool>,
    data: Option<ApiData>,
}

fn fetch_usage_from_api(api_key: &str, client: &reqwest::blocking::Client) -> Result<UsageData, String> {
    let response = client
        .get("https://bigmodel.cn/api/monitor/usage/quota/limit")
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("API Key 无效或已过期".to_string());
    }
    if !status.is_success() {
        return Err(format!("请求失败，状态码: {}", status));
    }

    let api_resp: ApiResponse = response
        .json()
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if api_resp.code.unwrap_or(0) != 200 {
        return Err(api_resp.msg.unwrap_or_else(|| "未知错误".to_string()));
    }

    let limits = api_resp
        .data
        .and_then(|d| d.limits)
        .unwrap_or_default();

    for (i, limit) in limits.iter().enumerate() {
        log::info!(
            "[API] limit[{}] type={} unit={:?} number={:?} percentage={:?} usage={:?} currentValue={:?} nextResetTime={:?}",
            i, limit.limit_type, limit.unit, limit.number, limit.percentage, limit.usage, limit.current_value, limit.next_reset_time
        );
    }

    let (fallback_hourly, fallback_weekly) = calc_reset_times();

    fn calc_pct(limit: &ApiLimit) -> f64 {
        if let Some(pct) = limit.percentage {
            if pct > 0.0 {
                return pct;
            }
        }
        match (limit.current_value, limit.usage) {
            (Some(cv), Some(u)) if u > 0.0 => (cv / u) * 100.0,
            _ => 0.0,
        }
    }

    fn fmt_reset_time(reset_ts: Option<u64>, fallback: &str) -> String {
        match reset_ts {
            Some(ts) => {
                let secs = ts / 1000;
                let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
                    .map(|utc| utc.with_timezone(&Local));
                match dt {
                    Some(dt) => {
                        let now = Local::now();
                        if dt.date_naive() == now.date_naive() {
                            format!("{:02}:{:02}", dt.hour(), dt.minute())
                        } else {
                            format!("{}月{}日 {:02}:{:02}", dt.month(), dt.day(), dt.hour(), dt.minute())
                        }
                    }
                    None => format!("约 {}", fallback),
                }
            }
            None => format!("约 {}", fallback),
        }
    }

    let tokens_limits: Vec<&ApiLimit> = limits
        .iter()
        .filter(|l| l.limit_type == "TOKENS_LIMIT")
        .collect();

    let hourly_limit = tokens_limits
        .iter()
        .find(|l| l.unit == Some(3) && l.number == Some(5))
        .copied();
    let weekly_limit = tokens_limits
        .iter()
        .find(|l| l.unit == Some(6) && l.number == Some(1))
        .copied();

    let hourly_percentage = match hourly_limit {
        Some(l) => format!("{}%", calc_pct(l).round() as u64),
        None => "0%".to_string(),
    };
    let hourly_reset = match hourly_limit {
        Some(l) => fmt_reset_time(l.next_reset_time, &fallback_hourly),
        None => format!("约 {}", fallback_hourly),
    };

    let weekly_percentage = match weekly_limit {
        Some(l) => format!("{}%", calc_pct(l).round() as u64),
        None => "0%".to_string(),
    };
    let weekly_reset = match weekly_limit {
        Some(l) => fmt_reset_time(l.next_reset_time, &fallback_weekly),
        None => fallback_weekly.clone(),
    };

    log::info!(
        "[API] result: hourly_pct={} hourly_reset={} weekly_pct={} weekly_reset={}",
        hourly_percentage, hourly_reset, weekly_percentage, weekly_reset
    );

    Ok(UsageData {
        hourly: QuotaInfo {
            percentage: hourly_percentage,
            reset_time: hourly_reset,
        },
        weekly: QuotaInfo {
            percentage: weekly_percentage,
            reset_time: weekly_reset,
        },
        timestamp: Local::now().to_rfc3339(),
    })
}

fn save_and_emit_usage(app: &AppHandle, data: &UsageData) {
    log::info!(
        "[DATA] hourly={} reset={} weekly={} reset={}",
        data.hourly.percentage,
        data.hourly.reset_time,
        data.weekly.percentage,
        data.weekly.reset_time
    );

    let cloned = data.clone();
    *app.state::<AppState>().usage_data.lock().unwrap() = Some(cloned.clone());
    let _ = app.emit("usage-data-updated", cloned);
}

fn fetch_and_emit_usage(app: &AppHandle) {
    let api_key = {
        let state = app.state::<AppState>();
        let key = state.current_api_key.lock().unwrap().clone();
        drop(state);
        key
    };

    match api_key {
        Some(key) => {
            let client = &app.state::<AppState>().http_client;
            match fetch_usage_from_api(&key, client) {
                Ok(data) => {
                    save_and_emit_usage(app, &data);
                }
                Err(e) => {
                    log::error!("[DATA] 获取额度数据失败: {}", e);
                    let _ = app.emit("api-error", e);
                }
            }
        }
        None => {
            log::warn!("[DATA] 未设置API Key");
        }
    }
}

#[tauri::command]
fn get_login_status(state: tauri::State<AppState>) -> bool {
    *state.is_logged_in.lock().unwrap()
}

#[tauri::command]
fn get_usage_data(state: tauri::State<AppState>) -> UsageData {
    state
        .usage_data
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| UsageData {
            hourly: QuotaInfo {
                percentage: "0%".to_string(),
                reset_time: "加载中...".to_string(),
            },
            weekly: QuotaInfo {
                percentage: "0%".to_string(),
                reset_time: "加载中...".to_string(),
            },
            timestamp: Local::now().to_rfc3339(),
        })
}

#[tauri::command]
async fn login_with_api_key(app: AppHandle, api_key: String) -> Result<bool, String> {
    let key = api_key.trim().to_string();
    if key.is_empty() {
        return Err("API Key 不能为空".to_string());
    }

    let client = app.state::<AppState>().http_client.clone();
    let key_clone = key.clone();
    let usage_data = tokio::task::spawn_blocking(move || {
        fetch_usage_from_api(&key_clone, &client)
    }).await.map_err(|e| format!("任务执行失败: {}", e))??;

    {
        let state = app.state::<AppState>();
        *state.is_logged_in.lock().unwrap() = true;
        *state.current_api_key.lock().unwrap() = Some(key.clone());
    }

    let masked = mask_key(&key);
    let id = format!("key_{}", Local::now().format("%Y%m%d%H%M%S"));
    let account = SavedAccount {
        id: id.clone(),
        label: masked,
        api_key: key.clone(),
        last_used: Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };

    let mut accounts = load_accounts();
    accounts.retain(|a| a.api_key != key);
    accounts.insert(0, account);
    if accounts.len() > 5 {
        accounts.truncate(5);
    }
    save_accounts(&accounts);

    let _ = app.emit("login-successful", ());
    save_and_emit_usage(&app, &usage_data);

    Ok(true)
}

#[tauri::command]
async fn login_with_saved_account(app: AppHandle, id: String) -> Result<bool, String> {
    let accounts = load_accounts();
    let account = accounts.iter().find(|a| a.id == id).cloned();
    match account {
        Some(acc) => {
            log::info!("[AUTH] logging in with saved key: {}", acc.label);

            let client = app.state::<AppState>().http_client.clone();
            let api_key = acc.api_key.clone();
            let usage_data = tokio::task::spawn_blocking(move || {
                fetch_usage_from_api(&api_key, &client)
            }).await.map_err(|e| format!("任务执行失败: {}", e))??;

            {
                let state = app.state::<AppState>();
                *state.is_logged_in.lock().unwrap() = true;
                *state.current_api_key.lock().unwrap() = Some(acc.api_key.clone());
            }

            let mut updated = accounts.clone();
            updated.retain(|a| a.id != id);
            let updated_acc = SavedAccount {
                id: acc.id,
                label: acc.label,
                api_key: acc.api_key,
                last_used: Local::now().format("%Y-%m-%d %H:%M").to_string(),
            };
            updated.insert(0, updated_acc);
            save_accounts(&updated);

            let _ = app.emit("login-successful", ());
            save_and_emit_usage(&app, &usage_data);

            Ok(true)
        }
        None => Err("未找到该Key".to_string()),
    }
}

#[tauri::command]
fn toggle_always_on_top(window: Window) -> bool {
    let is_on_top = window.is_always_on_top().unwrap_or(false);
    window.set_always_on_top(!is_on_top).unwrap_or(());
    !is_on_top
}

#[tauri::command]
fn start_dragging(window: Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn refresh_usage_data(app: AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let logged_in = *state.is_logged_in.lock().unwrap();
    if !logged_in {
        return Err("未登录".to_string());
    }
    drop(state);

    let app_clone = app.clone();
    std::thread::spawn(move || {
        fetch_and_emit_usage(&app_clone);
    });
    Ok(true)
}

#[tauri::command]
fn get_refresh_interval(state: tauri::State<AppState>) -> u64 {
    *state.refresh_interval_secs.lock().unwrap()
}

#[tauri::command]
fn set_refresh_interval(app: AppHandle, seconds: u64) {
    let clamped = seconds.clamp(30, 3600);
    *app.state::<AppState>().refresh_interval_secs.lock().unwrap() = clamped;
    log::info!("[CONFIG] refresh interval set to {}s", clamped);
}

#[tauri::command]
fn get_card_switch_secs(state: tauri::State<AppState>) -> u64 {
    *state.card_switch_secs.lock().unwrap()
}

#[tauri::command]
fn get_saved_accounts() -> Vec<SavedAccount> {
    load_accounts()
}

#[tauri::command]
fn remove_saved_account(id: String) {
    let mut accounts = load_accounts();
    accounts.retain(|a| a.id != id);
    save_accounts(&accounts);
}

#[tauri::command]
async fn logout(app: AppHandle) -> Result<bool, String> {
    log::info!("[AUTH] logging out...");
    {
        let state = app.state::<AppState>();
        *state.is_logged_in.lock().unwrap() = false;
        *state.current_api_key.lock().unwrap() = None;
        *state.usage_data.lock().unwrap() = None;
    }

    let _ = app.emit("logged-out", ());
    Ok(true)
}

#[tauri::command]
fn set_card_switch_secs(app: AppHandle, seconds: u64) {
    let clamped = if seconds == 0 { 0 } else { seconds.clamp(5, 300) };
    *app.state::<AppState>().card_switch_secs.lock().unwrap() = clamped;
    let _ = app.emit("card-switch-interval-changed", clamped);
    log::info!("[CONFIG] card switch interval set to {}s", clamped);
}

#[tauri::command]
fn hide_window_to_tray(window: Window) {
    log::info!("[TRAY] hiding window to tray");
    let _ = window.hide();
}

#[tauri::command]
fn quit_app(_app: AppHandle) {
    log::info!("[TRAY] quitting app");
    std::process::exit(0);
}

fn start_auto_refresh(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            let interval = *app.state::<AppState>().refresh_interval_secs.lock().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(interval));

            let logged_in = *app.state::<AppState>().is_logged_in.lock().unwrap();
            if logged_in {
                fetch_and_emit_usage(&app);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .manage(AppState {
            is_logged_in: Mutex::new(false),
            current_api_key: Mutex::new(None),
            usage_data: Mutex::new(None),
            refresh_interval_secs: Mutex::new(180),
            card_switch_secs: Mutex::new(30),
            http_client: reqwest::blocking::Client::new(),
        })
        .setup(|app| {
            let handle = app.handle().clone();

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_shadow(false);
            }

            // macOS: 设为后台代理程序，Dock 不显示图标，悬浮窗口照常显示
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                let _ = app.handle().set_activation_policy(ActivationPolicy::Accessory);
            }

            // Windows: 从任务栏隐藏图标，悬浮窗保持显示
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_skip_taskbar(true);
                }
            }

            // 创建系统托盘图标
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let img = image::load_from_memory(icon_bytes)
                .expect("Failed to decode tray icon")
                .into_rgba8();
            let (width, height) = img.dimensions();
            let rgba = img.into_raw();
            let icon = tauri::image::Image::new_owned(rgba, width, height);

            let show_hide_item = MenuItemBuilder::with_id("show_hide", "显示/隐藏窗口")
                .build(app)
                .expect("Failed to create show/hide menu item");
            let quit_item = MenuItemBuilder::with_id("quit", "退出")
                .build(app)
                .expect("Failed to create quit menu item");
            let menu = MenuBuilder::new(app)
                .item(&show_hide_item)
                .separator()
                .item(&quit_item)
                .build()
                .expect("Failed to build tray menu");

            let handle_for_tray = handle.clone();
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show_hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                    log::info!("[TRAY] window hidden");
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    log::info!("[TRAY] window shown");
                                }
                            }
                        }
                        "quit" => {
                            log::info!("[TRAY] quitting from tray menu");
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |_tray, event| {
                    use tauri::tray::MouseButton;
                    if let tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        if let Some(window) = handle_for_tray.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)
                .expect("Failed to build tray icon");

            start_auto_refresh(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_login_status,
            get_usage_data,
            login_with_api_key,
            login_with_saved_account,
            toggle_always_on_top,
            start_dragging,
            refresh_usage_data,
            get_saved_accounts,
            remove_saved_account,
            logout,
            get_refresh_interval,
            set_refresh_interval,
            get_card_switch_secs,
            set_card_switch_secs,
            hide_window_to_tray,
            quit_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // 拦截系统退出请求（Dock右键退出/Cmd+Q/Alt+F4等），
            // 只有通过托盘菜单或卡片「完全退出」才能退出程序
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                log::info!("[TRAY] preventing system exit, use tray menu to quit");
                api.prevent_exit();
            }
        });
}
