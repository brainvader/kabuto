//! company の型定義（2026/08/18/002.md）。

use serde::{Deserialize, Serialize};

/// 銘柄検索の結果1件。フロントエンドの `StockItem`（旧 search_stocks、
/// master.parquet直読み時代）と互換の形にして、呼び出し側の変更を避ける。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyItem {
    pub code: String,
    pub name: String,
    pub market: String,
    pub sector: String,
}

/// SurrealDBの`company`テーブルから読み取る生の行（market/sectorはoption）。
#[derive(Debug, Deserialize)]
pub(crate) struct CompanyRow {
    pub code: String,
    pub name: String,
    pub market: Option<String>,
    pub sector: Option<String>,
}

impl From<CompanyRow> for CompanyItem {
    fn from(r: CompanyRow) -> Self {
        Self {
            code: r.code,
            name: r.name,
            market: r.market.unwrap_or_default(),
            sector: r.sector.unwrap_or_default(),
        }
    }
}
