use std::io::Write;

use crate::process::process::Process;
use crate::process::process::ProcessKind;
use crate::process::process_state::ProcessState;
use crate::process::process_tracer::ProcessTracer;
use crate::profiler::profiler::PlantUML;
use crate::settings;


pub struct ProfileIF
{
	input_file_path: String,
	output_file_path: String,
}

impl ProfileIF
{

	pub fn new(input_file_path: String) -> ProfileIF {
		ProfileIF{
			input_file_path,
			output_file_path: "".to_string(),
		}
	}

	pub fn run(&mut self, inp_base: String) {
		let ( tx, rx) = std::sync::mpsc::channel();

		let tx_clj = |name: &String, id: i32, state: ProcessState, begin: i32, end: i32, delayed: bool| {
			let tx = std::sync::mpsc::Sender::clone(&tx);
			let fut = tx.send((name.clone(), id, state, begin, end, delayed));
		};
		//let tx_clj = self.make_closure();
		let mut procs_vec = vec![];
		let mut init_clj = |kind: ProcessKind, name: String, state: ProcessState, pri: i32, enable: bool, cycle:i32, time: Vec<i32>| {
			procs_vec.push(Process::new(kind, name, state, pri, enable, cycle, time, tx_clj));
		};
		//let trace_time = self.load_process_info(&mut init_clj);
		// ファイルから設定を読み出し
		let mut setting = settings::Settings::new();
		let load_result = setting.load(&self.input_file_path, &mut init_clj);
		match load_result {
			Ok(_) => (),
			Err(msg) => {
				panic!("setting file error: {}", msg);
			}
		}
		let trace_time = *settings::TRACE_TIME.get().unwrap();
		let pu_enable = *settings::PU_ENABLE.get().unwrap();
		let pu_div_time = *settings::PU_DIVTIME.get().unwrap();

		// トレース情報作成
		let mut tracer = ProcessTracer::new(procs_vec);
		let mut profiler_pu;
		if pu_enable {
			let mut pu = PlantUML::new(&inp_base, pu_div_time, trace_time);
			pu.make_header(&tracer.procs);
			profiler_pu = Some(pu);
		} else {
			profiler_pu = None;
		}


		let rx_clj = || {
			let mut profiler_pu = profiler_pu;

			// profiler前処理
			if let Some(profiler) = profiler_pu.as_mut() {
				let result = profiler.start();
				if result.is_err() {
					profiler_pu = None;
				}
			}

			// プロファイリング
			// txがすべて破棄されるとrxもループを終了する
			for data in rx {
				// PlantUML
				if let Some(profiler) = profiler_pu.as_mut() {
					profiler.profile(&data.0, data.1, data.2, data.3, data.4, data.5);
				}
			}

			// profiler後処理
			if let Some(profiler) = profiler_pu.as_mut() {
				profiler.finish();
			}
		};

		// 解析処理を別スレッドに投げる
		// ファイルI/Oを考えてこうしてるが、チャネルによるメッセージングの方がコスト高い可能性？
		let rx_thread = std::thread::spawn(rx_clj);
		// メインスレッドでプロセストレース実施
		println!(">> trace start.");
		tracer.run(trace_time);
		println!(">> trace finish.");
		println!("");
		// 各プロセスの状況を出力
		tracer.output_proc_result();
		// トレース終了したらtxを破棄してワーカースレッド終了
		drop(tx);
		// 一応スレッド終了を待機
		let join_result = rx_thread.join();
		join_result.unwrap();

		{
			let mut buf = "".to_string();
			println!("");
			println!("Press Enter Key:");
			std::io::stdin().read_line(&mut buf);
		}
	}
}