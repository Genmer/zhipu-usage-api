use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder, Window};
use chrono::{Local, Timelike, Datelike};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedAccount {
    id: String,
    label: String,
    last_used: String,
}

fn accounts_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".zhipu-monitor").join("accounts.json")
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
    usage_data: Mutex<Option<UsageData>>,
    data_window_exists: Mutex<bool>,
    refresh_interval_secs: Mutex<u64>,
    card_switch_secs: Mutex<u64>,
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
struct ScrapeResult {
    data: Option<UsageData>,
    debug: Vec<String>,
    #[serde(rename = "noSubscription")]
    no_subscription: bool,
    #[serde(rename = "userLabel")]
    user_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinkResult {
    text: Option<String>,
    href: Option<String>,
}

const SHOW_EXTRACT_OVERLAY_JS: &str = r#"
(function() {
  if (document.getElementById('__tauri_extract_overlay__')) return;
  var overlay = document.createElement('div');
  overlay.id = '__tauri_extract_overlay__';
  overlay.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.5);display:flex;align-items:center;justify-content:center;z-index:999999;pointer-events:none;';
  var box = document.createElement('div');
  box.style.cssText = 'background:rgba(30,30,30,0.95);border-radius:12px;padding:16px 24px;display:flex;align-items:center;gap:10px;';
  var spinner = document.createElement('div');
  spinner.style.cssText = 'width:16px;height:16px;border:2px solid rgba(255,255,255,0.3);border-top-color:#3b82f6;border-radius:50%;animation:__tauri_spin__ 0.8s linear infinite;';
  var text = document.createElement('span');
  text.style.cssText = 'color:#fff;font-size:13px;font-family:system-ui,sans-serif;';
  text.textContent = '提取用户ID用作记住登录状态使用...';
  var style = document.createElement('style');
  style.textContent = '@keyframes __tauri_spin__{to{transform:rotate(360deg)}}';
  document.head.appendChild(style);
  box.appendChild(spinner);
  box.appendChild(text);
  overlay.appendChild(box);
  document.body.appendChild(overlay);
})()
"#;

const EXTRACT_USER_JS: &str = r#"
(function() {
  var maxAttempts = 5;
  var attempt = 0;

  function submitLabel(label) {
    if (window.__TAURI__ && window.__TAURI__.core) {
      window.__TAURI__.core.invoke('submit_user_info', { label: label });
    } else {
      document.title = '__USER__' + label;
    }
  }

  function doExtract() {
    var allText = document.body.innerText || '';
    var accountIdMatch = allText.match(/账号\s*ID\s*[：:]\s*(\d+)/);
    var phoneMatch = allText.match(/(?:手机|电话)\s*[号码]?\s*[：:]\s*(1[3-9]\d{9})/);
    if (!phoneMatch) phoneMatch = allText.match(/(1[3-9]\d{9})/);
    var nicknameMatch = allText.match(/(?:昵称|用户名|名称)\s*[：:]\s*([^\n\s]{1,20})/);
    var label = accountIdMatch ? 'ID:' + accountIdMatch[1] :
                phoneMatch ? phoneMatch[1] :
                nicknameMatch ? nicknameMatch[1] : '';
    submitLabel(label);
  }

  function tryExtract() {
    attempt++;
    var selectors = [
      '[class*="avatar"]',
      '[class*="Avatar"]',
      '[class*="user-icon"]',
      '[class*="UserIcon"]',
      '[class*="header"] img',
      '[class*="Header"] img',
      '[class*="header"] [class*="icon"]',
      '[class*="Header"] [class*="Icon"]',
      'nav img',
      'header img',
      '[class*="navbar"] img',
      '[class*="NavBar"] img'
    ];

    var userIcon = null;
    for (var s = 0; s < selectors.length; s++) {
      userIcon = document.querySelector(selectors[s]);
      if (userIcon) break;
    }

    if (!userIcon) {
      var allImgs = document.querySelectorAll('img');
      for (var i = 0; i < allImgs.length; i++) {
        if (allImgs[i].width < 50 && allImgs[i].width > 10) { userIcon = allImgs[i]; break; }
      }
    }

    if (!userIcon) {
      var allSvgs = document.querySelectorAll('svg');
      for (var i = 0; i < allSvgs.length; i++) {
        var parent = allSvgs[i].parentElement;
        if (parent) {
          var cls = parent.getAttribute('class') || '';
          if (cls.indexOf('header') !== -1 || cls.indexOf('Header') !== -1 || cls.indexOf('nav') !== -1 || cls.indexOf('Nav') !== -1) {
            userIcon = parent;
            break;
          }
        }
      }
    }

    if (userIcon) {
      userIcon.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
      userIcon.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
      userIcon.click();
      setTimeout(doExtract, 1500);
    } else if (attempt < maxAttempts) {
      setTimeout(tryExtract, 800);
    } else {
      doExtract();
    }
  }

  tryExtract();
})()
"#;

