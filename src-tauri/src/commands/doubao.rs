use indexmap::IndexMap;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::provider::{Provider, ProviderMeta, UsageData, UsageResult, UsageScript};
use crate::services::{ProviderSortUpdate, SwitchResult};
use crate::store::AppState;

const APP_ID: &str = "doubao";
const NATIVE_PROVIDER_ID: &str = "doubao-native-account";
const QUOTA_URL: &str = "https://www.doubao.com/alice/commerce/sale/subscription/overview/";
const WEBSITE_URL: &str = "https://www.doubao.com";
const QUOTA_REFERER: &str = "https://www.doubao.com/member/quota-management";

fn native_provider(
    account: crate::doubao_config::DoubaoNativeAccount,
    existing: Option<Provider>,
) -> Provider {
    let mut provider = existing.unwrap_or_else(|| Provider {
        id: NATIVE_PROVIDER_ID.to_string(),
        name: "豆包".to_string(),
        settings_config: Value::Null,
        website_url: Some(WEBSITE_URL.to_string()),
        category: Some("official".to_string()),
        created_at: Some(chrono::Utc::now().timestamp_millis()),
        sort_index: Some(0),
        notes: None,
        meta: None,
        icon: Some("doubao".to_string()),
        icon_color: None,
        in_failover_queue: false,
    });
    provider.id = NATIVE_PROVIDER_ID.to_string();
    provider.name = "豆包".to_string();
    provider.settings_config = json!({
        "native": true,
        "profileName": account.profile_name,
        "credentialSource": "doubao-native",
        "baseUrl": "https://www.doubao.com",
        "installed": true,
    });
    provider.website_url = Some(WEBSITE_URL.to_string());
    provider.category = Some("official".to_string());
    provider.icon = Some("doubao".to_string());
    provider.icon_color = None;
    let meta = provider.meta.get_or_insert_with(ProviderMeta::default);
    meta.usage_script.get_or_insert_with(|| UsageScript {
        enabled: true,
        language: "javascript".to_string(),
        code: String::new(),
        timeout: Some(15),
        api_key: None,
        base_url: None,
        access_token: None,
        user_id: None,
        template_type: Some("token_plan".to_string()),
        auto_query_interval: Some(5),
        coding_plan_provider: Some("doubao".to_string()),
        access_key_id: None,
        secret_access_key: None,
        team_organization_id: None,
        team_project_id: None,
    });
    provider
}

fn sync_native(state: &AppState) -> Result<(usize, String), String> {
    let account = crate::doubao_config::native_account()?;
    if let Some(account) = account {
        let existing = state
            .db
            .get_provider_by_id(NATIVE_PROVIDER_ID, APP_ID)
            .map_err(|e| e.to_string())?;
        state
            .db
            .save_provider(APP_ID, &native_provider(account, existing))
            .map_err(|e| e.to_string())?;
        state
            .db
            .set_current_provider(APP_ID, NATIVE_PROVIDER_ID)
            .map_err(|e| e.to_string())?;
        Ok((1, NATIVE_PROVIDER_ID.to_string()))
    } else {
        if state
            .db
            .get_provider_by_id(NATIVE_PROVIDER_ID, APP_ID)
            .map_err(|e| e.to_string())?
            .is_some()
        {
            state
                .db
                .delete_provider(APP_ID, NATIVE_PROVIDER_ID)
                .map_err(|e| e.to_string())?;
        }
        Ok((0, String::new()))
    }
}

pub(crate) fn list_providers(state: &AppState) -> Result<IndexMap<String, Provider>, String> {
    sync_native(state)?;
    state
        .db
        .get_all_providers(APP_ID)
        .map_err(|e| e.to_string())
}

pub(crate) fn current_provider(state: &AppState) -> Result<String, String> {
    sync_native(state).map(|(_, current)| current)
}

pub(crate) fn doubao_add_provider(_state: &AppState, _provider: Provider) -> Result<bool, String> {
    Err("豆包桌面端仅支持导入原生账号，不支持新增自定义供应商".to_string())
}

pub(crate) fn doubao_update_provider(
    state: &AppState,
    original_id: Option<&str>,
    provider: Provider,
) -> Result<bool, String> {
    if original_id.is_some_and(|id| id != NATIVE_PROVIDER_ID) || provider.id != NATIVE_PROVIDER_ID {
        return Err("豆包原生账号 ID 不支持修改".to_string());
    }
    let account =
        crate::doubao_config::native_account()?.ok_or_else(|| "未检测到豆包桌面端".to_string())?;
    state
        .db
        .save_provider(APP_ID, &native_provider(account, Some(provider)))
        .map_err(|e| e.to_string())?;
    Ok(true)
}

