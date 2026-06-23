//! EDINET API v2 から有価証券報告書（XBRL）および EDINETコードリストを取得する CLI ツール。
//!
//! サブコマンド:
//!
//!   # EDINETコードリストを取得して CSV に保存
//!   cargo run -p fetch_xbrl -- codelist
//!   cargo run -p fetch_xbrl -- codelist --output D:/data/codelist
//!
//!   # XBRL（有価証券報告書）を取得
//!   cargo run -p fetch_xbrl -- fetch --from 2025-06-01 --to 2025-06-30
//!   cargo run -p fetch_xbrl -- fetch --from 2025-06-01 --to 2025-06-30 --sec-code 285A
//!   cargo run -p fetch_xbrl -- fetch --from 2025-06-01 --to 2025-06-30 --edinet-code E37543
//!   cargo run -p fetch_xbrl -- fetch --from 2025-06-01 --to 2025-06-30 --name キオクシア

use ::zip::ZipArchive;
use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate};
use clap::{Parser, Subcommand};
use polars::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::{fs, io::Cursor, path::PathBuf, thread, time};

const BASE_URL: &str = "https://api.edinet-fsa.go.jp/api/v2";
const CODELIST_URL: &str =
    "https://disclosure2dl.edinet-fsa.go.jp/searchdocument/codelist/Edinetcode.zip";

// ── CLI 定義 ─────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "fetch_xbrl", about = "EDINET API v2 ツール")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// EDINETコードリスト（全上場企業一覧）を取得して CSV に保存
    Codelist {
        /// 出力先ディレクトリ（デフォルト: data/codelist）
        #[arg(long, value_name = "DIR", default_value = "data/codelist")]
        output: PathBuf,
    },
    /// 東証上場銘柄一覧とEDINETコードリストを結合してParquetに保存
    Master {
        /// 東証上場銘柄一覧CSV（data_j.csv）
        #[arg(long, value_name = "FILE")]
        jpx: PathBuf,
        /// EDINETコードリストCSV（EdinetcodeDlInfo.csv）
        #[arg(long, value_name = "FILE")]
        codelist: PathBuf,
        /// 出力Parquetファイルパス
        #[arg(long, value_name = "FILE", default_value = "data/master.parquet")]
        output: PathBuf,
    },
    /// 有価証券報告書（XBRL）を取得
    Fetch {
        /// 検索開始日 YYYY-MM-DD
        #[arg(long, value_name = "DATE")]
        from: NaiveDate,
        /// 検索終了日 YYYY-MM-DD
        #[arg(long, value_name = "DATE")]
        to: NaiveDate,
        /// EDINETコードで絞り込み（複数可）例: E37543
        #[arg(long = "edinet-code", value_name = "CODE")]
        edinet_codes: Vec<String>,
        /// 証券コードで絞り込み（複数可）例: 285A
        #[arg(long = "sec-code", value_name = "CODE")]
        sec_codes: Vec<String>,
        /// 会社名（部分一致）で絞り込み（複数可）
        #[arg(long = "name", value_name = "NAME")]
        names: Vec<String>,
        /// 出力先ディレクトリ（デフォルト: data/xbrl）
        #[arg(long, value_name = "DIR", default_value = "data/xbrl")]
        output: PathBuf,
        /// デバッグ: APIレスポンスの生JSONを出力
        #[arg(long)]
        debug: bool,
    },
}

// ── EDINET API レスポンス型 ───────────────────────────────

#[allow(dead_code)]
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

// ── フィルター ────────────────────────────────────────────

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

// ── メイン ───────────────────────────────────────────────

fn main() -> Result<()> {
    // Windows のコンソール出力を UTF-8 に設定
    #[cfg(target_os = "windows")]
    unsafe {
        windows_sys::Win32::System::Console::SetConsoleOutputCP(65001);
    }

    for base in &["scripts/fetch_xbrl", ".", "..", "../.."] {
        let path = std::path::Path::new(base).join(".env");
        if path.exists() {
            dotenv::from_path(&path).ok();
            break;
        }
    }

    let cli = Cli::parse();

    match cli.command {
        Command::Master {
            jpx,
            codelist,
            output,
        } => run_master(&jpx, &codelist, &output),
        Command::Codelist { output } => run_codelist(&output),
        Command::Fetch {
            from,
            to,
            edinet_codes,
            sec_codes,
            names,
            output,
            debug,
        } => {
            let api_key = std::env::var("EDINET_API_KEY")
                .context(".env に EDINET_API_KEY が設定されていません")?;
            let filter = Filter {
                edinet_codes,
                sec_codes,
                names,
            };
            run_fetch(&api_key, &filter, from, to, &output, debug)
        }
    }
}