const SCRAPE_JS: &str = r#"
(function() {
  var result = { data: null, debug: [], noSubscription: false, userLabel: null };
  try {
    var allText = document.body.innerText || '';
    result.debug.push('innerText length: ' + allText.length);
    result.debug.push('innerText preview: ' + allText.substring(0, 200));

    if (allText.indexOf('尚未订阅') !== -1 || allText.indexOf('未订阅') !== -1 || allText.indexOf('没有订阅') !== -1 || allText.indexOf('暂无套餐') !== -1) {
      result.debug.push('detected: no subscription');
      result.noSubscription = true;
    }

    var p5match = allText.match(/每[\s]*5[\s]*小[\s]*时[\s\S]*?(\d+)\s*%/);
    var weekmatch = allText.match(/每[\s]*周[\s\S]*?(\d+)\s*%/);

    var p5reset = allText.match(/每[\s]*5[\s]*小[\s]*时[\s\S]*?(?:重置时间|距重置|重置|恢复时间|剩余时间|距恢复)[：:\s]*([^\n]+)/);
    if (!p5reset) p5reset = allText.match(/每[\s]*5[\s]*小[\s]*时[\s\S]*?(\d+[\s:]*\d+(?::\d+)?[^\n]*(?:重置|恢复|剩余)[^\n]*)/);
    if (!p5reset) {
      var p5section = allText.match(/每[\s]*5[\s]*小[\s]*时[\s\S]{0,300}/);
      if (p5section) {
        var timeM = p5section[0].match(/(\d{1,2}[:\.]\d{2}(?::\d{2})?|\d+小时?\d*分?|\d+天\d+小时?|\d+h\s*\d*m)/);
        if (timeM) p5reset = timeM;
      }
    }

    var weekReset = allText.match(/每[\s]*周[\s\S]*?(?:重置时间|距重置|重置|恢复时间|剩余时间|距恢复)[：:\s]*([^\n]+)/);
    if (!weekReset) weekReset = allText.match(/每[\s]*周[\s\S]*?(\d{4}[-.]\d{1,2}[-.]\d{1,2}[^\n]*(?:重置|恢复|剩余)[^\n]*)/);
    if (!weekReset) {
      var weekSection = allText.match(/每[\s]*周[\s\S]{0,300}/);
      if (weekSection) {
        var timeM2 = weekSection[0].match(/(\d{1,2}[:\.]\d{2}(?::\d{2})?|\d+小时?\d*分?|\d+天\d+小时?|\d+h\s*\d*m)/);
        if (timeM2) weekReset = timeM2;
      }
    }

    result.debug.push('p5match: ' + (p5match ? p5match[1] : 'null'));
    result.debug.push('weekmatch: ' + (weekmatch ? weekmatch[1] : 'null'));

    try {
      var allEls = document.querySelectorAll('*');
      for (var i = 0; i < allEls.length; i++) {
        var el = allEls[i];
        if (el.offsetWidth > 0 && el.offsetWidth < 60 && el.offsetHeight > 0 && el.offsetHeight < 60) {
          var cls = (el.getAttribute('class') || '').toLowerCase();
          var parentCls = (el.parentElement ? el.parentElement.getAttribute('class') || '' : '').toLowerCase();
          if (cls.indexOf('avatar') !== -1 || cls.indexOf('user') !== -1 || parentCls.indexOf('avatar') !== -1 || parentCls.indexOf('user') !== -1) {
            el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
            el.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
            el.click();
            break;
          }
        }
      }
    } catch(e) { result.debug.push('hover error: ' + e.message); }

    if (p5match || weekmatch) {
      result.data = {
        hourly: {
          percentage: p5match ? p5match[1] + '%' : '0%',
          resetTime: p5reset ? p5reset[1].trim() : '未知'
        },
        weekly: {
          percentage: weekmatch ? weekmatch[1] + '%' : '0%',
          resetTime: weekReset ? weekReset[1].trim() : '未知'
        },
        timestamp: new Date().toISOString()
      };
    }

    setTimeout(function() {
      try {
        var newText = document.body.innerText || '';
        var idMatch = newText.match(/账号\s*ID\s*[：:]\s*(\d+)/);
        if (idMatch) {
          result.userLabel = 'ID:' + idMatch[1];
        } else {
          var phoneMatch = newText.match(/(1[3-9]\d{9})/);
          if (phoneMatch) result.userLabel = phoneMatch[1];
        }
      } catch(e) {}

      if (window.__TAURI__ && window.__TAURI__.core) {
        result.debug.push('using __TAURI__.core.invoke');
        window.__TAURI__.core.invoke('submit_scrape_result', { result: result });
      } else if (window.__TAURI__ && window.__TAURI__.event) {
        result.debug.push('using __TAURI__.event.emit');
        window.__TAURI__.event.emit('scrape-result', result);
      } else {
        result.debug.push('ERROR: __TAURI__ not available, using title fallback');
        document.title = '__SCRAPE__' + JSON.stringify(result);
      }
    }, 1500);
  } catch(e) {
    result.debug.push('JS ERROR: ' + e.message);
    document.title = '__SCRAPE__' + JSON.stringify(result);
  }
})()
"#;

