//! financial_metric の型定義（2026/08/17/001.md：財務諸表ダッシュボードの前提）。
//! フィールド構成は schema.surql と対応させている。

use serde::{Deserialize, Serialize};

/// XBRLの5期比較サマリーから抽出した財務指標1件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialMetricRecord {
    pub doc_id: String,
    /// 証券コード。master.parquetのEDINETコード対応表から解決（未対応の書類はNone）。
    pub company_code: Option<String>,
    pub metric: String,
    pub xbrl_tag: String,
    pub value: f64,
    pub unit: String,
    pub fiscal_year: i32,
    pub consolidated: bool,
}
