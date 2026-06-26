use anyhow::Result;
use polars::prelude::*;
use std::{fs, path::PathBuf};
use unicode_normalization::UnicodeNormalization;

/// 東証上場銘柄一覧（data_j.csv）と EDINETコードリストを結合して Parquet に保存する。
///
/// # フィルタリング
/// 市場・商品区分が「内国株式」を含むもののみ（ETF・ETN・REIT 等を除外）
///
/// # 出力スキーマ
/// - 証券コード  String
/// - 銘柄名      String
/// - 市場区分    String  "プライム" / "スタンダード" / "グロース"
/// - 33業種区分  String
/// - EDINETコード String
/// - 決算日      String
/// - 法人番号    String
pub fn run(jpx_path: &PathBuf, codelist_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    println!("JPX銘柄一覧を読み込み中: {}", jpx_path.display());

    let jpx = CsvReadOptions::default()
        .with_has_header(true)
        .with_infer_schema_length(Some(0))
        .try_into_reader_with_file_path(Some(jpx_path.clone()))?
        .finish()
        .map_err(|e| anyhow::anyhow!("data_j.csv 読み込み失敗: {}", e))?;

    println!("  → {} 件（フィルタ前）", jpx.height());

    let jpx = jpx
        .lazy()
        .filter(col("市場・商品区分").str().contains(lit("内国株式"), true))
        .with_column(
            col("市場・商品区分")
                .str()
                .replace_all(lit("（内国株式）"), lit(""), true)
                .alias("市場区分"),
        )
        .with_column(col("コード").cast(DataType::String).alias("証券コード"))
        .select([
            col("証券コード"),
            col("銘柄名")
                .map(
                    |s| {
                        let ca = s.str()?;
                        let normalized: StringChunked =
                            ca.apply(|opt| opt.map(|s| s.nfkc().collect::<String>().into()));
                        Ok(Some(normalized.into_column()))
                    },
                    GetOutput::same_type(),
                )
                .alias("銘柄名"),
            col("市場区分"),
            col("33業種区分"),
        ])
        .collect()
        .map_err(|e| anyhow::anyhow!("JPX変換失敗: {}", e))?;

    println!("  → {} 件（個別株のみ）", jpx.height());

    println!(
        "EDINETコードリストを読み込み中: {}",
        codelist_path.display()
    );

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
            col("証券コード")
                .str()
                .slice(lit(0), lit(4)) // 先頭4文字を取得
                .alias("証券コード"),
        ])
        .collect()
        .map_err(|e| anyhow::anyhow!("EDINET変換失敗: {}", e))?;

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
