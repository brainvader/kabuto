//! decision モジュールを Tauri command として公開する層（2026/08/15/007.md Step 6）。
//!
//! すべて async fn として実装し、Tauri自身が管理する非同期ランタイム上で実行させる。
//! 独自の tokio::runtime::Runtime に block_on していた旧実装は、Tauri（WebView2の
//! IPCハンドラ）が既にtokioランタイムのワーカースレッド上でコマンドを呼び出すことがあり、
//! 「Cannot start a runtime from within a runtime」で実機クラッシュする不具合があった。

use super::store;
use super::types::{ConsideredEdge, Decision, DecisionSession, DecisionSessionSummary, Trigger};
use uuid::Uuid;

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

#[tauri::command]
pub async fn create_trigger_cmd(trigger: Trigger) -> Result<String, String> {
    let id = new_id();
    store::create_trigger(&id, &trigger).await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn create_decision_session_cmd(session: DecisionSession) -> Result<String, String> {
    let id = new_id();
    store::create_decision_session(&id, &session)
        .await
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn create_decision_cmd(decision: Decision) -> Result<String, String> {
    let id = new_id();
    store::create_decision(&id, &decision).await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn relate_prompted_cmd(trigger_id: String, session_id: String) -> Result<(), String> {
    store::relate_prompted(&trigger_id, &session_id)
        .await
        .map_err(|e| e.to_string())
}

/// target_table は "decision" | "decision_session"（入れ子のdecision_sessionを検討した場合）。
#[tauri::command]
pub async fn relate_considered_cmd(
    session_id: String,
    target_table: String,
    target_id: String,
    selected: bool,
    rejection_reason: Option<String>,
) -> Result<(), String> {
    store::relate_considered(
        &session_id,
        &target_table,
        &target_id,
        selected,
        rejection_reason.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_decision_session_cmd(
    id: String,
    conclusion: String,
    action_taken: String,
) -> Result<(), String> {
    store::resolve_decision_session(&id, &conclusion, &action_taken)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_decision_session_cmd(id: String) -> Result<Option<DecisionSession>, String> {
    store::get_decision_session(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_decision_sessions_cmd() -> Result<Vec<DecisionSessionSummary>, String> {
    store::list_decision_sessions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_decision_cmd(id: String) -> Result<Option<Decision>, String> {
    store::get_decision(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_considered_cmd(session_id: String) -> Result<Vec<ConsideredEdge>, String> {
    store::list_considered(&session_id).await.map_err(|e| e.to_string())
}
