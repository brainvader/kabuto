//! financial_metric をTauri commandとして公開する層（2026/08/17/001.md）。

use super::store;
use super::types::FinancialMetricRecord;

#[tauri::command]
pub async fn list_financial_metrics_cmd(company_code: String) -> Result<Vec<FinancialMetricRecord>, String> {
    store::list_by_company(&company_code).await.map_err(|e| e.to_string())
}
