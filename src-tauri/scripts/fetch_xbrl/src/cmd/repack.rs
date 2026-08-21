//! 肥大化した`data/kabuto.db`を、新しい空のDBへバッチコピーして詰め直す（2026/08/18/003.md）。
//! RocksDbのvalue logに積み上がった古いバージョンを持ち込まないため、
//! `export`/`import`は使わず、テーブルごとに読み出し→書き込みを行う。
//! embeddingは再計算せずそのまま複製する（OpenAI API呼び出しなし）。
//!
//! `--limit`で1回の実行で新規コピーする件数の上限を指定できる（未指定なら無制限）。
//! コピー先に既にあるIDはスキップするため、`--limit`で打ち切っても次回そのまま再開できる。
//!
//! スキーマは適用しない。実行前に`dst`へ`init-db`サブコマンドを1回実行しておくこと。
//! 各レコードは1回のUPSERT ... SETで、companyへのレコードリンクも含めて書き切る
//! （2026/08/21/001.md・002.md：CONTENT＋別クエリのUPDATEという2段階にすると
//! embeddingを含むレコード全体が2回書き込まれてDBが肥大化することが実測で分かったため）。

use anyhow::{Context, Result};
use serde::Deserialize;
use std::{io::Write, path::PathBuf};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;

const BATCH_SIZE: usize = 500;

pub fn run(src: &PathBuf, dst: &PathBuf, limit: Option<usize>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(src, dst, limit))
}

async fn run_async(src: &PathBuf, dst: &PathBuf, limit: Option<usize>) -> Result<()> {
    let src_str = src
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("srcパスが不正です: {}", src.display()))?;
    let dst_str = dst
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("dstパスが不正です: {}", dst.display()))?;

    let src_db = Surreal::new::<RocksDb>(src_str)
        .await
        .with_context(|| format!("コピー元DB初期化失敗: {}", src.display()))?;
    src_db.use_ns("kabuto").use_db("kabuto").await?;

    let dst_db = Surreal::new::<RocksDb>(dst_str)
        .await
        .with_context(|| format!("コピー先DB初期化失敗: {}", dst.display()))?;
    dst_db.use_ns("kabuto").use_db("kabuto").await?;

    let mut budget = limit;
    budget = copy_company(&src_db, &dst_db, budget).await?;
    budget = copy_financial_metric(&src_db, &dst_db, budget).await?;
    budget = copy_disclosure_text(&src_db, &dst_db, budget).await?;

    match budget {
        Some(0) => println!("上限に達したため打ち切りました。同じコマンドをもう一度実行すると続きから再開します。"),
        _ => println!("詰め替え完了。trigger/decision/decision_session/considered/prompted/macroは0件のためコピー対象外（2026/08/18時点で確認済み）。"),
    }
    Ok(())
}

/// SurrealDB 3.xはNULL（明示的な空値）とNONE（未設定）を区別し、
/// `option<T>`型フィールドへのNULL代入をエラーにする。serde_jsonは
/// Option::Noneを`null`にするため、バインド前にnullキーを取り除いて
/// 未設定（NONE相当）にする。
fn strip_nulls(mut v: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(map) = &mut v {
        map.retain(|_, val| !val.is_null());
    }
    v
}

/// `count()`だけの軽いクエリで件数を取る。src/dstが既に同じ件数なら、
/// そのテーブルの全件読み込みそのものをスキップするために使う。
async fn count_of(db: &Surreal<Db>, table: &str) -> Result<i64> {
    #[derive(Deserialize, SurrealValue)]
    struct CountRow {
        count: i64,
    }
    let mut resp = db.query(format!("SELECT count() FROM {table} GROUP ALL")).await?.check()?;
    let rows: Vec<CountRow> = resp.take(0)?;
    Ok(rows.first().map(|r| r.count).unwrap_or(0))
}

