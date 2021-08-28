use std::io::BufRead;
use regex::Regex;

use crate::process::process::Process;
use crate::process::process::ProcessKind;
use crate::process::process_state::ProcessState;
use crate::process::process_tracer::ProcessTracer;


// 各種設定
use once_cell::sync::OnceCell;
// トレース設定
/// トレース時間
static TRACE_TIME: OnceCell<u32> = OnceCell::new();
// PlantUML
/// 出力ファイル分割時間
static PU_DIVTIME: OnceCell<u32> = OnceCell::new();



enum LoadState {
	/// トレース時間定義解析
	TraceTime,
	/// PlantUML:出力ファイル分割時間解析
	PUDivTime,
	/// プロセス定義解析
	ProcessInfo,
	None,
}

pub struct Settings
{
	/// 設定ファイル
	input_file_path: String,
	/// Regex: unsignedデータ解析
	re_time: Regex,
	/// Regex: プロセス定義解析
	re_process: Regex,
}

impl Settings
{

	pub fn new(input_file_path: String) -> Settings {
		Settings{
			input_file_path,
			re_time: Regex::new(r"(\d+)").unwrap(),
			re_process: Regex::new(r"(\w+)\s+(\w+)\s+(\d+)\s+(\w+)\s+(\d+)((?:\s+(?:\d+))+)").unwrap(),
		}
	}

	pub fn load<T>(&self, cb: &mut T) -> Result<(),String>
		where T: FnMut(ProcessKind, String, i32, bool, i32, Vec<i32>) -> ()
	{
		// ファイルを開く
		let inp_path = std::path::Path::new(&self.input_file_path);
		use std::fs::File;
		//use std::error::Error;
		use std::io::BufReader;
		let file = match File::open(&inp_path) {
			Err(why) => {
				return Err(format!("couldn't open {}: {}", inp_path.display(), why.to_string()));
			},
			Ok(file) => file,
		};
		// ファイル読み込み
		let mut state = LoadState::None;
		for result in BufReader::new(file).lines() {
			if result.is_ok() {
				// 読み込んだテキストを解析
				let line = result.unwrap();
				if line.len() == 0 {
					// 空行はスキップ
				} else if (line.len() >= 2) && (&line[0..1] == "//") {
					// コメントはスキップ
				} else if &line[0..0] == "[" {
					// 先頭が[なら設定状態変更
					state = self.load_process_info_state(&line);
				} else {
					// その他は設定値として解析
					match state {
						LoadState::TraceTime => {
							let capture_opt = self.re_time.captures(&line);
							match capture_opt {
								Some(cap) => {
									// 正規表現がちゃんと定義できていればparseで失敗することはないはず
									let trace_time = cap[1].parse::<u32>().unwrap();
									match TRACE_TIME.set(trace_time) {
										Ok(_) => {
											// 値がセットできた
										},
										Err(_) => {
											// すでに値が設定済み
											return Err("TraceTime settings duplicate!".to_string());
										}
									}
								},
								None => {
									//
								}
							}
						},
						LoadState::PUDivTime => {
							let capture_opt = self.re_time.captures(&line);
							match capture_opt {
								Some(cap) => {
									// 正規表現がちゃんと定義できていればparseで失敗することはないはず
									let time = cap[1].parse::<u32>().unwrap();
									match PU_DIVTIME.set(time) {
										Ok(_) => {
											// 値がセットできた
										},
										Err(_) => {
											// すでに値が設定済み
											return Err("PlantUML DivTime settings duplicate!".to_string());
										}
									}

								},
								None => {
									//
								}
							}
						},
						LoadState::ProcessInfo => {
							// ProcessInfo取得
							// 正規表現でチェック
							let capture = self.re_process.captures(&line);
							match capture {
								Some(caps) => {
									// データ取得
									let name = caps[1].to_string();
									let kind = Settings::load_process_intr_task(&caps[2]);
									let pri: i32 = caps[3].parse::<i32>().unwrap();
									let enable = Settings::load_process_enable(&caps[4]);
									let cycle: i32 = caps[5].parse::<i32>().unwrap();
									// 処理時間は複数指定可能
									let mut time_vec: Vec<i32> = vec![];
									let time = &caps[6];
									for mat in self.re_time.find_iter(time) {
										time_vec.push(mat.as_str().parse::<i32>().unwrap());
									}
									//procs_vec.push(Process::new(kind, name, pri, enable, cycle, [100].to_vec(), cb));
									cb(kind, name, pri, enable, cycle, time_vec);
								}
								None => {
									// 何もしない
								}
							}
						},
						LoadState::None => {
							// Noneは不明な状態なのでスキップ
						}
					}
				}
			}
		}

		Ok(())
	}

	fn load_process_info_state(&self, _text: &str) -> LoadState {
		match _text {
			"[TraceInfo]"			=> LoadState::TraceTime,
			"[PlantUML_DivTime]"	=> LoadState::PUDivTime,
			"[ProcessInfo]"			=> LoadState::ProcessInfo,
			_						=> panic!("undefined Setting: {}", _text),
		}
	}

	fn load_process_intr_task(text: &str) -> ProcessKind {
		match text {
			"INTR" => ProcessKind::INTR,
			"TASK" => ProcessKind::TASK,
			_ => panic!("invalid ProcessKind: {}", text),
		}
	}

	fn load_process_enable(text: &str) -> bool {
		match text {
			"enable" => true,
			"disable" => false,
			_ => panic!("invalid enable/disable: {}", text),
		}
	}

}