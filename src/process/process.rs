use crate::process::process_state::ProcessState;
use crate::process::process_callback::ProcessCallback;

pub enum ProcessKind {
	INTR,
	TASK,
}

pub struct Process<T>
	where T: ProcessCallback
{
	// プロセス情報
	pub id: i32,					// プロセスID
	pub kind: ProcessKind,			// プロセス種類
	pub priority: i32,				// 優先度
	pub multi_intr: bool,			// 多重割込み許可
	time_cycle: i32,				// 起動周期
	time_proc: Vec<i32>,			// 処理時間[Max,Ave1,Ave2,...]
	time_proc_idx: usize,			// 処理時間選択idx
	pub name: String,			// プロセス名
	// プロセス制御情報
	state: ProcessState,			// 状態
	timer_cycle: i32,				// 起動周期タイマ
	pub timer_ready: i32,			// READY時間タイマ
	timer_run: i32,					// RUNNING時間タイマ
	// ログ情報
	log_cpu_time: i32,				// プロセス起動時CPU時間
	log_cycle_delayed: bool,		// 処理遅延有無
	log_callback: T,				// ログ生成時のコールバック関数
	// プロセス毎最大占有率
	max_cpu_use_rate: f32,			// プロセス占有率:起動周期当たりに占める時間割合
	max_cpu_time: i32,				// 最大占有率発生時のCPU時間
}