async fn existing_ids(dst: &Surreal<Db>, table: &str) -> Result<std::collections::HashSet<String>> {
    let mut resp = dst
        .query(format!("SELECT <string>id AS id FROM {table}"))
        .await?
        .check()?;
    #[derive(Deserialize, SurrealValue)]
    struct IdRow {
        id: String,
    }
    let rows: Vec<IdRow> = resp.take(0)?;
    // 例: "company:`1605`" → "1605"。数字始まりのID（証券コード等）は
    // <string>キャストした文字列表現ではバッククォートでエスケープされるため、
    // prefix除去だけでなくバッククォートも剥がす必要がある（テストで実測して確認）。
    Ok(rows
        .into_iter()
        .map(|r| {
            r.id
                .trim_start_matches(&format!("{table}:"))
                .trim_matches('`')
                .to_string()
        })
        .collect())
}

#[derive(Deserialize, SurrealValue)]
struct CompanyRow {
    code: String,
    name: String,
    sector: Option<String>,
    market: Option<String>,
    edinet_code: Option<String>,
}

async fn copy_company(src: &Surreal<Db>, dst: &Surreal<Db>, limit: Option<usize>) -> Result<Option<usize>> {
    if count_of(src, "company").await? == count_of(dst, "company").await? {
        println!("company: 件数が一致しているためスキップ");
        return Ok(limit);
    }
    let mut resp = src
        .query("SELECT code, name, sector, market, edinet_code FROM company")
        .await?
        .check()?;
    let rows: Vec<CompanyRow> = resp.take(0)?;
    let done = existing_ids(dst, "company").await?;
    let mut todo: Vec<&CompanyRow> = rows.iter().filter(|r| !done.contains(&r.code)).collect();
    if let Some(n) = limit {
        todo.truncate(n);
    }
    println!("company: {} 件中 {} 件が未コピー、{} 件コピーします", rows.len(), rows.len() - done.len(), todo.len());

    let mut count = 0usize;
    for chunk in todo.chunks(BATCH_SIZE) {
        let batch: Vec<serde_json::Value> = chunk
            .iter()
            .map(|r| {
                strip_nulls(serde_json::json!({
                    "id": r.code,
                    "code": r.code, "name": r.name,
                    "sector": r.sector, "market": r.market, "edinet_code": r.edinet_code,
                }))
            })
            .collect();
        dst.query(
            "FOR $row IN $batch { \
               UPSERT type::record('company', $row.id) SET \
                 code = $row.code, name = $row.name, sector = $row.sector, \
                 market = $row.market, edinet_code = $row.edinet_code; \
             }",
        )
            .bind(("batch", batch))
            .await?
            .check()?;
        count += chunk.len();
        print!("\r  company: {} / {} 件", count, todo.len());
        std::io::stdout().flush().ok();
    }
    if !todo.is_empty() {
        println!();
    }
    Ok(limit.map(|n| n.saturating_sub(todo.len())))
}

#[derive(Deserialize, SurrealValue)]
struct FinancialMetricRow {
    doc_id: String,
    company_code: Option<String>,
    metric: String,
    xbrl_tag: String,
    value: f64,
    unit: String,
    fiscal_year: i32,
    consolidated: bool,
}

