//! company をTauri commandとして公開する層。
//!
//! コマンド名`search_stocks`は、master.parquet直読みだった旧実装からの互換名
//! （2026/08/18/002.md：フロントエンド`useStockSearch`/`StockSearch.tsx`を
//! 変更せずに済ませるため、SurrealDB経由に切り替えた後もそのまま維持する）。

use super::store;
use super::types::CompanyItem;

#[tauri::command]
pub async fn search_stocks(
    query: String,
    market: Option<String>,
    sector: Option<String>,
) -> Result<Vec<CompanyItem>, String> {
    store::search_by_name(&query, market.as_deref(), sector.as_deref())
        .await
        .map_err(|e| e.to_string())
}
