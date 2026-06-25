use polars::prelude::*;
use serde::Serialize;
use std::path::PathBuf;

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
        PathBuf::from("../data/master.parquet"),
        PathBuf::from("data/master.parquet"),
    ];
    for p in &candidates {
        if p.exists() {
            return Ok(p.clone());
        }
    }
    anyhow::bail!("master.parquet が見つかりません")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![search_stocks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
