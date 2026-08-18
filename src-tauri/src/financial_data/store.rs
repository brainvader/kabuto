//! financial_metric のSurrealDB読み取り。

use super::types::FinancialMetricRecord;
use crate::db;
use anyhow::Result;

/// 指定した証券コードの財務指標を、決算年の昇順で返す。
pub async fn list_by_company(company_code: &str) -> Result<Vec<FinancialMetricRecord>> {
    let mut resp = db()
        .await
        .query(
            "SELECT doc_id, company.code AS company_code, metric, xbrl_tag, value, unit, fiscal_year, consolidated \
             FROM financial_metric WHERE company = type::thing('company', $code) ORDER BY fiscal_year ASC",
        )
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

        db.query("CREATE company:⟨1605⟩ SET code = '1605', name = 'テスト石油'")
            .await
            .expect("company作成失敗")
            .check()
            .expect("company作成にクエリエラー");
        db.query("CREATE company:⟨7203⟩ SET code = '7203', name = 'テスト自動車'")
            .await
            .expect("company作成失敗")
            .check()
            .expect("company作成にクエリエラー");

        for (id, doc_id, code, value, fiscal_year) in [
            ("docA_Revenue_2025", "docA", "1605", 100.0, 2025),
            ("docA_Revenue_2024", "docA", "1605", 90.0, 2024),
            ("docB_Revenue_2025", "docB", "7203", 500.0, 2025),
        ] {
            db.query(
                "UPSERT type::thing('financial_metric', $id) CONTENT \
                 { doc_id: $doc_id, metric: 'Revenue', xbrl_tag: 'RevenueSummaryOfBusinessResults', value: $value, unit: 'JPY', fiscal_year: $fiscal_year, consolidated: true }",
            )
            .bind(("id", id))
            .bind(("doc_id", doc_id))
            .bind(("value", value))
            .bind(("fiscal_year", fiscal_year))
            .await
            .expect("upsert失敗")
            .check()
            .expect("upsertにクエリエラー");
            db.query("UPDATE type::thing('financial_metric', $id) SET company = type::thing('company', $code)")
                .bind(("id", id))
                .bind(("code", code))
                .await
                .expect("companyリンク失敗")
                .check()
                .expect("companyリンクにクエリエラー");
        }

        let mut resp = db
            .query(
                "SELECT doc_id, company.code AS company_code, metric, xbrl_tag, value, unit, fiscal_year, consolidated \
                 FROM financial_metric WHERE company = type::thing('company', $code) ORDER BY fiscal_year ASC",
            )
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