const SCRAPE_WITH_LINK_JS: &str = r#"
(function() {
  try {
    var links = document.querySelectorAll('a[href]');
    var found = null;
    for (var i = 0; i < links.length; i++) {
      var href = links[i].getAttribute('href') || '';
      var text = (links[i].textContent || '').trim();
      if (text.indexOf('Coding') !== -1 || text.indexOf('coding') !== -1
        || href.indexOf('coding') !== -1 || href.indexOf('plan') !== -1) {
        found = { text: text, href: href };
        break;
      }
    }
    if (window.__TAURI__ && window.__TAURI__.core) {
      window.__TAURI__.core.invoke('submit_link_result', { result: found });
    } else if (window.__TAURI__ && window.__TAURI__.event) {
      window.__TAURI__.event.emit('scrape-link-result', found);
    } else {
      document.title = '__LINK__' + JSON.stringify(found);
    }
  } catch(e) {
    document.title = '__LINK__' + JSON.stringify(null);
  }
})()
"#;

fn process_scrape_result(app: &AppHandle, scrape: ScrapeResult) {
    for line in &scrape.debug {
        log::info!("[DATA-DOM] {}", line);
    }

    if let Some(ref label) = scrape.user_label {
        if !label.is_empty() {
            log::info!("[DATA] extracted user label from data window: {}", label);
            let account = SavedAccount {
                id: format!("acc_{}", Local::now().format("%Y%m%d%H%M%S")),
                label: label.clone(),
                last_used: Local::now().format("%Y-%m-%d %H:%M").to_string(),
            };
            let mut accounts = load_accounts();
            accounts.retain(|a| a.label != account.label);
            accounts.insert(0, account);
            if accounts.len() > 5 {
                accounts.truncate(5);
            }
            save_accounts(&accounts);
        }
    }

    if scrape.no_subscription {
        log::info!("[DATA] account has no Coding Plan subscription");
        let _ = app.emit("no-subscription", ());
        close_and_reset_data_window(app);
        return;
    }

    if let Some(mut data) = scrape.data {
        save_and_emit_usage(app, &mut data);
        close_and_reset_data_window(app);
    } else {
        log::info!("[DATA] no usage data found, trying to find Coding Plan link...");
        if let Some(win) = app.get_webview_window("data-scraper") {
            let _ = win.eval(SCRAPE_WITH_LINK_JS);
        }
    }
}

