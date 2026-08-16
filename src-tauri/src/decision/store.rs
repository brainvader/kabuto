//! decision / decision_session / trigger のSurrealDB読み書き。

use super::types::{ConsideredEdge, Decision, DecisionSession, DecisionSessionSummary, Trigger};
use crate::db;
use anyhow::Result;

pub async fn create_decision(id: &str, decision: &Decision) -> Result<()> {
    db()
        .await
        .query("UPSERT type::thing('decision', $id) CONTENT $data")
        .bind(("id", id.to_string()))
        .bind(("data", decision.clone()))
        .await?;
    Ok(())
}

pub async fn create_decision_session(id: &str, session: &DecisionSession) -> Result<()> {
    db()
        .await
        .query("UPSERT type::thing('decision_session', $id) CONTENT $data")
        .bind(("id", id.to_string()))
        .bind(("data", session.clone()))
        .await?;
    Ok(())
}

pub async fn create_trigger(id: &str, trigger: &Trigger) -> Result<()> {
    db()
        .await
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
        .await
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
        .await
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
        .await
        .query("SELECT * FROM type::thing('decision_session', $id)")
        .bind(("id", id.to_string()))
        .await?;
    let rows: Vec<DecisionSession> = resp.take(0)?;
    Ok(rows.into_iter().next())
}

pub async fn get_decision(id: &str) -> Result<Option<Decision>> {
    let mut resp = db()
        .await
        .query("SELECT * FROM type::thing('decision', $id)")
        .bind(("id", id.to_string()))
        .await?;
    let rows: Vec<Decision> = resp.take(0)?;
    Ok(rows.into_iter().next())
}

/// decision_session が検討した候補（decision または decision_session）へのエッジ一覧。
/// out はレコードリンクを文字列にキャストして返す（呼び出し側で table/id に分解する）。
pub async fn list_considered(session_id: &str) -> Result<Vec<ConsideredEdge>> {
    let mut resp = db()
        .await
        .query(
            "SELECT selected, rejection_reason, <string>out AS target FROM considered \
             WHERE in = type::thing('decision_session', $id)",
        )
        .bind(("id", session_id.to_string()))
        .await?;
    Ok(resp.take(0)?)
}

/// 一覧表示用にrecord idも含めて返す。DecisionSession自体（CONTENT書き込みに使う型）
/// にidフィールドを足すとSCHEMAFULLの検証に引っかかるため、別型で返す。
pub async fn list_decision_sessions() -> Result<Vec<DecisionSessionSummary>> {
    let mut resp = db()
        .await
        .query(
            "SELECT question, company, status, selection_mode, conclusion, action_taken, <string>id AS id \
             FROM decision_session ORDER BY created_at DESC",
        )
        .await?;
    Ok(resp.take(0)?)
}

/// decision_session に結論を確定させる（status を "resolved" にし、resolved_at を刻む）。
pub async fn resolve_decision_session(
    id: &str,
    conclusion: &str,
    action_taken: &str,
) -> Result<()> {
    db()
        .await
        .query(
            "UPDATE type::thing('decision_session', $id) SET \
             status = 'resolved', conclusion = $conclusion, action_taken = $action_taken, resolved_at = time::now()",
        )
        .bind(("id", id.to_string()))
        .bind(("conclusion", conclusion.to_string()))
        .bind(("action_taken", action_taken.to_string()))
        .await?;
    Ok(())
}

// ── テスト ────────────────────────────────────────────────────────────────────
// store.rs の各関数はグローバルシングルトン db()（本番の data/kabuto.db）に
// 直結しているため、本番データを汚さないよう isolated な SurrealKv インスタンスに
// 対して同一のクエリ文字列を直接発行し、SurrealQLの構文・挙動のみを検証する
// （lib.rs の schema_sql_applies_without_error と同じ考え方）。
#[cfg(test)]
mod tests {
    use surrealdb::engine::local::{Db, SurrealKv};
    use surrealdb::Surreal;

    async fn isolated_db(path: &str) -> Surreal<Db> {
        // UPSERT...CONTENT はDEFAULT付きフィールドを再適用しないため、
        // 前回実行時のレコードが残っているとcreated_at等の型チェックに失敗する。
        // 毎回まっさらな状態から始める。
        let _ = std::fs::remove_dir_all(path);
        let db = Surreal::new::<SurrealKv>(path).await.expect("SurrealDB 初期化失敗");
        db.use_ns("kabuto_test")
            .use_db("kabuto_test")
            .await
            .expect("NS/DB 選択失敗");
        db.query(include_str!("../../schema.surql"))
            .await
            .expect("スキーマ適用失敗");
        db
    }

    #[tokio::test]
    async fn list_considered_returns_related_targets_as_strings() {
        let db = isolated_db("data/test_considered.db").await;

        db.query("UPSERT type::thing('decision_session', 'sess1') CONTENT { question: 'q', status: 'exploring', selection_mode: 'exclusive', action_taken: 'none' }")
            .await
            .expect("session upsert失敗")
            .check()
            .expect("session upsertにクエリエラー");
        db.query("UPSERT type::thing('decision', 'dec1') CONTENT { hypothesis: 'h', method: 'pearson', confirmed: true, based_on_doc_ids: [] }")
            .await
            .expect("decision upsert失敗")
            .check()
            .expect("decision upsertにクエリエラー");
        db.query(
            "RELATE (type::thing('decision_session', 'sess1'))->considered->(type::thing('decision', 'dec1')) \
             SET selected = true, rejection_reason = NONE",
        )
        .await
        .expect("relate失敗")
        .check()
        .expect("relateにクエリエラー");

        let mut resp = db
            .query("SELECT selected, rejection_reason, <string>out AS target FROM considered WHERE in = type::thing('decision_session', 'sess1')")
            .await
            .expect("list_considered クエリ失敗");
        let rows: Vec<super::super::types::ConsideredEdge> = resp.take(0).expect("デシリアライズ失敗");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target, "decision:dec1");
        assert!(rows[0].selected);
        assert_eq!(rows[0].rejection_reason, None);
    }

    #[tokio::test]
    async fn list_decision_sessions_includes_record_id() {
        let db = isolated_db("data/test_list_sessions.db").await;

        db.query("UPSERT type::thing('decision_session', 'sessA') CONTENT { question: 'qA', status: 'exploring', selection_mode: 'exclusive', action_taken: 'none' }")
            .await
            .expect("upsert失敗")
            .check()
            .expect("upsertにクエリエラー");
        db.query("UPSERT type::thing('decision_session', 'sessB') CONTENT { question: 'qB', status: 'resolved', selection_mode: 'composite', action_taken: 'bought' }")
            .await
            .expect("upsert失敗")
            .check()
            .expect("upsertにクエリエラー");

        let mut resp = db
            .query(
                "SELECT question, company, status, selection_mode, conclusion, action_taken, <string>id AS id \
                 FROM decision_session ORDER BY question ASC",
            )
            .await
            .expect("list_decision_sessions クエリ失敗")
            .check()
            .expect("list_decision_sessionsにクエリエラー");
        let rows: Vec<super::super::types::DecisionSessionSummary> = resp.take(0).expect("デシリアライズ失敗");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "decision_session:sessA");
        assert_eq!(rows[0].question, "qA");
        assert_eq!(rows[1].id, "decision_session:sessB");
        assert_eq!(rows[1].action_taken, "bought");
    }
}
