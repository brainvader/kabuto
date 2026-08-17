//! XBRLの本編インスタンスドキュメントから、financial_metric/disclosure_text相当の
//! データを抽出し、中間ファイル（JSON Lines）に書き出す。
//! API呼び出しは行わないため、何度でも気軽に再実行できる（2026/08/15/007.md Step 2）。
//!
//! 抽出対象（2026/08/14/005.md）:
//! - `*SummaryOfBusinessResults`（5期比較サマリー）
//! - `*TextBlock`（事業等のリスク等の自然言語セクション）

use anyhow::{Context, Result};
use polars::prelude::*;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;
use regex::Regex;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// embedding対象とするナラティブなセクション（`*TextBlock`の接尾辞を除いた名前）。
/// `*TextBlock`は数百種類あり、役員経歴のような繰り返し項目や、貸借対照表・注記
/// のような表形式の財務データも含まれる（HTMLタグを剥がすと数字とラベルが混ざった
/// 読みにくいテキストになる）ため、実測して意味のある記述だけに絞り込んだ
/// （2026/08/15/007.md、実データでの検証）。
const NARRATIVE_SECTIONS: &[&str] = &[
    "BusinessRisks",
    "ManagementAnalysisOfFinancialPositionOperatingResultsAndCashFlows",
    "DescriptionOfBusiness",
    "BusinessResultsOfGroup",
    "BusinessResultsOfReportingCompany",
    "BusinessPolicyBusinessEnvironmentIssuesToAddressEtc",
    "Strategy",
    "RiskManagement",
    "DividendPolicy",
    "PolicyOnDevelopmentOfHumanResourcesAndInternalEnvironmentStrategy",
    "BasicPolicyOnHumanResourcesStrategyEmployeesEtc",
    "CompanyHistory",
    "CriticalContracts",
    "DisclosureOfSustainabilityRelatedFinancialInformation",
    "MetricsAndTargets",
];

/// 5期比較サマリーのcontextRefとして扱う値。これ以外（連結/個別区分やセグメント別
/// の次元付きcontextRef）は対象外にする。
const VALID_CONTEXTS: &[(&str, i32)] = &[
    ("CurrentYearDuration", 0),
    ("CurrentYearInstant", 0),
    ("Prior1YearDuration", 1),
    ("Prior1YearInstant", 1),
    ("Prior2YearDuration", 2),
    ("Prior2YearInstant", 2),
    ("Prior3YearDuration", 3),
    ("Prior3YearInstant", 3),
    ("Prior4YearDuration", 4),
    ("Prior4YearInstant", 4),
];

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
struct DisclosureTextRow {
    doc_id: String,
    company_code: Option<String>,
    section: String,
    fiscal_year: i32,
    text: String,
}

