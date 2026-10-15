use std::str::FromStr;
use tauri::{Emitter, State};

use crate::app_config::AppType;
use crate::services::subscription::SubscriptionQuota;
use crate::store::AppState;

/// 查询官方订阅额度
///
/// 读取 CLI 工具已有的 OAuth 凭据并调用官方 API 获取使用额度。
/// `Ok`（成功或确定性失败）写入 `UsageCache`、通知托盘刷新并 emit
/// `usage-cache-updated`，让前端 React Query 与托盘共享同一份最新数据；失败
/// 快照写入后 `format_subscription_summary` 会通过 `success=false` 守卫返回
/// `None`，托盘 suffix 自然消失，避免长期滞留旧配额数字。
/// `Err`（瞬时传输失败）不写快照、不 emit：保留上一份托盘快照，与前端
/// react-query reject 保留上次 data 的语义一致（emit 失败快照会经
/// `useUsageCacheBridge` 盲写回 query 缓存，抹掉本该保留的旧值）。
///
/// `provider_id`（仅 codex）：按供应商身份查询最新 live / 本机登录暂存，
/// 配置行中的 auth 只用于识别账号（见 `get_provider_codex_quota`）。缓存与托盘
/// 只反映「当前激活」账号：按 provider 查询仅当该条目恰为当前条目时才写
/// 快照并 emit，避免多个 OpenAI 账号各自轮询互相覆盖托盘数字。
#[tauri::command(rename_all = "camelCase")]
pub async fn get_subscription_quota(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    tool: String,
    provider_id: Option<String>,
) -> Result<SubscriptionQuota, String> {
    // provider 维度只对 codex 有意义；其他 tool 忽略该参数（保持共享路径）。
    let provider_scope = match (tool.as_str(), provider_id.as_deref()) {
        ("codex", Some(pid)) if !pid.trim().is_empty() => Some(pid.trim().to_owned()),
        _ => None,
    };

    let inner = match &provider_scope {
        Some(pid) => crate::services::subscription::get_provider_codex_quota(&state, pid).await,
        None => crate::services::subscription::get_subscription_quota(&tool).await,
    };

    if let Ok(snapshot) = &inner {
        if let Ok(app_type) = AppType::from_str(&tool) {
            let is_current_query = match &provider_scope {
                None => true,
                Some(pid) => {
                    crate::mode::current::provider_for(
                        &state.db,
                        &app_type,
                        crate::mode::current::Purpose::Direct,
                    )
                    .ok()
                    .flatten()
                        == Some(pid.clone())
                }
            };
            if is_current_query {
                let mut payload = serde_json::json!({
                    "kind": "subscription",
                    "appType": app_type.as_str(),
                    "data": snapshot,
                });
                if let Some(pid) = &provider_scope {
                    payload["providerId"] = serde_json::Value::String(pid.clone());
                }
                if let Err(e) = app.emit("usage-cache-updated", payload) {
                    log::error!("emit usage-cache-updated (subscription) 失败: {e}");
                }
                state
                    .usage_cache
                    .put_subscription(app_type, snapshot.clone());
                crate::tray::schedule_tray_refresh(&app);
            }
        }
    }
    inner
}
