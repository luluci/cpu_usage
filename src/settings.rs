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
pub static TRACE_TIME: OnceCell<i32> = OnceCell::new();
pub static TASK_USE_PREEMPT: OnceCell<bool> = OnceCell::new();
// PlantUML
pub static PU_ENABLE: OnceCell<bool> = OnceCell::new();
/// 出力ファイル分割時間
pub static PU_DIVTIME: OnceCell<i32> = OnceCell::new();



enum LoadState {
	/// トレース時間定義解析
	TraceInfo,
	/// PlantUML:出力ファイル分割時間解析
	PlantUML,
	/// プロセス定義解析
	ProcessInfo,
	None,
}

pub struct Settings
{
	// 正規表現定義
	/// Regex: unsignedデータ解析
	re_trace_info: Regex,
	re_plant_uml: Regex,
	/// Regex: プロセス定義解析
	re_process: Regex,
	re_time: Regex,
	// 設定ファイルから読みだしてOnceCellに渡すデータ
	trace_time: i32,		// トレース時間
	task_use_preempt: bool,		// 自動的にpreempt実施するかどうか
	pu_enable: bool,
	pu_divtime: i32,
}

impl Settings
{

	pub fn new() -> Settings {
		// Settingsインスタンス作成
		Settings{
			re_trace_info: Regex::new(r"(\w+)\s*=\s*(\w+)").unwrap(),
			re_plant_uml: Regex::new(r"(\w+)\s*=\s*(\w+)").unwrap(),
			re_process: Regex::new(r"(\w+)\s+(\w+)\s+(\w+)\s+(\d+)\s+(\w+)\s+(\d+)((?:\s+(?:\d+))+)").unwrap(),
			re_time: Regex::new(r"(\w+)").unwrap(),
			trace_time: 0,
			task_use_preempt: true,
			pu_enable: false,
			pu_divtime: 0,
		}
	}

	pub fn load<T>(&mut self, input_file_path: &String, cb: &mut T) -> Result<(),String>
		where T: FnMut(ProcessKind, String, ProcessState, i32, bool, i32, Vec<i32>) -> ()
	{
		// ファイルを開く
		let inp_path = std::path::Path::new(&input_file_path);
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
				} else if &line[..1] == "[" {
					// 先頭が[なら設定状態変更
					state = self.check_load_state(&line);
				} else {
					// その他は設定値として解析
					match state {
						LoadState::TraceInfo => {
							self.load_trace_info(&line);
						},
						LoadState::PlantUML => {
							self.load_plant_uml(&line);
						},
						LoadState::ProcessInfo => {
							self.load_process(&line, cb);
						},
						LoadState::None => {
							// Noneは不明な状態なのでスキップ
						}
					}
				}
			}
		}

		// 読み込みが完了したらグローバル変数にセット
		match TRACE_TIME.set(self.trace_time) {
			Ok(_) => {}
			Err(_) => {}
		}
		match TASK_USE_PREEMPT.set(self.task_use_preempt) {
			Ok(_) => {}
			Err(_) => {}
		}
		match PU_ENABLE.set(self.pu_enable) {
			Ok(_) => {}
			Err(_) => {}
		}
		match PU_DIVTIME.set(self.pu_divtime) {
			Ok(_) => {}
			Err(_) => {}
		}

		Ok(())
	}

	fn check_load_state(&mut self, _text: &str) -> LoadState {
		match _text {
			"[TraceInfo]"		=> LoadState::TraceInfo,
			"[PlantUML]" => {
				self.pu_enable = true;
				LoadState::PlantUML
			},
			"[ProcessInfo]"		=> LoadState::ProcessInfo,
			_					=> panic!("undefined Setting: {}", _text),
		}
	}

	fn load_trace_info(&mut self, _text: &str) {
		let capture_opt = self.re_trace_info.captures(_text);
		match capture_opt {
			Some(cap) => {
				let key = &cap[1];
				let val = &cap[2];
				match key {
					"TraceTime" => {
						match val.parse::<i32>() {
							Ok(time) => {
								self.trace_time = time;
							},
							Err(_) => {
								println!("invalid TraceTime: {}", val);
							}
						}
					}
					"TaskUsePreemption" => {
						match Settings::load_bool(val) {
							Ok(enable) => {
								self.task_use_preempt = enable;
							},
							Err(_) => {
								println!("invalid TaskUsePreemption: {}", val);
							}
						}
					}
					_ => {
						// 何もしない
					}
				}
			},
			None => {
				// マッチしないものはスキップ
			}
		}
	}

	fn load_plant_uml(&mut self, _text: &str) {
		let capture_opt = self.re_plant_uml.captures(&_text);
		match capture_opt {
			Some(cap) => {
				let key = &cap[1];
				let val = &cap[2];
				match key {
					"Enable" => {
						match Settings::load_bool(val) {
							Ok(enable) => {
								self.pu_enable = enable;
							},
							Err(_) => {
								println!("invalid bool value: {}", val);
							}
						}
					}
					"DivTime" => {
						match val.parse::<i32>() {
							Ok(time) => {
								self.pu_divtime = time;
							},
							Err(_) => {
								println!("invalid DivTime: {}", val);
							}
						}
					}
					_ => {
						// 何もしない
					}
				}
			},
			None => {
				//
			}
		}
	}

	pub fn load_process<T>(&mut self, _text: &str, cb: &mut T)
		where T: FnMut(ProcessKind, String, ProcessState, i32, bool, i32, Vec<i32>) -> ()
	{
		// ProcessInfo取得
		// 正規表現でチェック
		let capture = self.re_process.captures(&_text);
		match capture {
			Some(caps) => {
				// データ取得
				let name = caps[1].to_string();
				let kind = Settings::load_process_intr_task(&caps[2]);
				let state = Settings::load_process_state(&caps[3]);
				let pri: i32 = caps[4].parse::<i32>().unwrap();
				let enable = Settings::load_process_enable(&caps[5]);
				let cycle: i32 = caps[6].parse::<i32>().unwrap();
				// 処理時間は複数指定可能
				let mut time_vec: Vec<i32> = vec![];
				let time = &caps[7];
				for mat in self.re_time.find_iter(time) {
					time_vec.push(mat.as_str().parse::<i32>().unwrap());
				}
				//procs_vec.push(Process::new(kind, name, pri, enable, cycle, [100].to_vec(), cb));
				cb(kind, name, state, pri, enable, cycle, time_vec);
			}
			None => {
				// 何もしない
			}
		}
	}

	fn load_process_intr_task(text: &str) -> ProcessKind {
		match text {
			"INTR" => ProcessKind::INTR,
			"TASK" => ProcessKind::TASK,
			_ => panic!("invalid ProcessKind: {}", text),
		}
	}

	fn load_process_state(text: &str) -> ProcessState {
		match text {
			"WAITING" => ProcessState::WAITING,
			"READY" => ProcessState::READY,
			"DORMANT" => panic!("{} is not allowed initial state.", text),
			"RUNNING" => panic!("{} is not allowed initial state.", text),
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

	fn load_bool(text: &str) -> Result<bool,String> {
		match text {
			"true" => Ok(true),
			"false" => Ok(false),
			_ => Err(format!("invalid true/false: {}", text)),
		}
	}

}