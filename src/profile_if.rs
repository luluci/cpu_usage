use std::io::BufRead;
use std::io::Write;

use crate::process::process::Process;
use crate::process::process::ProcessKind;
use crate::process::process_state::ProcessState;
use crate::process::process_tracer::ProcessTracer;
use crate::profiler::profiler::PlantUML;

pub struct ProfileIF
{
	input_file_path: String,
	output_file_path: String,
}

impl ProfileIF
{

	pub fn new(input_file_path: String, output_file_path: String) -> ProfileIF {
		ProfileIF{
			input_file_path,
			output_file_path,
		}
	}

	// fn make_closure(&tx: tokio::sync::mpsc::Sender<(&str, i32, ProcessState, i32, i32, bool)>) -> impl ProcessCallback {
	// 	|name: &'static str, id: i32, state: ProcessState, begin: i32, end: i32, delayed: bool| {
	// 		tokio::spawn(async move{
	// 			tx.send((name, id, state, begin, end, delayed)).await;
	// 		});
	// 	}
	// }

	// fn handler(&mut self, name: &'static str, id: i32, state: ProcessState, begin: i32, end: i32, delayed: bool) {

	// }

	pub fn load_process_info<T>(&self, cb: &mut T) -> i32
		where T: FnMut(ProcessKind, String, i32, bool, i32, Vec<i32>) -> ()
	{
		let mut trace_time: i32 = 0;
		let inp_path = std::path::Path::new(&self.input_file_path);
		// ファイルを開く
		use std::fs::File;
		//use std::error::Error;
		use std::io::BufReader;
		use regex::Regex;
		let file = match File::open(&inp_path) {
			Err(why) => {
				panic!("couldn't open {}: {}", inp_path.display(), why.to_string());
			},
			Ok(file) => file,
		};
		// ファイル読み込み
		// ファイル記載変換クロージャ
		let fn_intr_task = |s: &str| -> ProcessKind {
			match s {
				"INTR" => ProcessKind::INTR,
				"TASK" => ProcessKind::TASK,
				_ => panic!("invalid ProcessKind: {}", s),
			}
		};
		let fn_enable = |s: &str| -> bool {
			match s {
				"enable" => true,
				"disable" => false,
				_ => panic!("invalid enable/disable: {}", s),
			}
		};
		// 正規表現定義
		let mut read_mode = 0;
		let re = Regex::new(r"(\w+)\s+(\w+)\s+(\d+)\s+(\w+)\s+(\d+)((?:\s+(?:\d+))+)").unwrap();
		let re_time = Regex::new(r"(\d+)").unwrap();
		let re_trace_info = Regex::new(r"(\d+)").unwrap();
		//let mut procs_vec = vec![];
		for result in BufReader::new(file).lines() {
			if result.is_ok() {
				let line = result.unwrap();
				if &line == "[TraceInfo]" {
					read_mode = 1;
				} else if &line == "[ProcessInfo]" {
					read_mode = 2;
				} else if (line.len() >= 2) && (&line[0..1] == "//") {
					// コメントはスキップ
				} else {
					match read_mode {
						1 => {
							let capture_opt = re_trace_info.captures(&line);
							match capture_opt {
								Some(cap) => {
									trace_time = cap[1].parse::<i32>().unwrap();
								},
								None => {
									//
								}
							}
						},
						2 => {
							// ProcessInfo取得
							// 正規表現でチェック
							let capture = re.captures(&line);
							match capture {
								Some(caps) => {
									// データ取得
									let name = caps[1].to_string();
									let kind = fn_intr_task(&caps[2]);
									let pri: i32 = caps[3].parse::<i32>().unwrap();
									let enable = fn_enable(&caps[4]);
									let cycle: i32 = caps[5].parse::<i32>().unwrap();
									// 処理時間は複数指定可能
									let mut time_vec: Vec<i32> = vec![];
									let time = &caps[6];
									for mat in re_time.find_iter(time) {
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
						_ => {
							// 
						}
					}
				}
			}
		}

		trace_time
	}

	pub fn run(&mut self) {
		let ( tx, rx) = std::sync::mpsc::channel();

		// ファイルオープンチェック
		let out_path = std::path::Path::new(&self.output_file_path);
		let file = match std::fs::File::create(out_path) {
			Err(why) => {
				panic!("couldn't open {}: {}", out_path.display(), why.to_string());
			},
			Ok(file) => file,
		};

		let tx_clj = |name: &String, id: i32, state: ProcessState, begin: i32, end: i32, delayed: bool| {
			let tx = std::sync::mpsc::Sender::clone(&tx);
			let fut = tx.send((name.clone(), id, state, begin, end, delayed));

		};
		//let tx_clj = self.make_closure();
		let mut procs_vec = vec![];
		let mut init_clj = |kind: ProcessKind, name: String, pri: i32, enable: bool, cycle:i32, time: Vec<i32>| {
			procs_vec.push(Process::new(kind, name, pri, enable, cycle, time, tx_clj));
		};
		let trace_time = self.load_process_info(&mut init_clj);

		// ファイルからプロセス情報をロード
		//let mut procs_vec :Vec<Process<impl ProcessCallback>> = Vec::new();
		//let mut procs_vec = vec![];
		//procs_vec.push(Process::new(ProcessKind::INTR, "proc1", 1, true, 500, [100].to_vec(), tx_clj));
		//procs_vec.push(Process::new(ProcessKind::INTR, "proc2", 2, true, 500, [100].to_vec(), tx_clj));
		// トレース情報作成
		let mut profiler = PlantUML::new();
		let mut tracer = ProcessTracer::new(procs_vec);
		profiler.make_header(&tracer.procs);


		let rx_clj = || {
			let mut profiler = profiler;
			let mut writer = std::io::BufWriter::new(file);
			// ヘッダを先に出力
			for data in profiler.header.iter_mut() {
				//println!("{}", data);
				writeln!(writer, "{}", data);
			}
			// txがすべて破棄されるとrxもループを終了する
			for data in rx {
				// プロファイリング
				profiler.profile(&data.0, data.1, data.2, data.3, data.4, data.5);
				// データを出力
				let mut buff = profiler.get_body();
				for data in buff.iter_mut() {
					//println!("{}", body);
					writeln!(writer, "{}", data);
				}
			}
			// 解析後にフッタ出力
			profiler.make_footer();
			for data in profiler.footer.iter_mut() {
				//println!("{}", data);
				writeln!(writer, "{}", data);
			}
		};

		// 解析処理を別スレッドに投げる
		// ファイルI/Oを考えてこうしてるが、チャネルによるメッセージングの方がコスト高い可能性？
		let rx_thread = std::thread::spawn(rx_clj);
		// メインスレッドでプロセストレース実施
		tracer.run(trace_time);
		// 各プロセスの状況を出力
		tracer.output_proc_result();
		// トレース終了したらtxを破棄してワーカースレッド終了
		drop(tx);
		// 一応スレッド終了を待機
		let join_result = rx_thread.join();
		join_result.unwrap();
	}
}