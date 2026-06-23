//! EDINET API v2 から有価証券報告書（XBRL）をダウンロードする CLI ツール。
//!
//! 使い方:
//!     cargo run -p fetch_xbrl -- --from 2025-06-01 --to 2025-06-10
//!     cargo run -p fetch_xbrl -- --from 2025-06-01 --to 2025-06-30 --sec-code 285A

use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate};
use clap::Parser;
use serde::Deserialize;
use serde_json::Value;
use std::{fs, io::Cursor, path::PathBuf, thread, time};

const BASE_URL: &str = "https://api.edinet-fsa.go.jp/api/v2";
const DEFAULT_OUTPUT: &str = "data/xbrl";

#[derive(Parser, Debug)]
#[command(
    name = "fetch_xbrl",
    about = "EDINET API v2 から有価証券報告書（XBRL）をダウンロード"
)]
struct Args {
    #[arg(long, value_name = "DATE")]
    from: NaiveDate,
    #[arg(long, value_name = "DATE")]
    to: NaiveDate,
    #[arg(long = "edinet-code", value_name = "CODE")]
    edinet_codes: Vec<String>,
    #[arg(long = "sec-code", value_name = "CODE")]
    sec_codes: Vec<String>,
    #[arg(long = "name", value_name = "NAME")]
    names: Vec<String>,
    #[arg(long, value_name = "DIR", default_value = DEFAULT_OUTPUT)]
    output: PathBuf,
    /// デバッグ: APIレスポンスの生JSONを出力する
    #[arg(long)]
    debug: bool,
}

// resultsの各フィールドはすべてnullになりうるため Option<String> で受ける
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Document {
    #[serde(rename = "docID")]
    doc_id: String,
    edinet_code: Option<String>,
    sec_code: Option<String>,
    filer_name: Option<String>,
    ordinance_code: Option<String>,
    form_code: Option<String>,
    doc_description: Option<String>,
    period_start: Option<String>,
    period_end: Option<String>,
    submit_date_time: Option<String>,
    // 仕様書にある残りのフィールドも受け取る（無視するが存在しないとパース失敗する場合があるため）
    #[serde(default)]
    seq_number: Option<u64>,
    #[serde(rename = "JCN")]
    jcn: Option<String>,
    fund_code: Option<String>,
    doc_type_code: Option<String>,
    issuer_edinet_code: Option<String>,
    subject_edinet_code: Option<String>,
    subsidiary_edinet_code: Option<String>,
    current_report_reason: Option<String>,
    #[serde(rename = "parentDocID")]
    parent_doc_id: Option<String>,
    ope_date_time: Option<String>,
    withdrawal_status: Option<String>,
    doc_info_edit_status: Option<String>,
    disclosure_status: Option<String>,
    xbrl_flag: Option<String>,
    pdf_flag: Option<String>,
    attach_doc_flag: Option<String>,
    english_doc_flag: Option<String>,
    csv_flag: Option<String>,
    legal_status: Option<String>,
}

impl Document {
    fn is_annual_report(&self) -> bool {
        self.ordinance_code.as_deref() == Some("010") && self.form_code.as_deref() == Some("030000")
    }
}

struct Filter {
    edinet_codes: Vec<String>,
    sec_codes: Vec<String>,
    names: Vec<String>,
}

impl Filter {
    fn is_empty(&self) -> bool {
        self.edinet_codes.is_empty() && self.sec_codes.is_empty() && self.names.is_empty()
    }

    fn matches(&self, doc: &Document) -> bool {
        if self.is_empty() {
            return true;
        }
        self.edinet_codes
            .iter()
            .any(|c| doc.edinet_code.as_deref() == Some(c.as_str()))
            || self
                .sec_codes
                .iter()
                .any(|c| doc.sec_code.as_deref() == Some(c.as_str()))
            || self.names.iter().any(|n| {
                doc.filer_name
                    .as_deref()
                    .map(|f| f.contains(n.as_str()))
                    .unwrap_or(false)
            })
    }
}