impl<T> Process<T>
	where T: ProcessCallback
{
	// コンストラクタ
	pub fn new(kind: ProcessKind, name: String, state: ProcessState, priority:i32, multi_intr:bool, time_cycle: i32, time_proc: Vec<i32>, cb: T) -> Process<T> {
		Process{
			id: -1,
			kind: kind,
			priority: priority,
			multi_intr: multi_intr,
			time_cycle: time_cycle,
			time_proc: time_proc,
			time_proc_idx: 0,
			name: name,
			state,
			timer_cycle: 0,
			timer_ready: 0,
			timer_run: 0,
			log_cpu_time: 0,
			log_cycle_delayed: false,
			log_callback: cb,
			// プロセス毎最大占有率
			max_cpu_use_rate: 0.0,
			max_cpu_time: 0,
		}
	}
	// INTRプロセスファクトリ
	pub fn intr(name: String, state: ProcessState, priority:i32, multi_intr:bool, time_cycle: i32, time_proc: Vec<i32>, cb: T) -> Process<T> {
		Process{
			id: -1,
			kind: ProcessKind::INTR,
			priority: priority,
			multi_intr: multi_intr,
			time_cycle: time_cycle,
			time_proc: time_proc,
			time_proc_idx: 0,
			name: name,
			state,
			timer_cycle: 0,
			timer_ready: 0,
			timer_run: 0,
			log_cpu_time: 0,
			log_cycle_delayed: false,
			log_callback: cb,
			// プロセス毎最大占有率
			max_cpu_use_rate: 0.0,
			max_cpu_time: 0,
		}
	}
	// TASKプロセスファクトリ
	pub fn task(name: String, state: ProcessState, priority:i32, multi_intr:bool, time_cycle: i32, time_proc: Vec<i32>, cb: T) -> Process<T> {
		Process{
			id: -1,
			kind: ProcessKind::TASK,
			priority: priority,
			multi_intr: multi_intr,
			time_cycle: time_cycle,
			time_proc: time_proc,
			time_proc_idx: 0,
			name: name,
			state,
			timer_cycle: 0,
			timer_ready: 0,
			timer_run: 0,
			log_cpu_time: 0,
			log_cycle_delayed: false,
			log_callback: cb,
			// プロセス毎最大占有率
			max_cpu_use_rate: 0.0,
			max_cpu_time: 0,
		}
	}

	pub fn go(&mut self, cpu_time:i32, elapse:i32) {
		// 経過時間更新
		self.timer_cycle += elapse;
		// 起動周期チェック
		self.check_cycle();
		// 状態処理
		self.check_state(cpu_time, elapse);
	}

	fn check_cycle(&mut self) {
		// 起動周期経過？
		if self.timer_cycle >= self.time_cycle {
			// 状態毎処理
			match self.state {
				// WAITINGでは処理なし
				ProcessState::WAITING => (),
				// READY中に次の起動周期が来てしまったため、処理つぶれが発生している
				ProcessState::READY => self.log_cycle_delayed = true,
				// RUNNING中に次の起動周期が来てしまったため、処理つぶれが発生している
				ProcessState::RUNNING => self.log_cycle_delayed = true,
				// DORMANTは不使用
				ProcessState::DORMANT => (),
			}
		}
	}

	fn check_state(&mut self, cpu_time:i32, elapse:i32) {
		// 状態毎処理
		match self.state {
			// WAITING処理
			ProcessState::WAITING => self.check_state_waiting(cpu_time),
			// READY処理
			ProcessState::READY => self.check_state_ready(elapse),
			// RUNNING処理
			ProcessState::RUNNING => self.check_state_running(cpu_time, elapse),
			// DORMANTは不使用
			ProcessState::DORMANT => self.check_state_dormant(),
		}
	}

	fn check_state_waiting(&mut self, cpu_time:i32) {
		if self.timer_cycle >= self.time_cycle {
			// 起動周期到達でタスク起床
			self.wakeup(cpu_time);
			// 起動周期タイマ初期化
			self.timer_cycle = 0;
		}
	}

	fn check_state_ready(&mut self, elapse:i32) {
		// READYは上位からディスパッチされるまで待機
		// 状態時間更新
		self.timer_ready += elapse;
	}

	fn check_state_running(&mut self, cpu_time:i32, elapse:i32) {
		// 状態時間更新
		self.timer_run += elapse;
		// 処理時間経過判定
		if self.timer_run >= self.time_proc[self.time_proc_idx] {
			// RUNNING終了してWAITINGへ
			// 処理時間idx更新
			self.time_proc_idx += 1;
			if self.time_proc_idx >= self.time_proc.len() {
				self.time_proc_idx = 0;
			}
			// 占有率計算
			self.calc_cpu_usage(cpu_time);
			// 状態遷移
			self.waiting(cpu_time);
			//
			self.timer_run = 0;
			self.timer_ready = 0;
		}
	}

	fn check_state_dormant(&mut self) {
		// 処理なし
	}

	fn calc_cpu_usage(&mut self, cpu_time:i32) {
		// プロセスが有効になっていた時間
		let active_time = self.timer_run + self.timer_ready;
		// 起動周期に占める割合＝CPU占有率
		let userate: f32 = active_time as f32 / self.time_cycle as f32 * 100.0;
		// 最大CPU占有率を覚えておく
		if userate > self.max_cpu_use_rate {
			self.max_cpu_use_rate = userate;
			self.max_cpu_time = cpu_time;
		}
	}

	pub fn init(&mut self, cpu_time:i32) {
		// ログ登録
		self.push_log(cpu_time);
	}

	pub fn wakeup(&mut self, cpu_time:i32) {
		// ログ登録
		self.push_log(cpu_time);
		// READYに遷移
		self.state = ProcessState::READY;
	}

	pub fn waiting(&mut self, cpu_time:i32) {
		// ログ登録
		self.push_log(cpu_time);
		// WAITINGに遷移
		self.state = ProcessState::WAITING;
	}

	pub fn dispatch(&mut self, cpu_time:i32) {
		// ログ登録
		self.push_log(cpu_time);
		// RUNNINGに遷移
		self.state = ProcessState::RUNNING;
	}

	pub fn preempt(&mut self, cpu_time:i32) {
		// ログ登録
		self.push_log(cpu_time);
		// READYに遷移
		self.state = ProcessState::READY;
	}

	pub fn is_waiting(&mut self) -> bool {
		if let ProcessState::WAITING = self.state {
			true
		} else {
			false
		}
	}

	pub fn is_ready(&mut self) -> bool {
		if let ProcessState::READY = self.state {
			true
		} else {
			false
		}
	}

	pub fn is_running(&mut self) -> bool {
		if let ProcessState::RUNNING = self.state {
			true
		} else {
			false
		}
	}


	/**
	ログ登録
	現在状態をログとしてプッシュする
	*/
	fn push_log(&mut self, cpu_time:i32) {
		// ログを通知
		(self.log_callback)(
			&self.name,
			self.id,
			self.state,
			self.log_cpu_time,
			cpu_time,
			self.log_cycle_delayed,
		);
		// ログクリア
		self.log_cpu_time = cpu_time;
		self.log_cycle_delayed = false;
	}
}

impl<T> std::fmt::Display for Process<T>
	where T: ProcessCallback
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut delay = "".to_string();
		if self.max_cpu_use_rate > 100.0 {
			delay = "(delayed!)".to_string();
		}
		write!(f, "[{:40}] | {:10.2}% (at {:10}) {}", &self.name, &self.max_cpu_use_rate, &self.max_cpu_time, delay)
	}
}