pub fn run(input_dir: &PathBuf, output_dir: &PathBuf, master_path: &PathBuf) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    let metrics_path = output_dir.join("financial_metrics.jsonl");
    let texts_path = output_dir.join("disclosure_texts.jsonl");

    let mut metrics_file = fs::File::create(&metrics_path)?;
    let mut texts_file = fs::File::create(&texts_path)?;

    let edinet_to_code = read_edinet_to_code_map(master_path)?;
    println!(
        "EDINETコード→証券コード対応: {} 件（{}）",
        edinet_to_code.len(),
        master_path.display()
    );

    let mut doc_count = 0usize;
    let mut metric_count = 0usize;
    let mut text_count = 0usize;
    let mut unmapped_edinet_codes: std::collections::HashSet<String> = std::collections::HashSet::new();

    for edinet_entry in fs::read_dir(input_dir)
        .with_context(|| format!("入力ディレクトリを開けません: {}", input_dir.display()))?
    {
        let edinet_dir = edinet_entry?.path();
        if !edinet_dir.is_dir() {
            continue;
        }
        let edinet_code = edinet_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let company_code = edinet_to_code.get(&edinet_code).cloned();
        if company_code.is_none() {
            unmapped_edinet_codes.insert(edinet_code.clone());
        }

        for doc_entry in fs::read_dir(&edinet_dir)? {
            let doc_dir = doc_entry?.path();
            if !doc_dir.is_dir() {
                continue;
            }
            let doc_id = doc_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();

            let Some(xbrl_file) = find_xbrl_file(&doc_dir)? else {
                continue;
            };

            let fiscal_year = match extract_fiscal_year(&xbrl_file) {
                Ok(y) => y,
                Err(e) => {
                    eprintln!("  × 決算年の抽出失敗 ({}): {}", doc_id, e);
                    continue;
                }
            };

            let (metrics, texts) =
                match parse_xbrl(&xbrl_file, &doc_id, company_code.as_deref(), fiscal_year) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("  × XBRL解析失敗 ({}): {}", doc_id, e);
                        continue;
                    }
                };

            for m in &metrics {
                writeln!(metrics_file, "{}", serde_json::to_string(m)?)?;
            }
            for t in &texts {
                writeln!(texts_file, "{}", serde_json::to_string(t)?)?;
            }

            println!(
                "  ✓ {} (FY{}): financial_metric {} 件 / disclosure_text {} 件",
                doc_id,
                fiscal_year,
                metrics.len(),
                texts.len()
            );

            doc_count += 1;
            metric_count += metrics.len();
            text_count += texts.len();
        }
    }

    println!("{}", "─".repeat(60));
    println!(
        "完了: {} 書類 / financial_metric {} 件 / disclosure_text {} 件",
        doc_count, metric_count, text_count
    );
    if !unmapped_edinet_codes.is_empty() {
        println!(
            "  ※ master.parquetに対応が無いEDINETコード: {} 件（company_codeはNULLのまま）",
            unmapped_edinet_codes.len()
        );
    }
    println!("出力: {}", metrics_path.display());
    println!("出力: {}", texts_path.display());

    Ok(())
}

/// master.parquetから EDINETコード→証券コード の対応表を作る。
/// `financial_metric`/`disclosure_text`はdoc_idしか持っておらず銘柄コードを
/// 直接持っていなかった（2026/08/16/002.md）ため、parse時点で解決して埋め込む。
fn read_edinet_to_code_map(master_path: &PathBuf) -> Result<HashMap<String, String>> {
    let file = fs::File::open(master_path)
        .with_context(|| format!("master.parquetを開けません: {}", master_path.display()))?;
    let df = ParquetReader::new(file)
        .finish()
        .map_err(|e| anyhow::anyhow!("master.parquet読み込み失敗: {}", e))?;

    let edinet_col = df
        .column("EDINETコード")
        .map_err(|e| anyhow::anyhow!("EDINETコード列が見つかりません: {}", e))?
        .str()
        .map_err(|e| anyhow::anyhow!("EDINETコード列の型が不正です: {}", e))?;
    let code_col = df
        .column("証券コード")
        .map_err(|e| anyhow::anyhow!("証券コード列が見つかりません: {}", e))?
        .str()
        .map_err(|e| anyhow::anyhow!("証券コード列の型が不正です: {}", e))?;

    Ok(edinet_col
        .into_iter()
        .zip(code_col.into_iter())
        .filter_map(|(e, c)| match (e, c) {
            (Some(e), Some(c)) if !e.is_empty() && !c.is_empty() => {
                Some((e.to_string(), c.to_string()))
            }
            _ => None,
        })
        .collect())
}