fn main() -> Result<()> {
    for base in &["scripts/fetch_xbrl", ".", "..", "../.."] {
        let path = std::path::Path::new(base).join(".env");
        if path.exists() {
            dotenv::from_path(&path).ok();
            println!(".env: {}", path.canonicalize()?.display());
            break;
        }
    }

    let args = Args::parse();
    let api_key =
        std::env::var("EDINET_API_KEY").context(".env に EDINET_API_KEY が設定されていません")?;

    let filter = Filter {
        edinet_codes: args.edinet_codes,
        sec_codes: args.sec_codes,
        names: args.names,
    };

    fs::create_dir_all(&args.output)?;
    println!("出力先: {}", args.output.canonicalize()?.display());
    println!("期間: {} 〜 {}", args.from, args.to);
    println!("{}", "─".repeat(60));

    let mut current = args.from;
    let mut found = 0;

    while current <= args.to {
        let date_str = current.format("%Y-%m-%d").to_string();

        match get_documents(&api_key, &date_str, args.debug) {
            Ok(docs) => {
                let matched: Vec<_> = docs
                    .iter()
                    .filter(|d| d.is_annual_report() && filter.matches(d))
                    .collect();

                if !matched.is_empty() {
                    println!("{}: {} 件の有価証券報告書", date_str, matched.len());
                }

                for doc in matched {
                    found += 1;
                    println!(
                        "  → {} ({}) {}",
                        doc.filer_name.as_deref().unwrap_or("不明"),
                        doc.edinet_code.as_deref().unwrap_or("-"),
                        doc.doc_description.as_deref().unwrap_or(""),
                    );
                    if let (Some(ps), Some(pe)) = (&doc.period_start, &doc.period_end) {
                        println!("    期間: {} 〜 {}", ps, pe);
                    }

                    let edinet_code = doc.edinet_code.as_deref().unwrap_or("unknown");
                    match download_and_extract(&api_key, &doc.doc_id, edinet_code, &args.output) {
                        Ok(out_dir) => {
                            let xbrl_files = find_files_by_ext(&out_dir, "xbrl");
                            println!(
                                "  ✓ XBRL: {} ファイル → {}",
                                xbrl_files.len(),
                                out_dir.display()
                            );
                        }
                        Err(e) => eprintln!("  × ダウンロード失敗: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("  {} エラー: {}", date_str, e),
        }

        current += Duration::days(1);
        thread::sleep(time::Duration::from_millis(500));
    }

    println!("{}", "─".repeat(60));
    println!("完了: {} 件取得", found);
    Ok(())
}

fn get_documents(api_key: &str, date: &str, debug: bool) -> Result<Vec<Document>> {
    let url = format!("{}/documents.json", BASE_URL);
    let client = reqwest::blocking::Client::new();
    let text = client
        .get(&url)
        .query(&[("date", date), ("type", "2"), ("Subscription-Key", api_key)])
        .timeout(std::time::Duration::from_secs(30))
        .send()?
        .error_for_status()?
        .text()?;

    if debug {
        eprintln!("[DEBUG {}]\n{}", date, &text[..text.len().min(1000)]);
    }

    // まず Value として受け取り results だけ取り出す
    let root: Value =
        serde_json::from_str(&text).with_context(|| format!("JSON全体のパース失敗 ({})", date))?;

    let results = match root.get("results") {
        Some(Value::Array(arr)) => arr.clone(),
        Some(Value::Null) | None => return Ok(vec![]),
        Some(other) => {
            if debug {
                eprintln!("[DEBUG] results の型が予想外: {:?}", other);
            }
            return Ok(vec![]);
        }
    };

    let mut docs = Vec::new();
    for item in &results {
        match serde_json::from_value::<Document>(item.clone()) {
            Ok(doc) => docs.push(doc),
            Err(e) => {
                if debug {
                    eprintln!("[DEBUG] Document パース失敗: {}\n  item: {}", e, item);
                }
            }
        }
    }

    Ok(docs)
}

fn download_and_extract(
    api_key: &str,
    doc_id: &str,
    edinet_code: &str,
    output_dir: &PathBuf,
) -> Result<PathBuf> {
    let url = format!("{}/documents/{}", BASE_URL, doc_id);
    let client = reqwest::blocking::Client::new();
    let bytes = client
        .get(&url)
        .query(&[("type", "1"), ("Subscription-Key", api_key)])
        .timeout(std::time::Duration::from_secs(120))
        .send()?
        .error_for_status()?
        .bytes()?;

    let out_dir = output_dir.join(edinet_code).join(doc_id);
    fs::create_dir_all(&out_dir)?;

    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;
    archive.extract(&out_dir)?;

    Ok(out_dir)
}

fn find_files_by_ext(dir: &PathBuf, ext: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(find_files_by_ext(&path, ext));
            } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                result.push(path);
            }
        }
    }
    result
}
