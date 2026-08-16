//! decision モジュールを Tauri command として公開する層（2026/08/15/007.md Step 6）。

use super::store;
use super::types::{ConsideredEdge, Decision, DecisionSession, DecisionSessionSummary, Trigger};
use crate::runtime;
use uuid::Uuid;

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

#[tauri::command]
pub fn create_trigger_cmd(trigger: Trigger) -> Result<String, String> {
    let id = new_id();
    runtime()
        .block_on(store::create_trigger(&id, &trigger))
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub fn create_decision_session_cmd(session: DecisionSession) -> Result<String, String> {
    let id = new_id();
    runtime()
        .block_on(store::create_decision_session(&id, &session))
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub fn create_decision_cmd(decision: Decision) -> Result<String, String> {
    let id = new_id();
    runtime()
        .block_on(store::create_decision(&id, &decision))
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub fn relate_prompted_cmd(trigger_id: String, session_id: String) -> Result<(), String> {
    runtime()
        .block_on(store::relate_prompted(&trigger_id, &session_id))
        .map_err(|e| e.to_string())
}

/// target_table は "decision" | "decision_session"（入れ子のdecision_sessionを検討した場合）。
#[tauri::command]
pub fn relate_considered_cmd(
    session_id: String,
    target_table: String,
    target_id: String,
    selected: bool,
    rejection_reason: Option<String>,
) -> Result<(), String> {
    runtime()
        .block_on(store::relate_considered(
            &session_id,
            &target_table,
            &target_id,
            selected,
            rejection_reason.as_deref(),
        ))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resolve_decision_session_cmd(
    id: String,
    conclusion: String,
    action_taken: String,
) -> Result<(), String> {
    runtime()
        .block_on(store::resolve_decision_session(&id, &conclusion, &action_taken))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_decision_session_cmd(id: String) -> Result<Option<DecisionSession>, String> {
    runtime()
        .block_on(store::get_decision_session(&id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_decision_sessions_cmd() -> Result<Vec<DecisionSessionSummary>, String> {
    runtime()
        .block_on(store::list_decision_sessions())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_decision_cmd(id: String) -> Result<Option<Decision>, String> {
    runtime()
        .block_on(store::get_decision(&id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_considered_cmd(session_id: String) -> Result<Vec<ConsideredEdge>, String> {
    runtime()
        .block_on(store::list_considered(&session_id))
        .map_err(|e| e.to_string())
}
