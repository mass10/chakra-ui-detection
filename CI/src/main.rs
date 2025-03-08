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

fn fix_windows_path(path: &str) -> String {
    return path.replace("\\", "/");
}

/// ファイル診断
#[allow(unused)]
fn diagnose_files(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut names: Vec<String> = vec![];
    let directory = std::fs::read_dir(path)?;
    for entry in directory {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        // パス文字列
        let abs_path = path.to_string_lossy().to_string();
        names.push(path.to_string_lossy().to_string());

        // // ファイル情報
        // let path_info = std::fs::metadata(&path)?;
        // // ファイルサイズ
        // let length = path_info.len();
        // // MD5
        // let md5sum = dump_file_md5(&abs_path)?;
        // println!("{}, {}, {}", fix_windows_path(&abs_path), length, md5sum);
    }

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
fn execute() -> Result<(), Box<dyn std::error::Error>> {
    chdir("..")?;

    // 開始位置のパス
    let paths = concat_path_parts(&["src", "components", "chakra"]);

    // 診断
    let _files = diagnose_files(&paths)?;

    return Ok(());
}

fn chdir(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_current_dir(path)?;
    return Ok(());
}

fn main() {
    let result = execute();
    if result.is_err() {
        println!("Error: {:?}", result.err());
        std::process::exit(1);
    }

    // let result = execute_command(&["find", "src/components/chakra", "-type", "f"]);
}
