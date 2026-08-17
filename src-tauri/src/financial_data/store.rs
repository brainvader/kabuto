//! financial_metric のSurrealDB読み取り。

use super::types::FinancialMetricRecord;
use crate::db;
use anyhow::Result;

/// 指定した証券コードの財務指標を、決算年の昇順で返す。
pub async fn list_by_company(company_code: &str) -> Result<Vec<FinancialMetricRecord>> {
    let mut resp = db()
        .await
        .query("SELECT * FROM financial_metric WHERE company_code = $code ORDER BY fiscal_year ASC")
        .bind(("code", company_code.to_string()))
        .await?;
    Ok(resp.take(0)?)
}

// ── テスト ────────────────────────────────────────────────────────────────────
// decision::store と同じ考え方で、本番の data/kabuto.db を汚さない isolated な
// SurrealKv インスタンスに対してクエリを検証する。
#[cfg(test)]
mod tests {
    use super::super::types::FinancialMetricRecord;
    use surrealdb::engine::local::{Db, SurrealKv};
    use surrealdb::Surreal;

    async fn isolated_db(path: &str) -> Surreal<Db> {
        let _ = std::fs::remove_dir_all(path);
        let db = Surreal::new::<SurrealKv>(path).await.expect("SurrealDB 初期化失敗");
        db.use_ns("kabuto_test")
            .use_db("kabuto_test")
            .await
            .expect("NS/DB 選択失敗");
        db.query(include_str!("../../schema.surql"))
            .await
            .expect("スキーマ適用失敗");
        db
    }

    #[tokio::test]
    async fn list_by_company_filters_and_orders_by_fiscal_year() {
        let db = isolated_db("data/test_financial_metrics.db").await;

        db.query(
            "UPSERT type::thing('financial_metric', 'docA_Revenue_2025') CONTENT \
             { doc_id: 'docA', company_code: '1605', metric: 'Revenue', xbrl_tag: 'RevenueSummaryOfBusinessResults', value: 100.0, unit: 'JPY', fiscal_year: 2025, consolidated: true }",
        )
        .await
        .expect("upsert失敗")
        .check()
        .expect("upsertにクエリエラー");
        db.query(
            "UPSERT type::thing('financial_metric', 'docA_Revenue_2024') CONTENT \
             { doc_id: 'docA', company_code: '1605', metric: 'Revenue', xbrl_tag: 'RevenueSummaryOfBusinessResults', value: 90.0, unit: 'JPY', fiscal_year: 2024, consolidated: true }",
        )
        .await
        .expect("upsert失敗")
        .check()
        .expect("upsertにクエリエラー");
        db.query(
            "UPSERT type::thing('financial_metric', 'docB_Revenue_2025') CONTENT \
             { doc_id: 'docB', company_code: '7203', metric: 'Revenue', xbrl_tag: 'RevenueSummaryOfBusinessResults', value: 500.0, unit: 'JPY', fiscal_year: 2025, consolidated: true }",
        )
        .await
        .expect("upsert失敗")
        .check()
        .expect("upsertにクエリエラー");

        let mut resp = db
            .query("SELECT * FROM financial_metric WHERE company_code = $code ORDER BY fiscal_year ASC")
            .bind(("code", "1605".to_string()))
            .await
            .expect("クエリ失敗")
            .check()
            .expect("クエリエラー");
        let rows: Vec<FinancialMetricRecord> = resp.take(0).expect("デシリアライズ失敗");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].fiscal_year, 2024);
        assert_eq!(rows[1].fiscal_year, 2025);
        assert!(rows.iter().all(|r| r.company_code.as_deref() == Some("1605")));
    }
}
