//!
//! バッチ処理
//!

#[macro_export]
macro_rules! info {
	($($arg:tt)*) => ({
		let timestamp = crate::util::get_current_timestamp_jst();
		eprintln!("{} [info] {}", timestamp, format!($($arg)*));
	})
}

#[macro_export]
macro_rules! warn {
	($($arg:tt)*) => ({
		let timestamp = crate::util::get_current_timestamp_jst();
		eprintln!("{} [warn] {}", timestamp, format!($($arg)*));
	})
}

#[macro_export]
macro_rules! error {
	($($arg:tt)*) => ({
		let timestamp = crate::util::get_current_timestamp_jst();
		eprintln!("{} [error] {}", timestamp, format!($($arg)*));
	})
}

const WIN32: bool = cfg!(windows);

mod util {

	#[allow(unused)]
	pub fn get_current_timestamp_utc() -> String {
		let now = chrono::Local::now();
		let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
		return timestamp;
	}

	pub fn get_current_timestamp_jst() -> String {
		let now = chrono::Utc::now();
		let local_time = now + chrono::Duration::hours(9);
		let timestamp = local_time.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
		// +#03 -> 符号付き、最低3桁を確保する、0埋め、整数
		let text = format!("{}{:+#03}:00", timestamp, 9);
		return text;
	}

	/// コマンドを実行
	pub fn execute_command_ex(command: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
		let result = std::process::Command::new(command[0])
			.args(&command[1..])
			.stderr(std::process::Stdio::inherit())
			.stdout(std::process::Stdio::inherit())
			.output()?;
		let exit_code = result.status.code().unwrap_or_default();
		return Ok(exit_code);
	}

	/// コマンドを実行
	pub fn execute_command(command: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
		let result = std::process::Command::new(command[0])
			.args(&command[1..])
			.stderr(std::process::Stdio::inherit())
			.stdout(std::process::Stdio::inherit())
			.output()?;
		if !result.status.success() {
			let exit_code = result.status.code().unwrap_or_default();
			let command_string = command.join(" ");
			let error = format!("Command exited with status: {} {}", exit_code, command_string);
			return Err(error.into());
		}
		return Ok(());
	}

	/// ファイルをバイナリで読み込み
	fn read_file_binary(path: &str) -> std::result::Result<std::vec::Vec<u8>, Box<dyn std::error::Error>> {
		use std::io::Read;
		// ファイルを開きます。
		let mut file = std::fs::File::open(path)?;

		// 読み取り用バッファ
		let mut buffer = std::vec::Vec::new();

		// ファイル全体を読み込み
		file.read_to_end(&mut buffer)?;

		return Ok(buffer);
	}

	/// ファイルの MD5 をダンプ
	pub fn generate_md5sum(path: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
		let buffer = read_file_binary(path)?;

		// MD5 を計算
		let response = md5::compute(buffer);

		return Ok(format!("{:x}", response));
	}

	/// ディレクトリを変更
	#[allow(unused)]
	fn chdir(path: &str) -> Result<(), Box<dyn std::error::Error>> {
		std::env::set_current_dir(path)?;
		return Ok(());
	}

	/// OS を考慮したパス結合
	pub fn concat_path_parts(parts: &[&str]) -> String {
		return parts.join(&std::path::MAIN_SEPARATOR.to_string());
	}
}

mod application {
	use crate::{info, util};

	/// パスの補正(出力用)
	///
	/// # Arguments
	/// * `path` - 元のパス文字列
	fn fix_path_string(path: &str) -> String {
		let path = path.replace("\\", "/");
		let path = path.trim_start_matches("../").to_string();
		return path;
	}

	/// Chakra UI のコンポーネントファイル群を診断し、チェックサムファイルを生成します。
	///
	/// # Arguments
	/// * `path` - 対象ディレクトリのパス
	/// * `outpath` - 出力ファイルのパス
	fn diagnose_files(path: &str, outpath: &str) -> Result<(), Box<dyn std::error::Error>> {
		use std::io::Write;

		// 相対パスのリスト
		let mut chakra_files: Vec<String> = vec![];

		// ディレクトリ内のファイルを列挙
		let directory = std::fs::read_dir(path)?;
		for entry in directory {
			let entry = entry?;
			let path = entry.path();
			chakra_files.push(path.to_string_lossy().to_string());
		}

		let file = std::fs::File::create(outpath)?;
		let mut writer = std::io::BufWriter::new(&file);

		// 破壊的ソート
		chakra_files.sort();

		for path in &chakra_files {
			// ファイル情報
			let path_info = std::fs::metadata(&path)?;
			// ファイルサイズ
			let length = path_info.len();
			// MD5
			let md5sum = util::generate_md5sum(&path)?;

			writer.write_all(format!("{}, {}, {}\n", fix_path_string(&path), length, md5sum).as_bytes())?;
		}

		return Ok(());
	}

