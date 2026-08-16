//! decision / decision_session / trigger のSurrealDB読み書き。

use super::types::{Decision, DecisionSession, Trigger};
use crate::db;
use anyhow::Result;

pub async fn create_decision(id: &str, decision: &Decision) -> Result<()> {
    db()
        .query("UPSERT type::thing('decision', $id) CONTENT $data")
        .bind(("id", id.to_string()))
        .bind(("data", decision.clone()))
        .await?;
    Ok(())
}

pub async fn create_decision_session(id: &str, session: &DecisionSession) -> Result<()> {
    db()
        .query("UPSERT type::thing('decision_session', $id) CONTENT $data")
        .bind(("id", id.to_string()))
        .bind(("data", session.clone()))
        .await?;
    Ok(())
}

pub async fn create_trigger(id: &str, trigger: &Trigger) -> Result<()> {
    db()
        .query("UPSERT type::thing('trigger', $id) CONTENT $data")
        .bind(("id", id.to_string()))
        .bind(("data", trigger.clone()))
        .await?;
    Ok(())
}

/// decision_session が候補（decision または decision_session）を検討したことを表す
/// エッジを張る。target_table は "decision" | "decision_session"。
pub async fn relate_considered(
    session_id: &str,
    target_table: &str,
    target_id: &str,
    selected: bool,
    rejection_reason: Option<&str>,
) -> Result<()> {
    db()
        .query(
            "RELATE (type::thing('decision_session', $session))->considered->(type::thing($table, $target)) \
             SET selected = $selected, rejection_reason = $reason",
        )
        .bind(("session", session_id.to_string()))
        .bind(("table", target_table.to_string()))
        .bind(("target", target_id.to_string()))
        .bind(("selected", selected))
        .bind(("reason", rejection_reason.map(|s| s.to_string())))
        .await?;
    Ok(())
}

/// trigger が decision_session を誘発したことを表すエッジを張る。
pub async fn relate_prompted(trigger_id: &str, session_id: &str) -> Result<()> {
    db()
        .query(
            "RELATE (type::thing('trigger', $trigger))->prompted->(type::thing('decision_session', $session))",
        )
        .bind(("trigger", trigger_id.to_string()))
        .bind(("session", session_id.to_string()))
        .await?;
    Ok(())
}

pub async fn get_decision_session(id: &str) -> Result<Option<DecisionSession>> {
    let mut resp = db()
        .query("SELECT * FROM type::thing('decision_session', $id)")
        .bind(("id", id.to_string()))
        .await?;
    let rows: Vec<DecisionSession> = resp.take(0)?;
    Ok(rows.into_iter().next())
}

pub async fn list_decision_sessions() -> Result<Vec<DecisionSession>> {
    let mut resp = db()
        .query("SELECT * FROM decision_session ORDER BY created_at DESC")
        .await?;
    Ok(resp.take(0)?)
}