async fn copy_financial_metric(src: &Surreal<Db>, dst: &Surreal<Db>, limit: Option<usize>) -> Result<Option<usize>> {
    if limit == Some(0) {
        return Ok(limit);
    }
    if count_of(src, "financial_metric").await? == count_of(dst, "financial_metric").await? {
        println!("financial_metric: 件数が一致しているためスキップ");
        return Ok(limit);
    }
    let mut resp = src
        .query(
            "SELECT doc_id, company.code AS company_code, metric, xbrl_tag, value, unit, fiscal_year, consolidated \
             FROM financial_metric",
        )
        .await?
        .check()?;
    let rows: Vec<FinancialMetricRow> = resp.take(0)?;
    let done = existing_ids(dst, "financial_metric").await?;
    let mut todo: Vec<&FinancialMetricRow> = rows
        .iter()
        .filter(|r| !done.contains(&format!("{}_{}_{}", r.doc_id, r.metric, r.fiscal_year)))
        .collect();
    if let Some(n) = limit {
        todo.truncate(n);
    }
    println!("financial_metric: {} 件中 {} 件が未コピー、{} 件コピーします", rows.len(), rows.len() - done.len(), todo.len());

    let mut count = 0usize;
    for chunk in todo.chunks(BATCH_SIZE) {
        let batch: Vec<serde_json::Value> = chunk
            .iter()
            .map(|r| {
                let id = format!("{}_{}_{}", r.doc_id, r.metric, r.fiscal_year);
                strip_nulls(serde_json::json!({
                    "id": id,
                    "doc_id": r.doc_id, "metric": r.metric, "xbrl_tag": r.xbrl_tag,
                    "value": r.value, "unit": r.unit,
                    "fiscal_year": r.fiscal_year, "consolidated": r.consolidated,
                    "company_code": r.company_code,
                }))
            })
            .collect();
        dst.query(
            "FOR $row IN $batch { \
               UPSERT type::record('financial_metric', $row.id) SET \
                 doc_id = $row.doc_id, metric = $row.metric, xbrl_tag = $row.xbrl_tag, \
                 value = $row.value, unit = $row.unit, \
                 fiscal_year = $row.fiscal_year, consolidated = $row.consolidated, \
                 company = IF $row.company_code != NONE \
                            THEN type::record('company', $row.company_code) \
                            ELSE NONE END; \
             }",
        )
            .bind(("batch", batch))
            .await?
            .check()?;

        count += chunk.len();
        print!("\r  financial_metric: {} / {} 件", count, todo.len());
        std::io::stdout().flush().ok();
    }
    if !todo.is_empty() {
        println!();
    }
    Ok(limit.map(|n| n.saturating_sub(todo.len())))
}

#[derive(Deserialize, SurrealValue)]
struct DisclosureKey {
    doc_id: String,
    section: String,
}

#[derive(Deserialize, SurrealValue)]
struct DisclosureTextRow {
    doc_id: String,
    company_code: Option<String>,
    section: String,
    fiscal_year: i32,
    text: String,
    embedding: Vec<f64>,
}

