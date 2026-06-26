use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fetch_xbrl", about = "EDINET API v2 ツール")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// EDINETコードリスト（全上場企業一覧）を取得して CSV に保存
    Codelist {
        #[arg(long, value_name = "DIR", default_value = "data/codelist")]
        output: PathBuf,
    },
    /// 東証上場銘柄一覧から個別株を抽出して Parquet に保存
    Master {
        #[arg(long, value_name = "FILE")]
        jpx: PathBuf,
        #[arg(long, value_name = "FILE")]
        codelist: PathBuf,
        #[arg(long, value_name = "FILE", default_value = "data/master.parquet")]
        output: PathBuf,
    },
    /// Parquet ファイルのスキーマと内容を表示
    Schema {
        #[arg(long, value_name = "FILE", default_value = "data/master.parquet")]
        input: PathBuf,
        /// データを表示する
        #[arg(long)]
        show_data: bool,
        /// 表示する列名（複数可、省略時は全列）
        #[arg(long = "col", value_name = "COL")]
        columns: Vec<String>,
        /// 表示行数（デフォルト: 10）
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// 有価証券報告書（XBRL）を取得
    Fetch {
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
        #[arg(long, value_name = "DIR", default_value = "data/xbrl")]
        output: PathBuf,
        #[arg(long)]
        debug: bool,
    },
}
