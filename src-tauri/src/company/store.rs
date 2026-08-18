//! company のSurrealDB読み取り。

use super::types::{CompanyItem, CompanyRow};
use crate::db;
use anyhow::Result;

/// 証券コードまたは銘柄名の部分一致（大文字小文字を区別しない）で銘柄を検索する。
/// market/sectorを指定した場合はさらに絞り込む。最大50件。
pub async fn search_by_name(
    query: &str,
    market: Option<&str>,
    sector: Option<&str>,
) -> Result<Vec<CompanyItem>> {
    let mut sql = String::from(
        "SELECT code, name, market, sector FROM company \
         WHERE (string::lowercase(name) CONTAINS string::lowercase($q) \
                OR string::lowercase(code) CONTAINS string::lowercase($q))",
    );
    if market.is_some() {
        sql.push_str(" AND market = $market");
    }
    if sector.is_some() {
        sql.push_str(" AND sector = $sector");
    }
    sql.push_str(" ORDER BY name LIMIT 50");

    let mut q = db().await.query(sql).bind(("q", query.to_string()));
    if let Some(m) = market {
        q = q.bind(("market", m.to_string()));
    }
    if let Some(s) = sector {
        q = q.bind(("sector", s.to_string()));
    }

    let mut resp = q.await?;
    let rows: Vec<CompanyRow> = resp.take(0)?;
    Ok(rows.into_iter().map(CompanyItem::from).collect())
}

// ── テスト ────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
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
    async fn search_by_name_matches_case_insensitively_and_filters() {
        let db = isolated_db("data/test_company_search.db").await;

        db.query(
            "CREATE company:⟨1605⟩ SET code = '1605', name = 'INPEX CORPORATION', market = 'プライム', sector = '鉱業';
             CREATE company:⟨5020⟩ SET code = '5020', name = 'ENEOSホールディングス', market = 'プライム', sector = '石油・石炭製品';
             CREATE company:⟨7203⟩ SET code = '7203', name = 'トヨタ自動車', market = 'プライム', sector = '輸送用機器';",
        )
        .await
        .expect("投入失敗")
        .check()
        .expect("投入にクエリエラー");

        // 大文字小文字を区別しない部分一致
        let results = search_by_name_against(&db, "inpex", None, None).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "1605");

        // 証券コードでも検索できる
        let results = search_by_name_against(&db, "7203", None, None).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "トヨタ自動車");

        // sectorで絞り込み
        let results = search_by_name_against(&db, "プライム", None, Some("鉱業")).await;
        assert_eq!(results.len(), 0); // "プライム"はname/codeに含まれないのでヒットなし

        let results = search_by_name_against(&db, "ENEOS", None, Some("石油・石炭製品")).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "5020");
    }

    async fn search_by_name_against(
        db: &Surreal<Db>,
        query: &str,
        market: Option<&str>,
        sector: Option<&str>,
    ) -> Vec<CompanyItem> {
        let mut sql = String::from(
            "SELECT code, name, market, sector FROM company \
             WHERE (string::lowercase(name) CONTAINS string::lowercase($q) \
                    OR string::lowercase(code) CONTAINS string::lowercase($q))",
        );
        if market.is_some() {
            sql.push_str(" AND market = $market");
        }
        if sector.is_some() {
            sql.push_str(" AND sector = $sector");
        }
        sql.push_str(" ORDER BY name LIMIT 50");

        let mut q = db.query(sql).bind(("q", query.to_string()));
        if let Some(m) = market {
            q = q.bind(("market", m.to_string()));
        }
        if let Some(s) = sector {
            q = q.bind(("sector", s.to_string()));
        }
        let mut resp = q.await.expect("クエリ失敗").check().expect("クエリエラー");
        let rows: Vec<CompanyRow> = resp.take(0).expect("デシリアライズ失敗");
        rows.into_iter().map(CompanyItem::from).collect()
    }
}