fn process_link_result(app: &AppHandle, link: Option<LinkResult>) {
    if let Some(l) = link {
        let href = l.href.unwrap_or_default();
        let full_url = if href.starts_with("http") {
            href
        } else {
            format!("https://open.bigmodel.cn{}", href)
        };
        log::info!("[DATA] navigating to: {}", full_url);
        if let Ok(url) = full_url.parse() {
            if let Some(win) = app.get_webview_window("data-scraper") {
                let app2 = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(8));
                    log::info!("[DATA] re-scraping after navigation...");
                    if let Some(w) = app2.get_webview_window("data-scraper") {
                        let _ = w.eval(SCRAPE_JS);
                    }
                });
                let _ = win.navigate(url);
            }
        } else {
            close_and_reset_data_window(app);
        }
    } else {
        log::info!("[DATA] no Coding Plan link found");
        close_and_reset_data_window(app);
    }
}

fn close_and_reset_data_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("data-scraper") {
        let _ = win.close();
    }
    *app.state::<AppState>().data_window_exists.lock().unwrap() = false;
}

#[tauri::command]
fn submit_scrape_result(app: AppHandle, result: ScrapeResult) {
    log::info!("[DATA] received scrape result via invoke");
    process_scrape_result(&app, result);
}

#[tauri::command]
fn submit_link_result(app: AppHandle, result: Option<LinkResult>) {
    log::info!("[DATA] received link result via invoke: {:?}", result);
    process_link_result(&app, result);
}

