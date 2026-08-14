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
                    let edinet_code = doc.edinet_code.as_deref().unwrap_or("unknown");
                    let out_dir = output_dir.join(edinet_code).join(&doc.doc_id);

                    if already_fetched(&out_dir) {
                        println!(
                            "  - {} ({}) 取得済みのためスキップ",
                            doc.filer_name.as_deref().unwrap_or("不明"),
                            edinet_code,
                        );
                        continue;
                    }

                    found += 1;
                    println!(
                        "  → {} ({}) {}",
                        doc.filer_name.as_deref().unwrap_or("不明"),
                        edinet_code,
                        doc.doc_description.as_deref().unwrap_or(""),
                    );
                    if let (Some(ps), Some(pe)) = (&doc.period_start, &doc.period_end) {
                        println!("    期間: {} 〜 {}", ps, pe);
                    }

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

                    // EDINETへの配慮。日付ループ側のスリープとは別に、同日内で複数件
                    // ダウンロードする場合にも間隔を空ける
                    thread::sleep(time::Duration::from_millis(500));
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

    // PublicDoc配下の本編XBRLインスタンス1つだけを取り出す。AuditDoc・表示用HTML・
    // 画像・タクソノミ拡張定義は今のパイプラインでは使わないため保存しない
    // （2026/08/14/006.md、1書類あたり44ファイル→1ファイルに削減）。
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();
        let Some(file_name) = entry
            .enclosed_name()
            .and_then(|p| p.file_name().map(|f| f.to_os_string()))
        else {
            continue;
        };
        let file_name = file_name.to_string_lossy().to_string();

        if is_main_instance_document(&entry_name, &file_name) {
            let dest = out_dir.join(&file_name);
            let mut out_file = fs::File::create(&dest)?;
            std::io::copy(&mut entry, &mut out_file)?;
            return Ok(out_dir);
        }
    }

    anyhow::bail!("本編XBRLインスタンスが見つかりません: doc_id={}", doc_id)
}

/// PublicDoc配下の本編XBRLインスタンスドキュメントかどうかを判定する。
/// 例: XBRL/PublicDoc/jpcrp030000-asr-001_E00540-000_2026-03-31_01_2026-06-09.xbrl
/// AuditDoc側のファイルは "jpaud-" 接頭辞のため、これだけで判別できる。
fn is_main_instance_document(entry_name: &str, file_name: &str) -> bool {
    entry_name.contains("PublicDoc") && file_name.starts_with("jpcrp") && file_name.ends_with(".xbrl")
}

/// 書類の出力先ディレクトリに、展開済みのファイルが既に存在するかどうかを見て判定する。
/// 冪等性の担保: 同じ期間・条件で再実行しても、未取得分だけがダウンロードされる。
fn already_fetched(out_dir: &PathBuf) -> bool {
    fs::read_dir(out_dir)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
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
