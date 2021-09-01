use crate::process::process::ProcessKind;
use crate::process::process::Process;
use crate::process::process_callback::ProcessCallback;

pub struct ProcessTracer<T>
	where T: ProcessCallback
{
	// プロセスリスト
	pub procs: Vec<Process<T>>,
	// プロセストレース情報
	active_proc_idx: Option<usize>,
	// トレース設定
	task_use_preempt: bool,
	// CPU占有率
	pub cpu_use_rate: f32,
	cpu_use_busy: i32,
	cpu_use_idle: i32,
}

impl<T> ProcessTracer<T>
	where T: ProcessCallback
{
	// コンストラクタ
	pub fn new<'a>(procs: Vec<Process<T>>, task_use_preempt: bool) -> ProcessTracer<T> {
		let mut data = ProcessTracer {
			procs,
			active_proc_idx: None,
			task_use_preempt,
			cpu_use_rate: 0.0,
			cpu_use_busy: 0,
			cpu_use_idle: 0,
		};

		// プロセスIDを設定
		for (idx, _proc) in data.procs.iter_mut().enumerate() {
			_proc.id = idx as i32;
		}

		data
	}

	pub fn run(&mut self, trace_time: i32) {
		// 計測時間作成
		let timemax = trace_time;
		let mut disp_cycle: i32 = 0;
		let mut disp_count: i32 = 0;
		// プロセス初期設定
		self.start_proc();
		// 計測時間分のトレース開始
		for cpu_time in 1..timemax {
			// アクティブプロセスの終了チェック
			self.check_running_proc();
			// ディスパッチチェック
			self.check_dispatch(cpu_time);
			// 時間を進める
			self.go_time(cpu_time, 1);
			// CPU使用カウント
			self.check_cpu_use();

			// 進捗出力: 1sec経過を通知
			disp_cycle += 1;
			if disp_cycle >= 1000000 {
				disp_count += 1;
				disp_cycle = 0;
				println!("{} sec elapsed.", disp_count);
			}
		}
		// CPU占有率計算
		let runtime = timemax as f32;
		self.cpu_use_rate = self.cpu_use_busy as f32 / runtime * 100 as f32;
	}

	fn start_proc(&mut self) {
		for proc in self.procs.iter_mut() {
			//proc.init(0);
		}
	}

	fn check_running_proc(&mut self) {
		match &mut self.active_proc_idx {
			Some(_idx) => {
				let proc = &mut self.procs[*_idx];
				if proc.is_waiting() {
					self.active_proc_idx = None;
				}
			},
			None => ()
		}
	}

	fn check_dispatch(&mut self, cpu_time:i32) {
		// READYプロセスから起動するプロセスを選択
		let next_proc = self.get_prior_proc();
		match next_proc {
			Some(_next_proc_idx) => {
				// 現アクティブプロセスがいればREADYに
				match &mut self.active_proc_idx {
					Some(_active_proc_idx) => {
						let active_proc = &mut self.procs[*_active_proc_idx];
						active_proc.preempt(cpu_time);
					},
					None => {
						// 何もしない
					}
				}
				// アクティブプロセス更新
				self.active_proc_idx = next_proc;
				// 新アクティブプロセスをディスパッチ
				let next_proc = &mut self.procs[_next_proc_idx];
				next_proc.dispatch(cpu_time);
			},
			None => {
				// 何もしない
			}
		}
	}

	fn get_prior_proc<'a>(&mut self) -> Option<usize> {
		let mut result: Option<usize> = None;
		let ready_proc_idx = self.get_prior_ready_proc();
		match self.active_proc_idx {
			Some(_active_proc_idx) => {
				let active_proc = &self.procs[_active_proc_idx];
				if active_proc.multi_intr {
					// 多重割込み許可ならRAEDYプロセスとディスパッチ要否判定
					//let ready_proc_idx = self.get_prior_ready_proc();
					// READYプロセスの存在チェック
					match ready_proc_idx {
						Some(_ready_proc_idx) => {
							// RUNNINGプロセスとREADYプロセスを比較してディスパッチ要否を判定
							let ready_proc= &self.procs[_ready_proc_idx];
							// ディスパッチ要であればREADYプロセスを選択
							if self.check_prior_process(&active_proc, &ready_proc) {
								result = Some(_ready_proc_idx);
							}
						},
						None => {
							// READYプロセスが無ければディスパッチ不要
							// result = None;
						}
					}
				} else {
					// 多重割込み禁止であればアクティブプロセス優先
					// result = None;
				}
			},
			None => {
				// アクティブプロセスが無ければREADYからプロセス選択
				result = ready_proc_idx;
			}
		}
		//
		result
	}

	/**
	READY状態のプロセスから優先度の高いものを選択
	 */
	fn get_prior_ready_proc(&mut self) -> Option<usize> {
		let mut result: Option<usize> = None;
		let mut max_pri = 0;
		let mut max_ready = 0;
		for (i, proc) in self.procs.iter_mut().enumerate() {
			if proc.is_ready() {
				match result {
					Some(_proc) => {
						// READYプロセスが複数あれば優先度で判定
						if proc.priority > max_pri {
							result = Some(i);
							max_pri = proc.priority;
							max_ready = proc.timer_ready;
						} else if proc.priority == max_pri {
							// 優先度が同じ場合はFCFS方式でREADY時間が長い方を選択
							if proc.timer_ready > max_ready {
								result = Some(i);
								max_pri = proc.priority;
								max_ready = proc.timer_ready;
							}
						} else {
							// 優先度が低い場合は何もしない
						}
					},
					None => {
						// 初回出現は無条件セット
						result = Some(i);
						max_pri = proc.priority;
						max_ready = proc.timer_ready;
					}
				}
			}
		}
		// 終了
		result
	}

	fn check_prior_process(&self, running: &Process<T>, ready: &Process<T>) -> bool
		where T: ProcessCallback
	{
		// アクティブプロセスが割り込みのとき、タスクの割り込み不可
		if let ProcessKind::TASK = ready.kind {
			if let ProcessKind::INTR = running.kind {
				return false;
			}
		}

		// READYプロセスの優先度が同じか低いときはディスパッチ不可
		if running.priority >= ready.priority {
			return false;
		}

		// タスクpreemptionが無効の場合はタスク間でのディスパッチ不可
		if !self.task_use_preempt {
			if let ProcessKind::TASK = ready.kind {
				if let ProcessKind::TASK = running.kind {
					return false;
				}
			}
		}

		true
	}

	fn go_time(&mut self, cpu_time:i32, elapse:i32) {
		for proc in self.procs.iter_mut() {
			proc.go(cpu_time, elapse);
		}
	}

	fn check_cpu_use(&mut self) {
		match &mut self.active_proc_idx {
			Some(_idx) => {
				self.cpu_use_busy += 1;
			},
			None => {
				self.cpu_use_idle += 1;
			}
		}
	}

	pub fn output_proc_result(&mut self) {
		// キャプション出力
		println!("[{:40}] | CPU-userate (occur point  )", "Process Name");
		println!("-{0:-<40}--+-{0:-<27}-", "");
		// 各プロセス情報出力
		for proc in self.procs.iter() {
			println!("{}", proc);
		}
		// 全体CPU使用率出力
		let delay: &str;
		if self.cpu_use_rate > 100.0 {
			delay = "(delayed!)";
		} else {
			delay = "";
		}
		println!("-{0:-<40}--+-{0:-<27}-", "");
		println!("[{:40}] | {:10.2}% {:15} {}", "all", self.cpu_use_rate, "", delay);
	}
}
