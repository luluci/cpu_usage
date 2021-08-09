use crate::process::process::ProcessKind;
use crate::process::process::Process;

pub struct ProcessTracer {
	// プロセスリスト
	procs: Vec<Process>,
	// プロセストレース情報
	active_proc: Option<Process>,
	active_proc_idx: Option<usize>,
	// CPU占有率
	cpu_use_rate: f32,
	cpu_use_busy: i32,
	cpu_use_idle: i32,
}

impl ProcessTracer {
	// コンストラクタ
	pub fn new<'a>(procs: Vec<Process>) -> ProcessTracer {
		ProcessTracer {
			procs: procs,
			active_proc: None,
			active_proc_idx: None,
			cpu_use_rate: 0.0,
			cpu_use_busy: 0,
			cpu_use_idle: 0,
		}
	}

	pub fn run(&mut self) {
		// 計測時間作成
		let timemax = 5 * 1000;
		// 計測時間分のトレース開始
		for cpu_time in 0..timemax {
			// アクティブプロセスの終了チェック
			self.check_running_proc();
			// ディスパッチチェック
			self.check_dispatch(cpu_time);
			// 時間を進める
			self.go_time(cpu_time, 1);
		}
	}

	fn check_running_proc(&mut self) {
		match &mut self.active_proc_idx {
			Some(_idx) => {
				let proc = &mut self.procs[*_idx];
				if proc.is_waiting() {
					self.active_proc = None;
				}
			},
			None => ()
		}
	}

	fn check_dispatch(&mut self, cpu_time:i32) {
		let next_proc = self.get_prior_proc();
		match next_proc {
			Some(_proc) => {
				// 現アクティブプロセスをREADYに
				match &mut self.active_proc {
					Some(_active_proc) => {
						_active_proc.preempt(cpu_time);
					},
					None => {

					}
				}
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
					match ready_proc_idx {
						Some(_ready_proc_idx) => {
							let ready_proc= &self.procs[_ready_proc_idx];
							let mut check = true;
							// アクティブプロセスが割り込みのとき、タスクは割り込めない
							if let ProcessKind::TASK = ready_proc.kind {
								if let ProcessKind::INTR = active_proc.kind {
									check = false;
								}
							}
							if check {
								if active_proc.priority >= ready_proc.priority {
									check = false;
								}
							}
							// ディスパッチ要であればREADYプロセスを選択
							if check {
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
				result = self.get_prior_ready_proc();
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
					}
				}
			}
		}
		// 終了
		result
	}

	fn go_time(&mut self, cpu_time:i32, elapse:i32) {
		match &mut self.active_proc {
			Some(_proc) => {
				_proc.go(cpu_time, elapse);
			},
			None => ()
		}
	}
}
