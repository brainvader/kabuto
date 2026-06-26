use anyhow::Result;
use polars::prelude::*;
use std::path::PathBuf;

pub fn run(input: &PathBuf) -> Result<()> {
    let file = std::fs::File::open(input)
        .map_err(|_| anyhow::anyhow!("ファイルを開けません: {}", input.display()))?;
    let df = ParquetReader::new(file)
        .finish()
        .map_err(|e| anyhow::anyhow!("Parquet読み込み失敗: {}", e))?;

    println!("ファイル: {}", input.display());
    println!("行数: {}", df.height());
    println!("{}", "─".repeat(40));
    println!("{:<20} {}", "列名", "型");
    println!("{}", "─".repeat(40));
    for field in df.schema().iter_fields() {
        println!("{:<20} {:?}", field.name(), field.dtype());
    }
    println!("{}", "─".repeat(40));
    Ok(())
}
