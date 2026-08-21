//! SurrealDBへスキーマを明示的に適用する（2026/08/21/002.md）。
//!
//! `ingest`/`companies`/`repack`は起動のたびにスキーマを再適用していたため、
//! 既存の全レコードが無駄に書き直され、DBが肥大化する原因になっていた
//! （実測で最大15倍の増幅）。スキーマ適用はこのコマンドでのみ、必要な時に
//! 明示的に行う。ingest/companies/repackは一切スキーマを適用しない。

use anyhow::{Context, Result};
use std::path::PathBuf;
use surrealdb::engine::local::RocksDb;
use surrealdb::Surreal;

// アプリ本体（src-tauri）と同じスキーマファイルを共有する。
const SCHEMA_SQL: &str = include_str!("../../../../schema.surql");

pub fn run(db_path: &PathBuf) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(db_path))
}

async fn run_async(db_path: &PathBuf) -> Result<()> {
    let db_path_str = db_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("DBパスが不正です: {}", db_path.display()))?;
    let db = Surreal::new::<RocksDb>(db_path_str)
        .await
        .with_context(|| format!("SurrealDB初期化失敗: {}", db_path.display()))?;
    db.use_ns("kabuto").use_db("kabuto").await?;

    println!("スキーマを適用中: {}", db_path.display());
    db.query(SCHEMA_SQL).await?.check()?;
    println!("完了");
    Ok(())
}
