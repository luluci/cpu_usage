use crate::process::process::Process;
use crate::process::process::ProcessKind;
use crate::process::process_state::ProcessState;
use crate::process::process_tracer::ProcessTracer;
use crate::process::process_callback::ProcessCallback;
use crate::profiler::profiler::PlantUML;

pub struct ProfileIF
{
	input_file_path: String,
}

impl ProfileIF
{

	pub fn new(input_file_path: String) -> ProfileIF {
		ProfileIF{
			input_file_path,
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

	pub fn run(&mut self) {
		let ( tx, mut rx) = std::sync::mpsc::channel();
		//let txx = &tx;

		let tx_clj = |name: &'static str, id: i32, state: ProcessState, begin: i32, end: i32, delayed: bool| {
			let tx = std::sync::mpsc::Sender::clone(&tx);
			let mut fut = tx.send((name, id, state, begin, end, delayed));
		};
		//let tx_clj = self.make_closure();

		// ファイルからプロセス情報をロード
		//let mut procs_vec :Vec<Process<impl ProcessCallback>> = Vec::new();
		let mut procs_vec = vec![];
		procs_vec.push(Process::new(ProcessKind::INTR, "proc1", 1, true, 500, [100].to_vec(), tx_clj));
		procs_vec.push(Process::new(ProcessKind::INTR, "proc2", 2, true, 500, [100].to_vec(), tx_clj));
		// トレース情報作成
		let mut profiler = PlantUML::new();
		let mut tracer = ProcessTracer::new(procs_vec);
		profiler.make_header(&tracer.procs);


		let rx_clj = || {
			let mut profiler = profiler;
			//let mut rcv;
			// ヘッダを先に出力
			for data in profiler.header.iter_mut() {
				println!("{}", data);
			}
			// 受信データ解析
			// while !finish {
			// 	rcv = rx.recv();
			// 	if rcv.is_ok() {
			// 		let data = rcv.unwrap();
			// 		// プロファイリング
			// 		profiler.profile(data.0, data.1, data.2, data.3, data.4, data.5);
			// 		// データを出力
			// 		let mut buff = profiler.get_body();
			// 		for body in buff.iter_mut() {
			// 			println!("{}", body);
			// 		}
			// 	}
			// }
			for data in rx {
				// プロファイリング
				profiler.profile(data.0, data.1, data.2, data.3, data.4, data.5);
				// データを出力
				let mut buff = profiler.get_body();
				for body in buff.iter_mut() {
					println!("{}", body);
				}
			}
			// 解析後にフッタ出力
			for data in profiler.footer.iter_mut() {
				println!("{}", data);
			}
		};

		let rx_thread = std::thread::spawn(rx_clj);

		tracer.run();
		drop(tx);
		let join_result = rx_thread.join();
		join_result.unwrap();

/* 		let mut profiler = PlantUML::new();
		profiler.make_header(&procs_vec);
		let mut tracer = ProcessTracer::new(procs_vec);
		tracer.run(); */
	}
}