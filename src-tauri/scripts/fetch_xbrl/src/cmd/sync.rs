use anyhow::Result;
use chrono::{Duration, Local, NaiveDate};
use polars::prelude::*;
use std::{fs, path::PathBuf};

use crate::types::Filter;

/// codelist → master → fetch を一括実行する。
///
/// - codelist は毎回取得し直す（コスト・容量ともに小さいため）
/// - master は JPX銘柄一覧（data_j.csv）が既に配置されている前提で実行する。
///   JPXはEDINETのような単純なダウンロードAPIを持たないため、自動取得はしない
/// - fetch のフィルタは、master.parquet に含まれる個別株のEDINETコード全件を使う。
///   これにより「必要なデータ（個別株の有価証券報告書）」だけが対象になり、
///   REIT・ETF等はそもそもmasterの時点で除外されている
///
/// fetch 自体は既に取得済みの書類をスキップするため（fetch::already_fetched）、
/// 同じ期間を指定して再実行しても安全（冪等）。
#[allow(clippy::too_many_arguments)]
pub fn run(
    codelist_dir: &PathBuf,
    jpx_path: &PathBuf,
    master_path: &PathBuf,
    xbrl_output: &PathBuf,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    debug: bool,
) -> Result<()> {
    println!("=== Step 1/3: codelist ===");
    crate::cmd::codelist::run(codelist_dir)?;
    let codelist_csv = find_codelist_csv(codelist_dir)?;

    if !jpx_path.exists() {
        anyhow::bail!(
            "JPX銘柄一覧が見つかりません: {}\n\
             https://www.jpx.co.jp/markets/statistics-equities/misc/01.html から\n\
             data_j.csv をダウンロードして配置してから再実行してください。",
            jpx_path.display()
        );
    }

    println!("\n=== Step 2/3: master ===");
    crate::cmd::master::run(jpx_path, &codelist_csv, master_path)?;

    println!("\n=== Step 3/3: fetch ===");
    let edinet_codes = read_edinet_codes(master_path)?;
    println!("対象: {} 社（master.parquet の個別株全件）", edinet_codes.len());

    let api_key = std::env::var("EDINET_API_KEY")
        .map_err(|_| anyhow::anyhow!(".env に EDINET_API_KEY が設定されていません"))?;

    let to = to.unwrap_or_else(|| Local::now().date_naive());
    let from = from.unwrap_or_else(|| to - Duration::days(90));

    let filter = Filter {
        edinet_codes,
        sec_codes: vec![],
        names: vec![],
    };

    crate::cmd::fetch::run(&api_key, &filter, from, to, xbrl_output, debug)
}

fn find_codelist_csv(dir: &PathBuf) -> Result<PathBuf> {
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("csv") {
            return Ok(path);
        }
    }
    anyhow::bail!("codelist CSVが見つかりません: {}", dir.display())
}

fn read_edinet_codes(master_path: &PathBuf) -> Result<Vec<String>> {
    let file = fs::File::open(master_path)
        .map_err(|_| anyhow::anyhow!("master.parquetを開けません: {}", master_path.display()))?;
    let df = ParquetReader::new(file)
        .finish()
        .map_err(|e| anyhow::anyhow!("master.parquet読み込み失敗: {}", e))?;
    let col = df
        .column("EDINETコード")
        .map_err(|e| anyhow::anyhow!("EDINETコード列が見つかりません: {}", e))?;
    let ca = col
        .str()
        .map_err(|e| anyhow::anyhow!("EDINETコード列の型が不正です: {}", e))?;
    Ok(ca
        .into_iter()
        .filter_map(|v| v.map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .collect())
}