fn save_and_emit_usage(app: &AppHandle, data: &mut UsageData) {
    let (fallback_hourly, fallback_weekly) = calc_reset_times();
    if data.hourly.reset_time == "未知" {
        data.hourly.reset_time = format!("约 {}", fallback_hourly);
    }
    if data.weekly.reset_time == "未知" {
        data.weekly.reset_time = fallback_weekly;
    }

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

fn create_data_window(app: &AppHandle) {
    let exists = *app.state::<AppState>().data_window_exists.lock().unwrap();
    if exists {
        log::info!("[DATA] data window already exists, skipping");
        return;
    }
    *app.state::<AppState>().data_window_exists.lock().unwrap() = true;

    let data_label = "data-scraper";

    log::info!("[DATA] creating data window...");
    let win = match WebviewWindowBuilder::new(
        app,
        data_label,
        WebviewUrl::External(
            "https://open.bigmodel.cn/coding-plan/personal/overview"
                .parse()
                .unwrap(),
        ),
    )
    .inner_size(1200.0, 800.0)
    .visible(false)
    .build()
    {
        Ok(w) => w,
        Err(e) => {
            log::error!("[DATA] failed to create data window: {}", e);
            *app.state::<AppState>().data_window_exists.lock().unwrap() = false;
            return;
        }
    };
    log::info!("[DATA] data window created, waiting for page load...");

    let _title_handler = win.on_window_event(move |event| {
        match event {
            tauri::WindowEvent::Destroyed => {
                log::info!("[DATA] data window destroyed");
            }
            tauri::WindowEvent::Focused(_) => {}
            _ => {}
        }
    });

    let app_clone = app.clone();

    app_clone.clone().listen("scrape-result", move |event| {
        log::info!("[DATA] scrape-result event received");
        let app_inner = app_clone.clone();
        let payload = event.payload().to_string();
        let scrape: ScrapeResult = match serde_json::from_str(&payload) {
            Ok(s) => s,
            Err(e) => {
                log::error!("[DATA] parse scrape result failed: {}", e);
                return;
            }
        };
        process_scrape_result(&app_inner, scrape);
    });

    let app_clone2 = app.clone();
    app_clone2.clone().listen("scrape-link-result", move |event| {
        log::info!("[DATA] scrape-link-result event received");
        let app_inner = app_clone2.clone();
        let payload = event.payload().to_string();
        let link: Option<LinkResult> = serde_json::from_str(&payload).ok();
        process_link_result(&app_inner, link);
    });

    let app_for_timer = app.clone();
    std::thread::spawn(move || {
        log::info!("[DATA] timer thread started, waiting 6 seconds...");
        std::thread::sleep(std::time::Duration::from_secs(6));
        log::info!("[DATA] injecting scrape JS...");
        if let Some(w) = app_for_timer.get_webview_window("data-scraper") {
            match w.eval(SCRAPE_JS) {
                Ok(_) => log::info!("[DATA] scrape JS injected successfully"),
                Err(e) => log::error!("[DATA] scrape JS injection failed: {}", e),
            }
        } else {
            log::error!("[DATA] data-scraper window not found for eval");
            return;
        }

        log::info!("[DATA] polling title for fallback...");
        for i in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if let Some(w) = app_for_timer.get_webview_window("data-scraper") {
                match w.title() {
                    Ok(title) => {
                        if title.starts_with("__SCRAPE__") {
                            log::info!("[DATA] got result via title fallback (attempt {})", i);
                            let json_str = &title["__SCRAPE__".len()..];
                            if let Ok(scrape) = serde_json::from_str::<ScrapeResult>(json_str) {
                                process_scrape_result(&app_for_timer, scrape);
                            } else {
                                log::error!("[DATA] failed to parse title scrape result");
                                close_and_reset_data_window(&app_for_timer);
                            }
                            return;
                        } else if title.starts_with("__LINK__") {
                            log::info!("[DATA] got link result via title fallback (attempt {})", i);
                            let json_str = &title["__LINK__".len()..];
                            let link: Option<LinkResult> = serde_json::from_str(json_str).ok();
                            process_link_result(&app_for_timer, link);
                            return;
                        }
                    }
                    Err(e) => {
                        log::error!("[DATA] failed to get window title: {}", e);
                        break;
                    }
                }
            } else {
                log::info!("[DATA] data window gone, stopping poll");
                return;
            }
        }
        log::warn!("[DATA] title poll timed out, no data received");
    });
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
async fn open_login_window(app: AppHandle) -> Result<bool, String> {
    let login_label = "login";

    if let Some(existing) = app.get_webview_window(login_label) {
        let _ = existing.set_focus();
        return Ok(true);
    }

    let login_win = WebviewWindowBuilder::new(
        &app,
        login_label,
        WebviewUrl::External("https://open.bigmodel.cn/login".parse().unwrap()),
    )
    .title("智谱AI - 登录")
    .inner_size(900.0, 700.0)
    .resizable(true)
    .build()
    .map_err(|e| format!("failed to create login window: {}", e))?;

    let app_clone = app.clone();
    let login_win_clone = login_win.clone();

    let _check_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let url = match login_win_clone.url() {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            };

            if !url.contains("/login") && !url.contains("/signin") {
                let cookies = match login_win_clone.cookies() {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                if cookies.is_empty() {
                    continue;
                }

                {
                    let state = app_clone.state::<AppState>();
                    *state.is_logged_in.lock().unwrap() = true;
                }

                let _ = app_clone.emit("login-successful", ());

                let _app_for_extract = app_clone.clone();
                let win_for_extract = login_win_clone.clone();
                let _ = tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let _ = win_for_extract.eval(SHOW_EXTRACT_OVERLAY_JS);
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    let _ = win_for_extract.eval(EXTRACT_USER_JS);
                });

                let app_for_close = app_clone.clone();
                let win_for_close = login_win_clone.clone();
                let _ = tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(8)).await;
                    let _ = win_for_close.close();

                    let _ = tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        create_data_window(&app_for_close);
                    });
                });

                break;
            }
        }
    });

    Ok(true)
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
        return Err("not logged in".to_string());
    }
    drop(state);

    create_data_window(&app);
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
fn submit_user_info(app: AppHandle, label: String) {
    if label.is_empty() {
        log::warn!("[AUTH] user label is empty, skipping save");
        return;
    }
    log::info!("[AUTH] detected user: {}", label);

    let id = format!("acc_{}", Local::now().format("%Y%m%d%H%M%S"));
    let account = SavedAccount {
        id: id.clone(),
        label: label.clone(),
        last_used: Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };

    let mut accounts = load_accounts();
    accounts.retain(|a| a.label != account.label);
    accounts.insert(0, account);
    if accounts.len() > 5 {
        accounts.truncate(5);
    }
    save_accounts(&accounts);
    let _ = app.emit("account-saved", label);
}

