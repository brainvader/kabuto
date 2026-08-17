//! `parse`の出力（financial_metrics.jsonl / disclosure_texts.jsonl）を読み、
//! SurrealDBへ投入する。disclosure_textは未embeddingのものだけOpenAIで
//! embeddingしてから投入する（2026/08/15/007.md Step 3）。
//!
//! アプリ本体（src-tauri）と同じ埋め込みSurrealKvエンジンを使うため、
//! アプリを起動したまま実行しないこと（同じDBファイルを同時に開けない）。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;

const EMBEDDING_MODEL: &str = "text-embedding-3-small";
const EMBEDDING_BATCH_SIZE: usize = 50;
/// 1件あたりの文字数上限。8,191トークンの入力上限に対する安全マージン。
const MAX_TEXT_CHARS: usize = 6000;

// アプリ本体（src-tauri）と同じスキーマファイルを共有する。
const SCHEMA_SQL: &str = include_str!("../../../../schema.surql");

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct DisclosureTextRow {
    doc_id: String,
    company_code: Option<String>,
    section: String,
    fiscal_year: i32,
    text: String,
}

#[derive(Deserialize)]
struct ExistingKey {
    doc_id: String,
    section: String,
}

pub fn run(input_dir: &PathBuf, db_path: &PathBuf) -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow::anyhow!(".env に OPENAI_API_KEY が設定されていません"))?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(input_dir, db_path, &api_key))
}

async fn run_async(input_dir: &PathBuf, db_path: &PathBuf, api_key: &str) -> Result<()> {
    let db_path_str = db_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("DBパスが不正です: {}", db_path.display()))?;
    let db = Surreal::new::<SurrealKv>(db_path_str)
        .await
        .with_context(|| format!("SurrealDB初期化失敗: {}", db_path.display()))?;
    db.use_ns("kabuto").use_db("kabuto").await?;

    println!("スキーマを適用中...");
    db.query(SCHEMA_SQL).await?;

    ingest_financial_metrics(&db, input_dir).await?;
    ingest_disclosure_texts(&db, input_dir, api_key).await?;

    Ok(())
}

async fn ingest_financial_metrics(db: &Surreal<Db>, input_dir: &PathBuf) -> Result<()> {
    let path = input_dir.join("financial_metrics.jsonl");
    let rows: Vec<FinancialMetricRow> = read_jsonl(&path)?;
    println!("financial_metric: {} 件を読み込み", rows.len());

    let mut count = 0usize;
    for chunk in rows.chunks(500) {
        let batch: Vec<serde_json::Value> = chunk
            .iter()
            .map(|r| {
                let id = format!("{}_{}_{}", r.doc_id, r.metric, r.fiscal_year);
                serde_json::json!({ "id": id, "data": r })
            })
            .collect();

        db.query("FOR $row IN $batch { UPSERT type::thing('financial_metric', $row.id) CONTENT $row.data; }")
            .bind(("batch", batch))
            .await?;

        count += chunk.len();
        print!("\r  financial_metric 投入: {} / {} 件", count, rows.len());
        std::io::stdout().flush().ok();
    }
    println!();
    Ok(())
}