// ── master サブコマンド ───────────────────────────────────

fn run_master(jpx_path: &PathBuf, codelist_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    println!("JPX銘柄一覧を読み込み中: {}", jpx_path.display());

    // JPX CSV 読み込み
    // 列: 日付,コード,銘柄名,市場・商品区分,33業種コード,33業種区分,17業種コード,17業種区分,規模コード,規模区分
    let jpx = CsvReadOptions::default()
        .with_has_header(true)
        .with_infer_schema_length(Some(10))
        .try_into_reader_with_file_path(Some(jpx_path.clone()))?
        .finish()
        .map_err(|e| anyhow::anyhow!("JPX CSV読み込み失敗: {}", e))?;

    println!("  → {} 件", jpx.height());

    // コード列を文字列に統一（4桁ゼロ埋め）
    let jpx = jpx
        .lazy()
        .with_column(col("コード").cast(DataType::String).alias("証券コード"))
        .select([
            col("証券コード"),
            col("銘柄名"),
            col("市場・商品区分").alias("市場区分"),
            col("33業種区分"),
            col("17業種区分"),
            col("規模区分"),
        ])
        .collect()
        .map_err(|e| anyhow::anyhow!("JPX変換失敗: {}", e))?;

    println!(
        "EDINETコードリストを読み込み中: {}",
        codelist_path.display()
    );

    // EDINETコードリスト読み込み（1行目はヘッダー情報なのでスキップ）
    // 列: EDINETコード,提出者種別,上場区分,連結の有無,資本金,決算日,提出者名,...,証券コード,提出者法人番号
    // 証券コードは "409A0" のような英数字混在があるため全列Stringで読んでからcast
    let codelist = CsvReadOptions::default()
        .with_has_header(true)
        .with_skip_rows(1)
        .with_infer_schema_length(Some(0))
        .try_into_reader_with_file_path(Some(codelist_path.clone()))?
        .finish()
        .map_err(|e| anyhow::anyhow!("EDINETコードリスト読み込み失敗: {}", e))?;

    println!("  → {} 件", codelist.height());

    let codelist = codelist
        .lazy()
        .select([
            col("ＥＤＩＮＥＴコード").alias("EDINETコード"),
            col("決算日"),
            col("提出者法人番号").alias("法人番号"),
            col("証券コード"),
        ])
        .collect()
        .map_err(|e| anyhow::anyhow!("EDINET変換失敗: {}", e))?;

    // 結合: 証券コードをキーにinner join
    println!("結合中...");
    let master = jpx
        .lazy()
        .join(
            codelist.lazy(),
            [col("証券コード")],
            [col("証券コード")],
            JoinArgs::new(JoinType::Left),
        )
        .collect()
        .map_err(|e| anyhow::anyhow!("結合失敗: {}", e))?;

    println!("  → {} 件", master.height());

    // Parquet出力
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(output_path)?;
    ParquetWriter::new(&mut file)
        .finish(&mut master.clone())
        .map_err(|e| anyhow::anyhow!("Parquet書き込み失敗: {}", e))?;

    println!("✓ 保存完了: {}", output_path.display());
    println!("  列: {:?}", master.get_column_names());
    println!("  行数: {}", master.height());

    Ok(())
}

// ── codelist サブコマンド ─────────────────────────────────

fn run_codelist(output_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    println!("EDINETコードリストをダウンロード中...");
    println!("URL: {}", CODELIST_URL);

    let client = reqwest::blocking::Client::new();
    let bytes = client
        .get(CODELIST_URL)
        .timeout(std::time::Duration::from_secs(60))
        .send()?
        .error_for_status()?
        .bytes()?;

    println!("ダウンロード完了 ({} bytes) ZIPを展開中...", bytes.len());

    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;
    archive.extract(output_dir)?;

    // Shift-JIS → UTF-8 変換
    println!("Shift-JIS → UTF-8 変換中...");
    for entry in fs::read_dir(output_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("csv") {
            let raw = fs::read(&path)?;
            let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&raw);
            fs::write(&path, decoded.as_bytes())?;
            println!(
                "  ✓ {} (UTF-8変換済)",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
        }
    }

    println!("✓ 出力先: {}", output_dir.canonicalize()?.display());
    Ok(())
}

// ── fetch サブコマンド ────────────────────────────────────

fn run_fetch(
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

// ── API 呼び出し ─────────────────────────────────────────

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