	/// Chakra UI のコンポーネントファイルを削除
	fn remove_components(path: &str) -> Result<(), Box<dyn std::error::Error>> {
		let reader = std::fs::read_dir(path)?;
		for entry in reader {
			let entry = entry?;
			let path = entry.path();
			let name = path.file_name().unwrap_or_default().to_str().unwrap_or_default();
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
		let chakra_components = detect_src_component_location()?;

		// チェックサムを出力
		let _ = diagnose_files(&chakra_components, "chakra_checksum.txt")?;

		// 差分の診断
		let affected = git_diff_checksum("chakra_checksum.txt")?;

		if affected > 0 {
			info!("出力された .tsx ファイルと checksum ファイルをリポジトリーに push してください。")
		} else {
			info!("チェックサムファイルに差分は検出されませんでした。");
		}

		info!("Ok.");

		return Ok(());
	}

	/// コンポーネントの場所を検出
	fn detect_src_component_location() -> Result<String, Box<dyn std::error::Error>> {
		return Ok(util::concat_path_parts(&["..", "src", "components", "chakra"]));
	}

	/// チェックサムファイルを比較します。
	fn compare_checksum_files() -> Result<(), Box<dyn std::error::Error>> {
		info!("チェックサムを比較します。");

		let exit_code = if crate::WIN32 {
			util::execute_command_ex(&[
				"wsl.exe",
				"diff",
				"-s",
				"-w",
				"./chakra_checksum.txt",
				"./chakra_checksum.tmp",
			])?
		} else {
			util::execute_command_ex(&["diff", "-s", "-w", "./chakra_checksum.txt", "./chakra_checksum.tmp"])?
		};

		if exit_code != 0 {
			let error = "チェックサムが一致しません。";
			return Err(error.into());
		}

		eprintln!("チェックサムが一致しました。code: {}", exit_code);

		return Ok(());
	}

	/// チェックサムファイルを比較
	fn check_checksum() -> Result<(), Box<dyn std::error::Error>> {
		// 開始位置のパス
		let chakra_components = detect_src_component_location()?;

		// コンポーネントを消去
		remove_components(&chakra_components)?;

		// コンポーネントを出力
		util::execute_command(&[
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
		compare_checksum_files()?;

		return Ok(());
	}

	// 差分行を3つのフィールドに分割します。
	fn split_diff_line(line: &str) -> [String; 3] {
		let parts: Vec<String> = line.split(", ").map(|s| s.to_string()).collect();
		let mut result = [String::new(), String::new(), String::new()];
		for i in 0..3 {
			if i < parts.len() {
				result[i] = parts[i].clone();
			}
		}
		return result;
	}

	/// OUT.tmp を診断します。
	fn analyze_out_tmp(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
		use std::io::BufRead;

		let file = std::fs::File::open(path)?;
		let reader = std::io::BufReader::new(file);

		let mut names_set = std::collections::BTreeSet::<String>::new();

		for line in reader.lines() {
			let line = line?;
			if line.starts_with("---") {
				continue;
			}
			if line.starts_with("+++") {
				continue;
			}
			if line.starts_with("@@") {
				continue;
			}

			if line.starts_with("-") {
				let parts = split_diff_line(&line[1..]);
				names_set.insert(parts[0].clone());
				continue;
			}
			if line.starts_with("+") {
				let parts = split_diff_line(&line[1..]);
				names_set.insert(parts[0].clone());
				continue;
			}
		}

		if names_set.is_empty() {
			info!("差分は検出されませんでした。");
		} else {
			info!("差分が検出されました。影響を受けるファイル一覧:");
			for name in &names_set {
				info!("- {}", name);
			}
		}

		return Ok(names_set.len());
	}

	/// パス補正
	///
	/// # Arguments
	/// * `path` - 元のパス文字列
	fn fix_path_prefix_for_linux(path: &str) -> String {
		if path.starts_with(".") {
			return path.to_string();
		}
		if path.starts_with("/") {
			return path.to_string();
		}
		return format!("./{}", path);
	}

	/// git diff でチェックサムファイルを比較
	fn git_diff_checksum(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
		if crate::WIN32 {
			let path = fix_path_prefix_for_linux(path);
			util::execute_command(&["wsl.exe", "git", "diff", &path, ">OUT.tmp"])?;
		} else {
			let path = fix_path_prefix_for_linux(path);
			util::execute_command(&["git", "diff", &path, ">OUT.tmp"])?;
		}

		// 出力ファイルを解析
		let affected = analyze_out_tmp("OUT.tmp")?;

		// 一時ファイルを削除
		let _ = std::fs::remove_file("OUT.tmp");

		return Ok(affected);
	}

	/// 実行
	///
	/// # Arguments
	/// * `args` - コマンドライン引数
	pub fn execute(args: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
		// コマンドライン引数の解析
		let mut options = getopts::Options::new();
		options.optflag("g", "generate", "generate checksum file");
		options.optflag("c", "check", "check checksum file");
		let matches = options.parse(args)?;

		if matches.opt_present("generate") {
			// ========== チェックサムファイルを生成 ==========
			generate_checksum_file()?;
		} else if matches.opt_present("check") {
			// ========== チェックサムファイルを比較 ==========
			check_checksum()?;
		} else {
			// ========== USAGE ==========
			eprintln!("{}", options.usage("Usage:"));
			return Ok(());
		}
		return Ok(());
	}
}

/// Rust アプリケーションのエントリーポイント
fn main() {
	let args: Vec<String> = std::env::args().skip(1).collect();

	let result = application::execute(&args);
	if result.is_err() {
		let error = result.unwrap_err();
		error!("{:?}", error);
		std::process::exit(1);
	}
}