fn find_xbrl_file(doc_dir: &Path) -> Result<Option<PathBuf>> {
    for entry in fs::read_dir(doc_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("xbrl") {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

/// ファイル名（例: jpcrp030000-asr-001_E00037-000_2026-03-31_01_2026-06-17.xbrl）の
/// 3番目のセグメントが決算期末日なので、その年を決算年として使う。
fn extract_fiscal_year(xbrl_file: &Path) -> Result<i32> {
    let file_name = xbrl_file
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow::anyhow!("ファイル名が不正です"))?;
    let period_end = file_name
        .split('_')
        .nth(2)
        .ok_or_else(|| anyhow::anyhow!("ファイル名から決算期末日を抽出できません: {}", file_name))?;
    period_end
        .get(0..4)
        .ok_or_else(|| anyhow::anyhow!("決算期末日の形式が不正です: {}", period_end))?
        .parse::<i32>()
        .with_context(|| format!("決算年のパース失敗: {}", period_end))
}

fn parse_xbrl(
    path: &Path,
    doc_id: &str,
    company_code: Option<&str>,
    fiscal_year: i32,
) -> Result<(Vec<FinancialMetricRow>, Vec<DisclosureTextRow>)> {
    let content = fs::read(path)?;
    let mut reader = Reader::from_reader(content.as_slice());

    let mut metrics = Vec::new();
    let mut texts = Vec::new();
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let full_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let local = local_name(&full_name);

                if let Some(local) = local.strip_suffix("SummaryOfBusinessResults") {
                    let context_ref = attr_value(&e, b"contextRef");
                    let unit_ref = attr_value(&e, b"unitRef").unwrap_or_default();
                    let name_bytes = e.name().as_ref().to_vec();
                    let text = read_element_text(&mut reader, QName(&name_bytes))?;

                    if let (Some(ctx), Ok(value)) = (context_ref, text.trim().parse::<f64>()) {
                        if let Some((_, offset)) = VALID_CONTEXTS.iter().find(|(c, _)| *c == ctx) {
                            metrics.push(FinancialMetricRow {
                                doc_id: doc_id.to_string(),
                                company_code: company_code.map(|c| c.to_string()),
                                metric: local.to_string(),
                                xbrl_tag: format!("{}SummaryOfBusinessResults", local),
                                value,
                                unit: unit_ref,
                                fiscal_year: fiscal_year - offset,
                                consolidated: true,
                            });
                        }
                    }
                    continue;
                }

                if let Some(local) = local.strip_suffix("TextBlock") {
                    if !NARRATIVE_SECTIONS.contains(&local) {
                        continue;
                    }
                    let section = local.to_string();
                    let name_bytes = e.name().as_ref().to_vec();
                    let raw = read_element_text(&mut reader, QName(&name_bytes))?;
                    let clean = clean_text_block(&raw);
                    if !clean.trim().is_empty() {
                        texts.push(DisclosureTextRow {
                            doc_id: doc_id.to_string(),
                            company_code: company_code.map(|c| c.to_string()),
                            section,
                            fiscal_year,
                            text: clean,
                        });
                    }
                    continue;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok((metrics, texts))
}

/// `prefix:LocalName` 形式のタグ名からローカル名だけを取り出す。
fn local_name(qualified: &str) -> &str {
    qualified.split(':').next_back().unwrap_or(qualified)
}

fn attr_value(e: &BytesStart, key: &[u8]) -> Option<String> {
    e.attributes().flatten().find_map(|a| {
        if a.key.as_ref() == key {
            Some(String::from_utf8_lossy(&a.value).to_string())
        } else {
            None
        }
    })
}

/// 開始タグを読んだ直後から、対応する終了タグまでのテキストを読み進めて連結する。
/// XBRLのTextBlockは中に実XML要素を持たず、エスケープされたHTML文字列がテキストと
/// して入っているだけなので、Text/CDATAイベントを拾うだけで完結する。
fn read_element_text(reader: &mut Reader<&[u8]>, start_name: QName) -> Result<String> {
    let mut text = String::new();
    let mut local_buf = Vec::new();
    loop {
        local_buf.clear();
        match reader.read_event_into(&mut local_buf)? {
            Event::Text(e) => text.push_str(&e.unescape()?),
            Event::CData(e) => text.push_str(&String::from_utf8_lossy(&e.into_inner())),
            Event::End(e) if e.name() == start_name => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(text)
}

/// TextBlockの中身（エスケープ解除済みのHTML文字列）からタグを除去し、
/// embeddingに使うプレーンテキストにする。
fn clean_text_block(html: &str) -> String {
    let tag_re = Regex::new(r"(?s)<[^>]*>").unwrap();
    let without_tags = tag_re.replace_all(html, " ");
    without_tags
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
