use tauri::State;

use crate::kimicode_config;
use crate::store::AppState;

// ============================================================================
// Kimi Code Provider Commands
// ============================================================================

/// Import providers from Kimi Code live config to database.
///
/// Kimi Code uses additive mode — users may already have providers
/// configured in config.toml (including the official OAuth provider).
#[tauri::command]
pub fn import_kimicode_providers_from_live(state: State<'_, AppState>) -> Result<usize, String> {
    crate::services::provider::import_kimicode_providers_from_live(state.inner())
        .map_err(|e| e.to_string())
}

/// Get provider ids in the Kimi Code live config.
#[tauri::command]
pub fn get_kimicode_live_provider_ids() -> Result<Vec<String>, String> {
    kimicode_config::get_providers()
        .map(|providers| providers.keys().cloned().collect())
        .map_err(|e| e.to_string())
}

/// 直接启动 Kimi.app（聊天/工作客户端，com.moonshot.kimichat）。

#[tauri::command]
pub fn open_kimi_app() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // 先按 bundle id 精确启动，失败时退回按应用名（用户可能装在非标准位置）
        let by_bundle = std::process::Command::new("open")
            .args(["-b", "com.moonshot.kimichat"])
            .status();
        match by_bundle {
            Ok(status) if status.success() => return Ok(()),
            other => {
                log::debug!("open -b com.moonshot.kimichat 未成功: {other:?}，回退按应用名启动");
            }
        }
        let status = std::process::Command::new("open")
            .args(["-a", "Kimi"])
            .status()
            .map_err(|e| format!("启动 Kimi 失败: {e}"))?;
        return if status.success() {
            Ok(())
        } else {
            Err(format!("open 命令退出码: {status}"))
        };
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("当前平台不支持启动 Kimi".to_string())
    }
}
