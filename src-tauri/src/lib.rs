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

fn db() -> &'static Surreal<Db> {
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

const SCHEMA_SQL: &str = r#"
-- company ノード
-- 証券コードを主キーとする最小スキーマ
-- 後から DEFINE FIELD で拡張可能
DEFINE TABLE company SCHEMAFULL;
DEFINE FIELD code ON TABLE company TYPE string;
DEFINE FIELD name ON TABLE company TYPE string;

-- macro ノード（MacroIndicator）
-- 将来: WTI / USDJPY / NIKKEI / FED_RATE など
DEFINE TABLE macro SCHEMAFULL;
DEFINE FIELD name     ON TABLE macro TYPE string;
DEFINE FIELD category ON TABLE macro TYPE string; -- COMMODITY | CURRENCY | INDEX | INTEREST_RATE
DEFINE FIELD source   ON TABLE macro TYPE string;

-- AFFECTED_BY エッジ（company → macro）
-- 将来: LLM が抽出した仮説エッジ
DEFINE TABLE affected_by SCHEMAFULL;
DEFINE FIELD confidence ON TABLE affected_by TYPE option<float>;
DEFINE FIELD confirmed  ON TABLE affected_by TYPE bool DEFAULT false;
DEFINE FIELD lag_days   ON TABLE affected_by TYPE option<int>;
"#;

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

// ── Tauri エントリポイント ────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![init_db, search_stocks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