pub(crate) fn doubao_delete_provider(_state: &AppState, _id: &str) -> Result<bool, String> {
    Err("豆包原生账号不能在 CC Switch 中删除".to_string())
}

pub(crate) fn doubao_switch_provider(state: &AppState, id: &str) -> Result<SwitchResult, String> {
    if id != NATIVE_PROVIDER_ID || crate::doubao_config::native_account()?.is_none() {
        return Err("未检测到可用的豆包桌面端原生账号".to_string());
    }
    state
        .db
        .set_current_provider(APP_ID, NATIVE_PROVIDER_ID)
        .map_err(|e| e.to_string())?;
    Ok(SwitchResult { warnings: vec![] })
}

pub(crate) fn doubao_update_sort_order(
    state: &AppState,
    updates: Vec<ProviderSortUpdate>,
) -> Result<bool, String> {
    if let Some(update) = updates.iter().find(|item| item.id == NATIVE_PROVIDER_ID) {
        if let Some(mut provider) = state
            .db
            .get_provider_by_id(NATIVE_PROVIDER_ID, APP_ID)
            .map_err(|e| e.to_string())?
        {
            provider.sort_index = Some(update.sort_index);
            state
                .db
                .save_provider(APP_ID, &provider)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(true)
}

#[tauri::command]
pub fn import_doubao_account_from_live(state: State<'_, AppState>) -> Result<usize, String> {
    sync_native(state.inner()).map(|(count, _)| count)
}

#[tauri::command]
pub fn get_doubao_live_provider_ids() -> Result<Vec<String>, String> {
    Ok(if crate::doubao_config::native_account()?.is_some() {
        vec![NATIVE_PROVIDER_ID.to_string()]
    } else {
        vec![]
    })
}

fn value_as_f64(value: Option<&Value>) -> Option<f64> {
    value.and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_str()?.trim().parse::<f64>().ok())
    })
}

fn quota_window_kind(value: &Value) -> Option<&'static str> {
    let kind = value.get("window_type")?;
    if let Some(number) = kind.as_i64() {
        return match number {
            1 => Some("5小时"),
            2 => Some("7天"),
            _ => None,
        };
    }
    let kind = kind.as_str()?.to_ascii_uppercase();
    if kind == "H5" || kind.contains("5H") || kind.contains("5_HOUR") {
        Some("5小时")
    } else if kind == "D7" || kind.contains("7D") || kind.contains("7_DAY") {
        Some("7天")
    } else {
        None
    }
}

fn format_reset_time(value: Option<&Value>) -> Option<String> {
    let number = value_as_f64(value)? as i64;
    let seconds = if number > 10_000_000_000 {
        number / 1000
    } else {
        number
    };
    if seconds <= 0 {
        return None;
    }
    chrono::DateTime::from_timestamp(seconds, 0).map(|time| time.to_rfc3339())
}

fn parse_window(value: &Value) -> Option<UsageData> {
    let plan_name = quota_window_kind(value)?;
    let mut used = value_as_f64(value.get("used_percent"));
    if used.is_none() {
        let used_amount = value_as_f64(value.get("used_amount"));
        let total_amount = value_as_f64(value.get("total_amount"));
        used = used_amount
            .zip(total_amount)
            .and_then(|(used, total)| (total > 0.0).then_some(used / total * 100.0));
    }
    let mut used = used?;
    if (0.0..=1.0).contains(&used) {
        used *= 100.0;
    }
    used = used.clamp(0.0, 100.0);
    Some(UsageData {
        plan_name: Some(plan_name.to_string()),
        remaining: Some(100.0 - used),
        total: Some(100.0),
        used: Some(used),
        unit: Some("%".to_string()),
        is_valid: Some(true),
        invalid_message: None,
        extra: format_reset_time(value.get("end_time")),
    })
}

fn collect_windows(value: &Value, data: &mut Vec<UsageData>) {
    if let Some(window) = parse_window(value) {
        let name = window.plan_name.as_deref();
        if !data.iter().any(|item| item.plan_name.as_deref() == name) {
            data.push(window);
        }
    }
    match value {
        Value::Array(items) => items.iter().for_each(|item| collect_windows(item, data)),
        Value::Object(map) => map.values().for_each(|item| collect_windows(item, data)),
        _ => {}
    }
}

fn parse_quota_response(value: &Value) -> Result<Vec<UsageData>, String> {
    let mut data = Vec::new();
    collect_windows(value, &mut data);
    data.sort_by_key(|item| match item.plan_name.as_deref() {
        Some("5小时") => 0,
        Some("7天") => 1,
        _ => 2,
    });
    if data.is_empty() {
        let message = value
            .get("message")
            .or_else(|| value.get("msg"))
            .or_else(|| value.get("status_msg"))
            .and_then(Value::as_str)
            .filter(|message| !message.trim().is_empty())
            .unwrap_or("豆包未返回可显示的 5 小时/7 天额度；请确认已登录并开通套餐");
        Err(message.to_string())
    } else {
        Ok(data)
    }
}