/// disclosure_textはembedding（1536次元の配列）を含み1件あたりのデータ量が大きいため、
/// company/financial_metricとは違い「全件読み込んでからRust側で絞り込む」方式にしない。
/// 先にdoc_id/sectionだけの軽いクエリでIDを組み立て、コピー先に無い分だけを
/// バッチ単位でembedding込みの本体を取りに行く（毎回全件のembeddingを読む無駄を避ける）。
async fn copy_disclosure_text(src: &Surreal<Db>, dst: &Surreal<Db>, limit: Option<usize>) -> Result<Option<usize>> {
    if limit == Some(0) {
        return Ok(limit);
    }
    let mut resp = src.query("SELECT doc_id, section FROM disclosure_text").await?.check()?;
    let keys: Vec<DisclosureKey> = resp.take(0)?;
    let done = existing_ids(dst, "disclosure_text").await?;
    let mut todo_ids: Vec<String> = keys
        .iter()
        .map(|k| format!("{}_{}", k.doc_id, k.section))
        .filter(|id| !done.contains(id))
        .collect();
    if let Some(n) = limit {
        todo_ids.truncate(n);
    }
    println!(
        "disclosure_text: {} 件中 {} 件が未コピー、{} 件コピーします",
        keys.len(),
        keys.len() - done.len(),
        todo_ids.len()
    );

    let mut count = 0usize;
    for chunk in todo_ids.chunks(BATCH_SIZE) {
        // <string>id はテーブル名込み（例: "disclosure_text:docA_BusinessRisks"）で
        // 返るため、比較対象のIDにも同じ形式でテーブル名を含める必要がある
        // （これを見落とし、裸のIDのまま渡していたために0件しかヒットせず、
        // 進捗バーは100%まで進むのに実際は何もコピーされない、というバグが実際に起きた）。
        let prefixed_ids: Vec<String> = chunk.iter().map(|id| format!("disclosure_text:{id}")).collect();
        let mut resp = src
            .query(
                "SELECT doc_id, company.code AS company_code, section, fiscal_year, text, embedding \
                 FROM disclosure_text WHERE <string>id IN $ids",
            )
            .bind(("ids", prefixed_ids))
            .await?
            .check()?;
        let rows: Vec<DisclosureTextRow> = resp.take(0)?;
        if rows.len() != chunk.len() {
            anyhow::bail!(
                "disclosure_textのコピー中に件数不一致（期待{}件、実際{}件）。IDの組み立てかクエリを確認してください。",
                chunk.len(),
                rows.len()
            );
        }

        let batch: Vec<serde_json::Value> = rows
            .iter()
            .map(|r| {
                let id = format!("{}_{}", r.doc_id, r.section);
                strip_nulls(serde_json::json!({
                    "id": id,
                    "doc_id": r.doc_id, "section": r.section, "fiscal_year": r.fiscal_year,
                    "text": r.text, "embedding": r.embedding,
                    "company_code": r.company_code,
                }))
            })
            .collect();
        dst.query(
            "FOR $row IN $batch { \
               UPSERT type::record('disclosure_text', $row.id) SET \
                 doc_id = $row.doc_id, section = $row.section, fiscal_year = $row.fiscal_year, \
                 text = $row.text, embedding = $row.embedding, \
                 company = IF $row.company_code != NONE \
                            THEN type::record('company', $row.company_code) \
                            ELSE NONE END; \
             }",
        )
            .bind(("batch", batch))
            .await?
            .check()?;

        count += rows.len();
        print!("\r  disclosure_text: {} / {} 件", count, todo_ids.len());
        std::io::stdout().flush().ok();
    }
    if !todo_ids.is_empty() {
        println!();
    }
    Ok(limit.map(|n| n.saturating_sub(todo_ids.len())))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA_SQL: &str = include_str!("../../../../schema.surql");

    async fn isolated_db(path: &str) -> Surreal<Db> {
        let _ = std::fs::remove_dir_all(path);
        let db = Surreal::new::<RocksDb>(path).await.expect("SurrealDB 初期化失敗");
        db.use_ns("kabuto_test").use_db("kabuto_test").await.expect("NS/DB 選択失敗");
        db.query(SCHEMA_SQL).await.expect("スキーマ適用失敗").check().expect("スキーマにクエリエラー");
        db
    }

    /// 数字始まりの証券コード（<string>キャストするとcompany:`1301`のように
    /// バッククォートでエスケープされるID）が、existing_idsで正しく元の
    /// コード文字列に戻ることを確認する回帰テスト。
    #[tokio::test]
    async fn existing_ids_strips_backtick_escaping() {
        let db = isolated_db("data/test_repack_existing_ids.db").await;
        db.query("CREATE company:⟨1301⟩ SET code = '1301', name = 'テスト水産'")
            .await
            .expect("作成失敗")
            .check()
            .expect("作成にクエリエラー");

        let ids = existing_ids(&db, "company").await.expect("existing_ids失敗");

        assert!(ids.contains("1301"), "バッククォートが剥がれず {ids:?} に '1301' が含まれない");
    }

    /// copy_disclosure_textが使っている`WHERE <string>id IN $ids`が、
    /// 実際にIDでレコードを絞り込めるかを確認する回帰テスト。
    #[tokio::test]
    async fn string_id_in_clause_matches_rows() {
        let db = isolated_db("data/test_repack_id_in.db").await;
        db.query(
            "CREATE disclosure_text:⟨docA_BusinessRisks⟩ SET doc_id = 'docA', section = 'BusinessRisks', fiscal_year = 2025, text = 'x', embedding = [];
             CREATE disclosure_text:⟨docB_BusinessRisks⟩ SET doc_id = 'docB', section = 'BusinessRisks', fiscal_year = 2025, text = 'y', embedding = [];",
        )
        .await
        .expect("作成失敗")
        .check()
        .expect("作成にクエリエラー");

        let ids = vec!["disclosure_text:docA_BusinessRisks".to_string()];
        let mut resp = db
            .query("SELECT doc_id FROM disclosure_text WHERE <string>id IN $ids")
            .bind(("ids", ids))
            .await
            .expect("クエリ失敗")
            .check()
            .expect("クエリエラー");
        #[derive(serde::Deserialize, Debug, SurrealValue)]
        struct Row {
            doc_id: String,
        }
        let rows: Vec<Row> = resp.take(0).expect("デシリアライズ失敗");

        assert_eq!(rows.len(), 1, "IN句が1件ヒットするはずが {rows:?} だった");
    }

    /// 新設計（UPSERT ... SET、companyリンク込みの1回書き込み）で、
    /// company/financial_metric/disclosure_textを1件ずつコピーし、
    /// リンクも正しく張れることを確認する。
    #[tokio::test]
    async fn single_write_copy_preserves_data_and_links() {
        let src = isolated_db("data/test_repack_src.db").await;
        let dst = isolated_db("data/test_repack_dst.db").await;

        src.query("CREATE company:⟨1301⟩ SET code = '1301', name = 'テスト水産'")
            .await.expect("作成失敗").check().expect("作成エラー");
        src.query(
            "UPSERT type::record('financial_metric', 'docA_Revenue_2025') SET \
               doc_id = 'docA', metric = 'Revenue', xbrl_tag = 'x', value = 100.0, unit = 'JPY', \
               fiscal_year = 2025, consolidated = true, company = company:⟨1301⟩;",
        )
        .await.expect("作成失敗").check().expect("作成エラー");
        src.query(
            "UPSERT type::record('disclosure_text', 'docA_BusinessRisks') SET \
               doc_id = 'docA', section = 'BusinessRisks', fiscal_year = 2025, text = 'リスクです', \
               embedding = [0.1, 0.2], company = company:⟨1301⟩;",
        )
        .await.expect("作成失敗").check().expect("作成エラー");

        copy_company(&src, &dst, None).await.expect("company失敗");
        copy_financial_metric(&src, &dst, None).await.expect("financial_metric失敗");
        copy_disclosure_text(&src, &dst, None).await.expect("disclosure_text失敗");

        #[derive(Deserialize, Debug, SurrealValue)]
        struct FmRow { value: f64, company_name: Option<String> }
        let mut resp = dst
            .query("SELECT `value` AS value, company.name AS company_name FROM financial_metric:⟨docA_Revenue_2025⟩")
            .await.expect("クエリ失敗").check().expect("クエリエラー");
        let fm: Vec<FmRow> = resp.take(0).unwrap();
        assert_eq!(fm.len(), 1);
        assert_eq!(fm[0].value, 100.0);
        assert_eq!(fm[0].company_name.as_deref(), Some("テスト水産"));

        #[derive(Deserialize, Debug, SurrealValue)]
        struct DtRow { text: String, embedding: Vec<f64>, company_name: Option<String> }
        let mut resp2 = dst
            .query("SELECT text, embedding, company.name AS company_name FROM disclosure_text:⟨docA_BusinessRisks⟩")
            .await.expect("クエリ失敗").check().expect("クエリエラー");
        let dt: Vec<DtRow> = resp2.take(0).unwrap();
        assert_eq!(dt.len(), 1);
        assert_eq!(dt[0].text, "リスクです");
        assert_eq!(dt[0].embedding, vec![0.1, 0.2]);
        assert_eq!(dt[0].company_name.as_deref(), Some("テスト水産"));
    }
}
