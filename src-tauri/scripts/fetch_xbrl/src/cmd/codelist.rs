use anyhow::Result;
use encoding_rs::SHIFT_JIS;
use std::{fs, io::Cursor, path::PathBuf};
use zip::ZipArchive;

const CODELIST_URL: &str =
    "https://disclosure2dl.edinet-fsa.go.jp/searchdocument/codelist/Edinetcode.zip";

pub fn run(output_dir: &PathBuf) -> Result<()> {
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

    println!("Shift-JIS → UTF-8 変換中...");
    for entry in fs::read_dir(output_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("csv") {
            let raw = fs::read(&path)?;
            let (decoded, _, _) = SHIFT_JIS.decode(&raw);
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
