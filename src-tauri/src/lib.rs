mod company;
mod decision;
mod financial_data;

use company::commands::search_stocks;
use decision::commands::{
    create_decision_cmd, create_decision_session_cmd, create_trigger_cmd, get_decision_cmd,
    get_decision_session_cmd, list_considered_cmd, list_decision_sessions_cmd,
    relate_considered_cmd, relate_prompted_cmd, resolve_decision_session_cmd,
};
use financial_data::commands::list_financial_metrics_cmd;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;
use tokio::sync::OnceCell;

// ── SurrealDB インスタンス（グローバルシングルトン） ──────────────────────────
// Tauriコマンドは async fn として実装し、Tauri自身が管理する非同期ランタイム上で
// 実行させる。以前は独自に作った tokio::runtime::Runtime に対して block_on していたが、
// Tauri（WebView2のIPCハンドラ）が既にtokioランタイムのワーカースレッド上で
// コマンドを呼び出すことがあり、「Cannot start a runtime from within a runtime」で
// パニックしていた（実機で操作中にクラッシュする形で再現）。

pub(crate) async fn db() -> &'static Surreal<Db> {
    static DB: OnceCell<Surreal<Db>> = OnceCell::const_new();
    DB.get_or_init(|| async {
        let db = Surreal::new::<SurrealKv>("data/kabuto.db")
            .await
            .expect("SurrealDB 初期化失敗");
        db.use_ns("kabuto")
            .use_db("kabuto")
            .await
            .expect("NS/DB 選択失敗");
        db
    })
    .await
}

// ── SurrealDB スキーマ定義 ────────────────────────────────────────────────────
// scripts/fetch_xbrl の ingest コマンドとも共有する単一のスキーマファイル。
const SCHEMA_SQL: &str = include_str!("../schema.surql");

/// アプリ起動時に SurrealDB を初期化しスキーマを適用する。
/// 既存スキーマへの再適用は冪等（DEFINE は上書き）。
#[tauri::command]
async fn init_db() -> Result<(), String> {
    db().await.query(SCHEMA_SQL).await.map_err(|e| e.to_string())?;
    Ok(())
}

// ── テスト ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// SCHEMA_SQL が実際にSurrealQLとして構文エラーなく適用できることを確認する。
    /// 本番用の data/kabuto.db とは別のDBファイルを使い、状態を汚さない。
    #[tokio::test]
    async fn schema_sql_applies_without_error() {
        let test_db = Surreal::new::<SurrealKv>("data/test_schema.db")
            .await
            .expect("SurrealDB 初期化失敗");
        test_db
            .use_ns("kabuto_test")
            .use_db("kabuto_test")
            .await
            .expect("NS/DB 選択失敗");
        let mut response = test_db.query(SCHEMA_SQL).await.expect("スキーマ適用失敗");
        // クエリ内にエラーがあれば take で拾われる
        response.take::<Option<()>>(0).expect("スキーマにエラーがあります");
    }
}

// ── Tauri エントリポイント ────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            init_db,
            search_stocks,
            create_trigger_cmd,
            create_decision_session_cmd,
            create_decision_cmd,
            relate_prompted_cmd,
            relate_considered_cmd,
            resolve_decision_session_cmd,
            get_decision_session_cmd,
            list_decision_sessions_cmd,
            get_decision_cmd,
            list_considered_cmd,
            list_financial_metrics_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