async fn query_native_usage(
    profile_name: String,
    timeout_secs: u64,
) -> Result<UsageResult, String> {
    let session = tauri::async_runtime::spawn_blocking(move || {
        crate::doubao_config::load_web_session(&profile_name)
    })
    .await
    .map_err(|e| format!("读取豆包登录态任务失败: {e}"))??;

    let mut query = vec![
        ("version_code", "20800".to_string()),
        ("language", "zh".to_string()),
        ("device_platform", "web".to_string()),
        ("doubao_device_platform", "web".to_string()),
        ("aid", "497858".to_string()),
        ("real_aid", "497858".to_string()),
        ("pkg_type", "release_version".to_string()),
        ("pc_version", "0.1.0".to_string()),
        ("doubao_pc_version", "0.1.0".to_string()),
        ("region", "CN".to_string()),
        ("sys_region", "CN".to_string()),
        ("samantha_web", "1".to_string()),
        ("web_platform", "desktop".to_string()),
        ("use-olympus-account", "1".to_string()),
        ("runtime", "web".to_string()),
        ("runtime_version", "3.36.4".to_string()),
        ("client_platform", "pc_client".to_string()),
        ("chromium_version", "147.0.7727.149".to_string()),
        ("channel", "mac_official".to_string()),
    ];
    if !session.device_id.is_empty() {
        query.push(("device_id", session.device_id.clone()));
        query.push(("web_id", session.device_id.clone()));
        query.push(("tea_uuid", session.device_id.clone()));
        query.push(("fp", format!("verify_{}", session.device_id)));
    }
    if !session.ms_token.is_empty() {
        query.push(("msToken", session.ms_token.clone()));
    }
    let request_id = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    query.push(("web_tab_id", uuid::Uuid::new_v4().to_string()));
    query.push((
        "a_bogus",
        format!("mf-{}-{}", &request_id[..34], &request_id[..6]),
    ));

    let client = crate::proxy::http_client::get();
    let request = if let Some(signed_url) = session.signed_quota_url {
        client.post(signed_url)
    } else {
        client.post(QUOTA_URL).query(&query)
    };
    let response = request
        .json(&json!({ "product_line": "membership" }))
        .header(reqwest::header::ACCEPT, "application/json, text/plain, */*")
        .header(reqwest::header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9")
        .header(reqwest::header::CACHE_CONTROL, "no-cache")
        .header(reqwest::header::COOKIE, session.cookie_header)
        .header(reqwest::header::REFERER, QUOTA_REFERER)
        .header(reqwest::header::ORIGIN, "https://www.doubao.com")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-origin")
        .header(
            "sec-ch-ua",
            "\"Chromium\";v=\"147\", \"Not_A Brand\";v=\"99\"",
        )
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
        .header("agw-js-conv", "str")
        .header("x-flow-trace", format!("04-{}-{}-01", uuid::Uuid::new_v4(), &request_id[..16]))
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36",
        )
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .send()
        .await
        .map_err(|error| {
            if error.is_timeout() {
                "连接豆包额度接口超时".to_string()
            } else {
                "连接豆包额度接口失败".to_string()
            }
        })?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("读取豆包额度响应失败: {e}"))?;
    if !status.is_success() {
        return Ok(UsageResult {
            success: false,
            data: None,
            error: Some(format!("豆包额度接口返回 HTTP {}", status.as_u16())),
        });
    }
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("豆包额度响应格式异常: {e}"))?;
    match parse_quota_response(&value) {
        Ok(data) => Ok(UsageResult {
            success: true,
            data: Some(data),
            error: None,
        }),
        Err(error) => Ok(UsageResult {
            success: false,
            data: None,
            error: Some(error),
        }),
    }
}

#[tauri::command]
pub async fn query_doubao_provider_usage(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] providerId: String,
) -> Result<UsageResult, String> {
    let provider = state
        .db
        .get_provider_by_id(&providerId, APP_ID)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "供应商不存在".to_string())?;
    let script = provider
        .meta
        .as_ref()
        .and_then(|meta| meta.usage_script.as_ref())
        .ok_or_else(|| "未配置用量查询".to_string())?;
    if !script.enabled {
        return Err("用量查询未启用".to_string());
    }
    let profile_name = provider
        .settings_config
        .get("profileName")
        .and_then(Value::as_str)
        .ok_or_else(|| "豆包原生 Profile 信息缺失，请重新导入".to_string())?
        .to_string();
    query_native_usage(profile_name, script.timeout.unwrap_or(15)).await
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn test_doubao_usage_script(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] providerId: String,
    #[allow(non_snake_case)] _scriptCode: String,
    timeout: Option<u64>,
    #[allow(non_snake_case)] _apiKey: Option<String>,
    #[allow(non_snake_case)] _baseUrl: Option<String>,
    #[allow(non_snake_case)] _accessToken: Option<String>,
    #[allow(non_snake_case)] _userId: Option<String>,
    #[allow(non_snake_case)] _templateType: Option<String>,
) -> Result<UsageResult, String> {
    let provider = state
        .db
        .get_provider_by_id(&providerId, APP_ID)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "供应商不存在".to_string())?;
    let profile_name = provider
        .settings_config
        .get("profileName")
        .and_then(Value::as_str)
        .ok_or_else(|| "豆包原生 Profile 信息缺失，请重新导入".to_string())?
        .to_string();
    query_native_usage(profile_name, timeout.unwrap_or(15)).await
}

