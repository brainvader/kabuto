mod decision;

use decision::commands::{
    create_decision_cmd, create_decision_session_cmd, create_trigger_cmd, get_decision_cmd,
    get_decision_session_cmd, list_considered_cmd, list_decision_sessions_cmd,
    relate_considered_cmd, relate_prompted_cmd, resolve_decision_session_cmd,
};
use polars::prelude::*;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::OnceLock;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;
use tokio::runtime::Runtime;

// ── Tokio ランタイム（グローバルシングルトン） ────────────────────────────────

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("tokio runtime"))
}

// ── SurrealDB インスタンス（グローバルシングルトン） ──────────────────────────

pub(crate) fn db() -> &'static Surreal<Db> {
    static DB: OnceLock<Surreal<Db>> = OnceLock::new();
    DB.get_or_init(|| {
        runtime().block_on(async {
            let db = Surreal::new::<SurrealKv>("data/kabuto.db")
                .await
                .expect("SurrealDB 初期化失敗");
            db.use_ns("kabuto")
                .use_db("kabuto")
                .await
                .expect("NS/DB 選択失敗");
            db
        })
    })
}

// ── SurrealDB スキーマ定義 ────────────────────────────────────────────────────
// scripts/fetch_xbrl の ingest コマンドとも共有する単一のスキーマファイル。
const SCHEMA_SQL: &str = include_str!("../schema.surql");

/// アプリ起動時に SurrealDB を初期化しスキーマを適用する。
/// 既存スキーマへの再適用は冪等（DEFINE は上書き）。
#[tauri::command]
fn init_db() -> Result<(), String> {
    runtime()
        .block_on(async { db().query(SCHEMA_SQL).await })
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── master.parquet 検索 ───────────────────────────────────────────────────────

/// master.parquet の1行に対応する銘柄情報
#[derive(Debug, Serialize)]
pub struct StockItem {
    pub code: String,
    pub name: String,
    pub market: String,
    pub sector: String,
}

/// master.parquet を Polars でフィルタリングして銘柄を検索する。
///
/// # Arguments
/// - `query`  — 証券コードまたは銘柄名の部分一致
/// - `market` — 市場区分フィルター（None で全市場）
/// - `sector` — 33業種区分フィルター（None で全業種）
///
/// # Returns
/// 最大 50 件の `StockItem` リスト
#[tauri::command]
fn search_stocks(
    query: String,
    market: Option<String>,
    sector: Option<String>,
) -> Result<Vec<StockItem>, String> {
    let path = resolve_parquet_path().map_err(|e| e.to_string())?;
    let df =
        LazyFrame::scan_parquet(&path, ScanArgsParquet::default()).map_err(|e| e.to_string())?;

    let q = query.to_lowercase();
    let mut lazy = df.filter(
        col("証券コード")
            .cast(DataType::String)
            .str()
            .to_lowercase()
            .str()
            .contains(lit(q.clone()), false)
            .or(col("銘柄名")
                .cast(DataType::String)
                .str()
                .to_lowercase()
                .str()
                .contains(lit(q), false)),
    );

    if let Some(m) = market {
        lazy = lazy.filter(col("市場区分").eq(lit(m)));
    }

    if let Some(s) = sector {
        lazy = lazy.filter(col("33業種区分").eq(lit(s)));
    }

    let result = lazy
        .select([
            col("証券コード"),
            col("銘柄名"),
            col("市場区分"),
            col("33業種区分"),
        ])
        .limit(50)
        .collect()
        .map_err(|e| e.to_string())?;

    let codes = result
        .column("証券コード")
        .map_err(|e| e.to_string())?
        .str()
        .map_err(|e| e.to_string())?;
    let names = result
        .column("銘柄名")
        .map_err(|e| e.to_string())?
        .str()
        .map_err(|e| e.to_string())?;
    let markets = result
        .column("市場区分")
        .map_err(|e| e.to_string())?
        .str()
        .map_err(|e| e.to_string())?;
    let sectors = result
        .column("33業種区分")
        .map_err(|e| e.to_string())?
        .str()
        .map_err(|e| e.to_string())?;

    let items = (0..result.height())
        .filter_map(|i| {
            Some(StockItem {
                code: codes.get(i)?.to_string(),
                name: names.get(i)?.to_string(),
                market: markets.get(i).unwrap_or("").to_string(),
                sector: sectors.get(i).unwrap_or("").to_string(),
            })
        })
        .collect();

    Ok(items)
}

fn resolve_parquet_path() -> anyhow::Result<PathBuf> {
    let candidates = [
        PathBuf::from("data/master.parquet"), // src-tauri/ 基準
        PathBuf::from("../data/master.parquet"),
        PathBuf::from("src-tauri/data/master.parquet"),
    ];
    for p in &candidates {
        if p.exists() {
            return Ok(p.clone());
        }
    }
    anyhow::bail!("master.parquet が見つかりません")
}

// ── テスト ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// SCHEMA_SQL が実際にSurrealQLとして構文エラーなく適用できることを確認する。
    /// 本番用の data/kabuto.db とは別のDBファイルを使い、状態を汚さない。
    #[test]
    fn schema_sql_applies_without_error() {
        runtime().block_on(async {
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
        });
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
