use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate};
use serde_json::Value;
use std::{fs, io::Cursor, path::PathBuf, thread, time};
use zip::ZipArchive;

use crate::types::{Document, Filter};

const BASE_URL: &str = "https://api.edinet-fsa.go.jp/api/v2";

pub fn run(
    api_key: &str,
    filter: &Filter,
    from: NaiveDate,
    to: NaiveDate,
    output_dir: &PathBuf,
    debug: bool,
) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    println!("出力先: {}", output_dir.canonicalize()?.display());
    println!(
        "期間: {} 〜 {} ({} 日間)",
        from,
        to,
        (to - from).num_days() + 1
    );
    if filter.is_empty() {
        println!("絞り込み: なし（全社）");
    }
    println!("{}", "─".repeat(60));

    let mut current = from;
    let mut found = 0;

    while current <= to {
        let date_str = current.format("%Y-%m-%d").to_string();

        match get_documents(api_key, &date_str, debug) {
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
                    match download_and_extract(api_key, &doc.doc_id, edinet_code, output_dir) {
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

    let root: Value =
        serde_json::from_str(&text).with_context(|| format!("JSONパース失敗 ({})", date))?;

    let results = match root.get("results") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => return Ok(vec![]),
    };

    let mut docs = Vec::new();
    for item in &results {
        match serde_json::from_value::<Document>(item.clone()) {
            Ok(doc) => docs.push(doc),
            Err(e) => {
                if debug {
                    eprintln!("[DEBUG] パース失敗: {}\n  item: {}", e, item);
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
    let mut archive = ZipArchive::new(cursor)?;
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