#[tauri::command]
pub async fn stream_check_doubao_provider(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] providerId: String,
) -> Result<crate::services::stream_check::StreamCheckResult, String> {
    use crate::services::stream_check::{HealthStatus, StreamCheckResult};

    state
        .db
        .get_provider_by_id(&providerId, APP_ID)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "供应商不存在".to_string())?;
    let config = crate::services::stream_check::StreamCheckConfig::default();
    let started = std::time::Instant::now();
    let response = crate::proxy::http_client::get()
        .get("https://www.doubao.com/")
        .header("accept", "text/html")
        .header("accept-encoding", "identity")
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .send()
        .await;
    let latency = started.elapsed().as_millis() as u64;
    let result = match response {
        Ok(response) => StreamCheckResult {
            status: if latency <= config.degraded_threshold_ms {
                HealthStatus::Operational
            } else {
                HealthStatus::Degraded
            },
            success: true,
            message: "Reachable".to_string(),
            response_time_ms: Some(latency),
            http_status: Some(response.status().as_u16()),
            tested_at: chrono::Utc::now().timestamp(),
            retry_count: 0,
        },
        Err(error) => StreamCheckResult {
            status: HealthStatus::Failed,
            success: false,
            message: error.to_string(),
            response_time_ms: Some(latency),
            http_status: None,
            tested_at: chrono::Utc::now().timestamp(),
            retry_count: 0,
        },
    };
    Ok(result)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubaoStatus {
    installed: bool,
    profile_name: Option<String>,
    cookie_database_available: bool,
}

#[tauri::command]
pub fn get_doubao_status() -> DoubaoStatus {
    let installed = crate::doubao_config::doubao_installed();
    let profile_name = crate::doubao_config::active_profile_name().ok();
    let cookie_database_available = profile_name.as_ref().is_some_and(|profile| {
        crate::doubao_config::support_dir()
            .is_ok_and(|root| root.join(profile).join("Cookies").is_file())
    });
    DoubaoStatus {
        installed,
        profile_name,
        cookie_database_available,
    }
}

#[tauri::command]
pub fn open_doubao_app() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if !crate::doubao_config::doubao_installed() {
            return Err("Doubao.app 未安装".to_string());
        }
        let status = std::process::Command::new("open")
            .args(["-b", "com.bot.pc.doubao"])
            .status()
            .map_err(|e| format!("启动豆包失败: {e}"))?;
        return if status.success() {
            Ok(())
        } else {
            Err(format!("启动豆包失败: {status}"))
        };
    }
    #[cfg(not(target_os = "macos"))]
    Err("豆包桌面端原生集成目前仅支持 macOS".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_five_hour_and_seven_day_windows() {
        let response = json!({
            "data": {
                "window_limit_groups": [
                    {"window_type": "H5", "used_percent": 0.21, "end_time": 1_900_000_000},
                    {"window_type": "D7", "used_amount": 29, "total_amount": 100, "end_time": 1_900_010_000}
                ]
            }
        });
        let parsed = parse_quota_response(&response).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].plan_name.as_deref(), Some("5小时"));
        assert_eq!(parsed[0].used, Some(21.0));
        assert_eq!(parsed[1].plan_name.as_deref(), Some("7天"));
        assert!((parsed[1].used.unwrap_or_default() - 29.0).abs() < 1e-9);
        assert_eq!(format_reset_time(Some(&json!(0))), None);
    }

    #[tokio::test]
    #[ignore = "requires a local logged-in Doubao desktop profile"]
    async fn live_doubao_quota() {
        let profile = crate::doubao_config::active_profile_name().unwrap();
        let result = query_native_usage(profile, 15).await.unwrap();
        assert!(result.success, "{result:?}");
        let data = result.data.unwrap_or_default();
        assert!(data
            .iter()
            .any(|item| item.plan_name.as_deref() == Some("5小时")));
        assert!(data
            .iter()
            .any(|item| item.plan_name.as_deref() == Some("7天")));
    }
}
