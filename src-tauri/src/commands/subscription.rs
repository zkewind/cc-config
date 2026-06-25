use tauri::State;

use crate::services::subscription::SubscriptionQuota;
use crate::store::AppState;

/// 查询官方订阅额度
#[tauri::command]
pub async fn get_subscription_quota(
    _state: State<'_, AppState>,
    tool: String,
) -> Result<SubscriptionQuota, String> {
    crate::services::subscription::get_subscription_quota(&tool).await
}
