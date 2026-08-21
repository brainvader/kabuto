//! EDINET API v2 から有価証券報告書（XBRL）および EDINETコードリストを取得する CLI ツール。
//!
//! サブコマンド:
//!   codelist  EDINETコードリストを取得して CSV に保存
//!   master    東証上場銘柄一覧から個別株を抽出して Parquet に保存
//!   schema    Parquet ファイルのスキーマと内容を表示
//!   fetch     XBRL（有価証券報告書）を取得（取得済みの書類はスキップする）
//!   parse     取得済みXBRLから financial_metric/disclosure_text を抽出する（APIなし）
//!   ingest    parseの出力をSurrealDBへ投入する（disclosure_textはOpenAIでembedding）
//!   sync      codelist → master → fetch を一括実行する（繰り返し実行しても安全）

mod cli;
mod cmd;
mod types;

use anyhow::Result;
use clap::Parser;
use cli::Command;

fn main() -> Result<()> {
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

    let cli = cli::Cli::parse();

    match cli.command {
        Command::Schema {
            input,
            show_data,
            columns,
            limit,
        } => cmd::schema::run(&input, show_data, &columns, limit),
        Command::Master {
            jpx,
            codelist,
            output,
        } => cmd::master::run(&jpx, &codelist, &output),
        Command::Codelist { output } => cmd::codelist::run(&output),
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
                .map_err(|_| anyhow::anyhow!(".env に EDINET_API_KEY が設定されていません"))?;
            let filter = types::Filter {
                edinet_codes,
                sec_codes,
                names,
            };
            cmd::fetch::run(&api_key, &filter, from, to, &output, debug)
        }
        Command::Parse { input, output, master } => cmd::parse::run(&input, &output, &master),
        Command::InitDb { db } => cmd::init_db::run(&db),
        Command::Ingest { input, db } => cmd::ingest::run(&input, &db),
        Command::Companies { master, db } => cmd::companies::run(&master, &db),
        Command::Repack { src, dst, limit } => cmd::repack::run(&src, &dst, limit),
        Command::Sync {
            codelist_dir,
            jpx,
            master,
            output,
            from,
            to,
            debug,
        } => cmd::sync::run(&codelist_dir, &jpx, &master, &output, from, to, debug),
    }
}