async fn ingest_disclosure_texts(db: &Surreal<Db>, input_dir: &PathBuf, api_key: &str) -> Result<()> {
    let path = input_dir.join("disclosure_texts.jsonl");
    let rows: Vec<DisclosureTextRow> = read_jsonl(&path)?;
    println!("disclosure_text: {} 件を読み込み", rows.len());

    // 既に投入済み（= embedding済み）の(doc_id, section)を取得し、未処理分だけに絞る。
    // これがdisclosure_textの冪等性・embedding再計算防止の要になる。
    let mut existing_resp = db.query("SELECT doc_id, section FROM disclosure_text").await?;
    let existing: Vec<ExistingKey> = existing_resp.take(0)?;
    let existing_keys: HashSet<(String, String)> = existing
        .into_iter()
        .map(|e| (e.doc_id, e.section))
        .collect();

    let to_embed: Vec<&DisclosureTextRow> = rows
        .iter()
        .filter(|r| !existing_keys.contains(&(r.doc_id.clone(), r.section.clone())))
        .collect();

    println!(
        "未embedding: {} / {} 件（{} 件は投入済みのためスキップ）",
        to_embed.len(),
        rows.len(),
        rows.len() - to_embed.len()
    );

    let mut embedded = 0usize;
    for chunk in to_embed.chunks(EMBEDDING_BATCH_SIZE) {
        let texts: Vec<String> = chunk.iter().map(|r| truncate(&r.text, MAX_TEXT_CHARS)).collect();
        let embeddings = embed_batch(api_key, &texts)
            .await
            .with_context(|| format!("embedding失敗（{}件目のバッチ）", embedded))?;

        let batch: Vec<serde_json::Value> = chunk
            .iter()
            .zip(embeddings.iter())
            .map(|(r, emb)| {
                let id = format!("{}_{}", r.doc_id, r.section);
                serde_json::json!({
                    "id": id,
                    "data": {
                        "doc_id": r.doc_id,
                        "company_code": r.company_code,
                        "section": r.section,
                        "fiscal_year": r.fiscal_year,
                        "text": r.text,
                        "embedding": emb,
                    }
                })
            })
            .collect();

        db.query("FOR $row IN $batch { UPSERT type::thing('disclosure_text', $row.id) CONTENT $row.data; }")
            .bind(("batch", batch))
            .await?;

        embedded += chunk.len();
        print!("\r  disclosure_text embedding+投入: {} / {} 件", embedded, to_embed.len());
        std::io::stdout().flush().ok();
    }
    println!();

    backfill_company_code(db, &rows).await?;
    Ok(())
}

/// 既にembedding済み（= 上のembed_batchでスキップされた）disclosure_textにも
/// company_codeを反映する。embeddingの再計算を避けるため、CONTENTでの
/// 全体上書きではなくフィールド単位のUPDATEにする（2026/08/16/002.md）。
///
/// レコードIDは ⟨doc_id⟩_⟨section⟩ で決定的に分かっているため、`WHERE doc_id = ...`
/// のような検索ベースの更新はしない。doc_id列にインデックスが無い状態でこれを
/// やった結果、実データ（34,429件）に対して2,372回のフルスキャンが発生し、
/// 完了しないほど遅くなった実測結果がある（2026/08/17/003.md）。主キー直接指定なら
/// テーブルサイズに関係なく高速。
async fn backfill_company_code(db: &Surreal<Db>, rows: &[DisclosureTextRow]) -> Result<()> {
    let updates: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let id = format!("{}_{}", r.doc_id, r.section);
            serde_json::json!({ "id": id, "company_code": r.company_code })
        })
        .collect();

    println!("company_codeのバックフィル対象: {} 件", updates.len());

    let mut done = 0usize;
    for chunk in updates.chunks(500) {
        db.query(
            "FOR $row IN $batch { UPDATE type::thing('disclosure_text', $row.id) SET company_code = $row.company_code; }",
        )
        .bind(("batch", chunk.to_vec()))
        .await?;
        done += chunk.len();
        print!("\r  company_codeバックフィル: {} / {} 件", done, updates.len());
        std::io::stdout().flush().ok();
    }
    println!();
    Ok(())
}

fn read_jsonl<T: for<'de> Deserialize<'de>>(path: &PathBuf) -> Result<Vec<T>> {
    let file = fs::File::open(path).with_context(|| format!("開けません: {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        rows.push(serde_json::from_str(&line).with_context(|| format!("JSON解析失敗: {}", line))?);
    }
    Ok(rows)
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        text.chars().take(max_chars).collect()
    }
}

async fn embed_batch(api_key: &str, texts: &[String]) -> Result<Vec<Vec<f64>>> {
    #[derive(Serialize)]
    struct EmbeddingRequest<'a> {
        model: &'a str,
        input: &'a [String],
    }
    #[derive(Deserialize)]
    struct EmbeddingItem {
        embedding: Vec<f64>,
    }
    #[derive(Deserialize)]
    struct EmbeddingResponse {
        data: Vec<EmbeddingItem>,
    }

    let client = reqwest::Client::new();
    let resp: EmbeddingResponse = client
        .post("https://api.openai.com/v1/embeddings")
        .bearer_auth(api_key)
        .json(&EmbeddingRequest {
            model: EMBEDDING_MODEL,
            input: texts,
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp.data.into_iter().map(|d| d.embedding).collect())
}