#[tauri::command]
fn get_saved_accounts() -> Vec<SavedAccount> {
    load_accounts()
}

fn clear_webview_browsing_data(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        match win.clear_all_browsing_data() {
            Ok(_) => log::info!("[AUTH] cleared all browsing data"),
            Err(e) => log::error!("[AUTH] failed to clear browsing data: {}", e),
        }
    }
}

#[tauri::command]
fn remove_saved_account(app: AppHandle, id: String) {
    let mut accounts = load_accounts();
    accounts.retain(|a| a.id != id);
    save_accounts(&accounts);
    if accounts.is_empty() {
        clear_webview_browsing_data(&app);
    }
}

#[tauri::command]
async fn login_with_saved_account(app: AppHandle, id: String) -> Result<bool, String> {
    let accounts = load_accounts();
    let account = accounts.iter().find(|a| a.id == id).cloned();
    match account {
        Some(acc) => {
            log::info!("[AUTH] logging in with saved account: {}", acc.label);
            {
                let state = app.state::<AppState>();
                *state.is_logged_in.lock().unwrap() = true;
            }
            let mut updated = accounts.clone();
            updated.retain(|a| a.id != id);
            let updated_acc = SavedAccount {
                id: acc.id,
                label: acc.label,
                last_used: Local::now().format("%Y-%m-%d %H:%M").to_string(),
            };
            updated.insert(0, updated_acc);
            save_accounts(&updated);
            let _ = app.emit("login-successful", ());
            create_data_window(&app);
            Ok(true)
        }
        None => Err("account not found".to_string()),
    }
}

#[tauri::command]
async fn logout(app: AppHandle) -> Result<bool, String> {
    log::info!("[AUTH] logging out...");
    {
        let state = app.state::<AppState>();
        *state.is_logged_in.lock().unwrap() = false;
        *state.usage_data.lock().unwrap() = None;
    }

    if let Some(win) = app.get_webview_window("data-scraper") {
        let _ = win.close();
    }
    *app.state::<AppState>().data_window_exists.lock().unwrap() = false;

    let _ = app.emit("logged-out", ());
    Ok(true)
}

#[tauri::command]
fn set_card_switch_secs(app: AppHandle, seconds: u64) {
    let clamped = seconds.clamp(5, 300);
    *app.state::<AppState>().card_switch_secs.lock().unwrap() = clamped;
    let _ = app.emit("card-switch-interval-changed", clamped);
    log::info!("[CONFIG] card switch interval set to {}s", clamped);
}

fn start_auto_refresh(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            let interval = *app.state::<AppState>().refresh_interval_secs.lock().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(interval));

            let logged_in = *app.state::<AppState>().is_logged_in.lock().unwrap();
            if logged_in {
                create_data_window(&app);
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
            usage_data: Mutex::new(None),
            data_window_exists: Mutex::new(false),
            refresh_interval_secs: Mutex::new(180),
            card_switch_secs: Mutex::new(30),
        })
        .setup(|app| {
            let handle = app.handle().clone();

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_shadow(false);
            }

            start_auto_refresh(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_login_status,
            get_usage_data,
            open_login_window,
            toggle_always_on_top,
            start_dragging,
            refresh_usage_data,
            submit_scrape_result,
            submit_link_result,
            submit_user_info,
            get_saved_accounts,
            remove_saved_account,
            login_with_saved_account,
            logout,
            get_refresh_interval,
            set_refresh_interval,
            get_card_switch_secs,
            set_card_switch_secs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
