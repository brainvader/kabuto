//! master.parquetの銘柄情報をSurrealDBのcompanyテーブルへ投入する（2026/08/18/002.md）。
//! APIは呼ばない。証券コードを主キー（type::thing('company', code)）としてUPSERTする。
//! スキーマは適用しない（`init-db`サブコマンドで先に適用しておくこと、2026/08/21/002.md）。

use anyhow::{Context, Result};
use polars::prelude::*;
use std::{fs, io::Write, path::PathBuf};
use surrealdb::engine::local::SurrealKv;
use surrealdb::Surreal;

struct CompanyRow {
    code: String,
    name: String,
    sector: Option<String>,
    market: Option<String>,
    edinet_code: Option<String>,
}

pub fn run(master_path: &PathBuf, db_path: &PathBuf) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(master_path, db_path))
}

async fn run_async(master_path: &PathBuf, db_path: &PathBuf) -> Result<()> {
    let db_path_str = db_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("DBパスが不正です: {}", db_path.display()))?;
    let db = Surreal::new::<SurrealKv>(db_path_str)
        .await
        .with_context(|| format!("SurrealDB初期化失敗: {}", db_path.display()))?;
    db.use_ns("kabuto").use_db("kabuto").await?;

    let rows = read_companies(master_path)?;
    println!("company: {} 件を読み込み（{}）", rows.len(), master_path.display());

    let mut count = 0usize;
    for chunk in rows.chunks(500) {
        let batch: Vec<serde_json::Value> = chunk
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.code,
                    "code": r.code,
                    "name": r.name,
                    "sector": r.sector,
                    "market": r.market,
                    "edinet_code": r.edinet_code,
                })
            })
            .collect();

        // CONTENTではなくSETを使う。CONTENTは既存レコードへのUPSERT時、
        // 渡さなかったフィールド（updated_at等のDEFAULT付き）を空にしてしまい
        // スキーマ違反になることがある（2026/08/21/001.mdで実際に発生）。
        db.query(
            "FOR $row IN $batch { \
               UPSERT type::thing('company', $row.id) SET \
                 code = $row.code, name = $row.name, sector = $row.sector, \
                 market = $row.market, edinet_code = $row.edinet_code; \
             }",
        )
            .bind(("batch", batch))
            .await?
            .check()?;

        count += chunk.len();
        print!("\r  company 投入: {} / {} 件", count, rows.len());
        std::io::stdout().flush().ok();
    }
    println!();
    Ok(())
}

fn read_companies(master_path: &PathBuf) -> Result<Vec<CompanyRow>> {
    let file = fs::File::open(master_path)
        .with_context(|| format!("master.parquetを開けません: {}", master_path.display()))?;
    let df = ParquetReader::new(file)
        .finish()
        .map_err(|e| anyhow::anyhow!("master.parquet読み込み失敗: {}", e))?;

    let code_col = df.column("証券コード")?.str()?;
    let name_col = df.column("銘柄名")?.str()?;
    let sector_col = df.column("33業種区分")?.str()?;
    let market_col = df.column("市場区分")?.str()?;
    let edinet_col = df.column("EDINETコード")?.str()?;

    let mut rows = Vec::new();
    for i in 0..df.height() {
        let (Some(code), Some(name)) = (code_col.get(i), name_col.get(i)) else {
            continue;
        };
        rows.push(CompanyRow {
            code: code.to_string(),
            name: name.to_string(),
            sector: sector_col.get(i).map(|s| s.to_string()),
            market: market_col.get(i).map(|s| s.to_string()),
            edinet_code: edinet_col.get(i).map(|s| s.to_string()),
        });
    }
    Ok(rows)
}
