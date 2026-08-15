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

-- financial_metric ノード（2026/08/14/005.md）
-- レコードIDは ⟨doc_id⟩_⟨metric⟩_⟨fiscal_year⟩ の複合キー。
-- 1つのXBRL提出書類（doc_id）から複数レコードが生成される。
DEFINE TABLE financial_metric SCHEMAFULL;
DEFINE FIELD doc_id       ON TABLE financial_metric TYPE string;
DEFINE FIELD metric       ON TABLE financial_metric TYPE string;  -- 正規化した指標名
DEFINE FIELD xbrl_tag     ON TABLE financial_metric TYPE string;  -- 元のタクソノミ要素名
DEFINE FIELD value        ON TABLE financial_metric TYPE float;
DEFINE FIELD unit         ON TABLE financial_metric TYPE string;
DEFINE FIELD fiscal_year  ON TABLE financial_metric TYPE int;
DEFINE FIELD consolidated ON TABLE financial_metric TYPE bool;

-- disclosure_text ノード（2026/08/14/005.md）
-- レコードIDは ⟨doc_id⟩_⟨section⟩ の複合キー。
-- embeddingはOpenAI text-embedding-3-small。対象は自然言語セクションのみ。
DEFINE TABLE disclosure_text SCHEMAFULL;
DEFINE FIELD doc_id      ON TABLE disclosure_text TYPE string;
DEFINE FIELD section     ON TABLE disclosure_text TYPE string;
DEFINE FIELD fiscal_year ON TABLE disclosure_text TYPE int;
DEFINE FIELD text        ON TABLE disclosure_text TYPE string;
DEFINE FIELD embedding   ON TABLE disclosure_text TYPE array<float>;

-- trigger ノード（2026/08/15/003.md）
-- 「なぜこの仮説を思いついたか」という思考の切っ掛けを記録する。
DEFINE TABLE trigger SCHEMAFULL;
DEFINE FIELD type        ON TABLE trigger TYPE string; -- human_note | news | disclosure_text_match | precedent_recall
DEFINE FIELD description ON TABLE trigger TYPE string;
DEFINE FIELD source_ref  ON TABLE trigger TYPE option<string>;
DEFINE FIELD created_at  ON TABLE trigger TYPE datetime DEFAULT time::now();

-- decision ノード（2026/08/15/002.md）
-- 1つの仮説検証の結果。based_on_doc_ids で financial_metric/disclosure_text の元データまで遡れる。
DEFINE TABLE decision SCHEMAFULL;
DEFINE FIELD hypothesis       ON TABLE decision TYPE string;
DEFINE FIELD method           ON TABLE decision TYPE string;
DEFINE FIELD correlation      ON TABLE decision TYPE option<float>;
DEFINE FIELD confirmed        ON TABLE decision TYPE bool DEFAULT false;
DEFINE FIELD lag_days         ON TABLE decision TYPE option<int>;
DEFINE FIELD based_on_doc_ids ON TABLE decision TYPE array<string>;
DEFINE FIELD computed_at      ON TABLE decision TYPE datetime DEFAULT time::now();

-- decision_session ノード（2026/08/15/002・003・004.md）
-- 選択肢の集合と結論。旧 company->AFFECTED_BY->macro はこの仕組みに統合し廃止した。
DEFINE TABLE decision_session SCHEMAFULL;
DEFINE FIELD question       ON TABLE decision_session TYPE string;
DEFINE FIELD company        ON TABLE decision_session TYPE option<record<company>>;
DEFINE FIELD status         ON TABLE decision_session TYPE string DEFAULT 'exploring';  -- exploring | resolved
DEFINE FIELD selection_mode ON TABLE decision_session TYPE string DEFAULT 'exclusive';  -- exclusive | composite
DEFINE FIELD conclusion     ON TABLE decision_session TYPE option<string>;
DEFINE FIELD action_taken   ON TABLE decision_session TYPE string DEFAULT 'none';  -- bought | sold | watched | none
DEFINE FIELD created_at     ON TABLE decision_session TYPE datetime DEFAULT time::now();
DEFINE FIELD resolved_at    ON TABLE decision_session TYPE option<datetime>;

-- RELATE エッジ
-- has_metric / has_disclosure: company が持つ財務データ・開示テキストへの参照
DEFINE TABLE has_metric SCHEMAFULL;
DEFINE TABLE has_disclosure SCHEMAFULL;

-- prompted: trigger が decision_session を誘発したことを表す
DEFINE TABLE prompted SCHEMAFULL;

-- considered: decision_session が検討した候補。decision にも decision_session にも
-- RELATE できる（ネストした判断を許容するため、IN/OUTの型は固定しない）。
DEFINE TABLE considered SCHEMAFULL;
DEFINE FIELD selected         ON TABLE considered TYPE bool;
DEFINE FIELD rejection_reason ON TABLE considered TYPE option<string>;
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
        .invoke_handler(tauri::generate_handler![init_db, search_stocks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
