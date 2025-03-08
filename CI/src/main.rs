//!
//! バッチ処理
//!

/// コマンドを実行
fn execute_command(command: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
	let result = std::process::Command::new(command[0])
		.args(&command[1..])
		.stderr(std::process::Stdio::inherit())
		.stdout(std::process::Stdio::inherit())
		.output()?;
	if !result.status.success() {
		let error = format!("Command failed: {}", command.join(" "));
		return Err(error.into());
	}
	return Ok(());
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
fn fix_path_string(path: &str) -> String {
	let path = path.replace("\\", "/");
	let path = path.trim_start_matches("../").to_string();
	return path;
}

/// ファイル診断
fn diagnose_files(path: &str, outpath: &str) -> Result<(), Box<dyn std::error::Error>> {
	use std::io::Write;

	let mut names: Vec<String> = vec![];

	// ディレクトリ内のファイルを列挙
	let directory = std::fs::read_dir(path)?;
	for entry in directory {
		let entry = entry?;
		let path = entry.path();
		// let abs_path = path.to_string_lossy().to_string();
		names.push(path.to_string_lossy().to_string());
	}

	let file = std::fs::File::create(outpath)?;
	let mut writer = std::io::BufWriter::new(&file);

	// 破壊的ソート
	names.sort();

	for path in &names {
		// ファイル情報
		let path_info = std::fs::metadata(&path)?;
		// ファイルサイズ
		let length = path_info.len();
		// MD5
		let md5sum = dump_file_md5(&path)?;

		// println!("{}, {}, {}", fix_path_string(&path), length, md5sum);

		writer.write_all(
			format!("{}, {}, {}\n", fix_path_string(&path), length, md5sum).as_bytes(),
		)?;
	}

	return Ok(());
}

/// ディレクトリを変更
#[allow(unused)]
fn chdir(path: &str) -> Result<(), Box<dyn std::error::Error>> {
	std::env::set_current_dir(path)?;
	return Ok(());
}

/// OS を考慮したパス結合
fn concat_path_parts(parts: &[&str]) -> String {
	return parts.join(&std::path::MAIN_SEPARATOR.to_string());
}

/// Chakra UI のコンポーネントファイルを削除
fn remove_components(path: &str) -> Result<(), Box<dyn std::error::Error>> {
	let reader = std::fs::read_dir(path)?;
	for entry in reader {
		let entry = entry?;
		let path = entry.path();
		let name = path
			.file_name()
			.unwrap_or_default()
			.to_str()
			.unwrap_or_default();
		// TSX ファイルのみ削除
		if name.ends_with(".tsx") {
			std::fs::remove_file(&path)?;
		}
	}
	return Ok(());
}

/// チェックサムファイルを生成
fn generate_checksum_file() -> Result<(), Box<dyn std::error::Error>> {
	// 開始位置のパス
	let chakra_components = concat_path_parts(&["..", "src", "components", "chakra"]);

	// 診断
	let _ = diagnose_files(&chakra_components, "chakra_checksum.txt")?;

	eprintln!("[INFO] チェックサムを出力しました。");
	eprintln!("[INFO] Ok.");

	return Ok(());
}

/// チェックサムファイルを比較
fn check_checksum() -> Result<(), Box<dyn std::error::Error>> {
	// 開始位置のパス
	let chakra_components = concat_path_parts(&["..", "src", "components", "chakra"]);

	// コンポーネントを消去
	remove_components(&chakra_components)?;

	// コンポーネントを出力
	execute_command(&[
		"npx",
		"chakra",
		"snippet",
		"add",
		"--outdir",
		&chakra_components,
		"--all",
	])?;

	// チェックサムを出力
	let _ = diagnose_files(&chakra_components, "chakra_checksum.tmp")?;

	// 比較
	execute_command(&["diff", "-s", "chakra_checksum.txt", "chakra_checksum.tmp"])?;

	return Ok(());
}

/// 実行
fn execute(args: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
	// コマンドライン引数の解析
	let mut options = getopts::Options::new();
	options.optflag("g", "generate", "generate checksum file");
	options.optflag("c", "check", "check checksum file");
	let matches = options.parse(args)?;

	if matches.opt_present("generate") {
		// チェックサムファイルを生成
		generate_checksum_file()?;
	} else if matches.opt_present("check") {
		// チェックサムファイルを比較
		check_checksum()?;
	} else {
		// エラー
		let error = "オプションが指定されていません。";
		return Err(error.into());
	}
	return Ok(());
}

/// Rust アプリケーションのエントリーポイント
fn main() {
	let args: Vec<String> = std::env::args().skip(1).collect();

	let result = execute(&args);
	if result.is_err() {
		println!("Error: {:?}", result.unwrap_err());
		std::process::exit(1);
	}
}
