//!
//! バッチ処理
//!

/// コマンドを実行
#[allow(unused)]
fn execute_command(command: &[&str]) -> std::process::Output {
    std::process::Command::new(command[0])
        .args(&command[1..])
        .output()
        .expect("failed to execute process")
}

/// ファイルの MD5 をダンプ
fn dump_file_md5(path: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    use std::io::Read;

    // ファイルを開きます。
    let mut file = std::fs::File::open(path)?;

    // 読み取り用バッファ
    let mut buffer = std::vec::Vec::new();

    // ファイル全体を読み込み
    file.read_to_end(&mut buffer)?;

    // MD5 を計算
    let response = md5::compute(buffer);

    return Ok(format!("{:x}", response));
}

/// パスの補正(出力用)
fn fix_windows_path(path: &str) -> String {
    return path.replace("\\", "/");
}

/// ファイル診断
#[allow(unused)]
fn diagnose_files(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut names: Vec<String> = vec![];

    // ディレクトリ内のファイルを列挙
    let directory = std::fs::read_dir(path)?;
    for entry in directory {
        let entry = entry?;
        let path = entry.path();
        let abs_path = path.to_string_lossy().to_string();
        names.push(path.to_string_lossy().to_string());
    }

    // 破壊的ソート
    names.sort();

    for path in &names {
        let path_info = std::fs::metadata(&path)?;
        // ファイル情報
        let path_info = std::fs::metadata(&path)?;
        // ファイルサイズ
        let length = path_info.len();
        // MD5
        let md5sum = dump_file_md5(&path)?;
        println!("{}, {}, {}", fix_windows_path(&path), length, md5sum);
    }

    return Ok(());
}

/// OS を考慮したパス結合
fn concat_path_parts(parts: &[&str]) -> String {
    return parts.join(&std::path::MAIN_SEPARATOR.to_string());
}

/// 実行
fn execute(args: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    chdir("..")?;

    // 開始位置のパス
    let paths = concat_path_parts(&["src", "components", "chakra"]);

    // 診断
    let _files = diagnose_files(&paths)?;

    eprintln!("[INFO] チェックサムを出力しました。");
    eprintln!("[INFO] Ok.");

    return Ok(());
}

/// ディレクトリを変更
fn chdir(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_current_dir(path)?;
    return Ok(());
}

/// Rust アプリケーションのエントリーポイント
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let result = execute(&args);
    if result.is_err() {
        println!("Error: {:?}", result.err());
        std::process::exit(1);
    }

    // let result = execute_command(&["find", "src/components/chakra", "-type", "f"]);
}
