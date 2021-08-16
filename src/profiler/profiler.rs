use std::collections::{HashMap, LinkedList};

use crate::process::process::Process;
use crate::process::process_state::ProcessState;
use crate::process::process_callback::ProcessCallback;


type BuffContainer = LinkedList<String>;

pub struct PlantUML {
	// PlantUMLデータ
	pub header: BuffContainer,
	pub body: BuffContainer,
	pub footer: BuffContainer,
	// 制御データ
	last_time_proc: HashMap<i32,i32>,		// プロセス毎の最新CPU時間
	last_time: i32,							// 前回CPU時間
}

impl PlantUML {

	pub fn new() -> PlantUML {
		PlantUML{
			header: BuffContainer::new(),
			body: BuffContainer::new(),
			footer: BuffContainer::new(),
			last_time_proc: HashMap::new(),
			last_time: -1,
		}
	}

	pub fn get_body(&mut self) -> BuffContainer {
		let new_body = BuffContainer::new();
		std::mem::replace(&mut self.body, new_body)
	}

	pub fn make_header<T: ProcessCallback>(&mut self, _procs: &Vec<Process<T>>) {
		// ヘッダ初期化
		self.header.push_back("@startuml CPUusage".to_string());
		self.header.push_back("scale 5 as 5 pixels".to_string());
		for _proc in _procs.iter() {
			self.header.push_back(format!("robust \"{}\" as W{}", _proc.name, _proc.id));
			self.last_time_proc.insert(_proc.id, -1);
		}
	}

	pub fn profile(&mut self, name: &'static str, id: i32, state: ProcessState, log_cpu_time_begin: i32, log_cpu_time_end: i32, log_cycle_delayed: bool,) {
		// ログ時間チェック
		// 時間補正:同じプロセス内で時間が重複したら+1して見た目上ずらす
		let mut fixed_time = log_cpu_time_begin;
		if self.last_time_proc[&id] == fixed_time {
			fixed_time += 1;
		}
		// 前回地更新
		if let Some(time) = self.last_time_proc.get_mut(&id) {
			*time = log_cpu_time_begin;
		}

		// CPU時間変化判定:前回出力と差異があれば@timeを出力する
		if self.last_time != fixed_time {
			// PlamtUML生成
			self.body.push_back("".to_string());
			self.body.push_back(format!("@{}", fixed_time));
			// 時間更新
			self.last_time = fixed_time;
		}
		// PlamtUML生成
		self.body.push_back(format!("W{} is {}", id, state));
		// ハイライト設定
		if log_cycle_delayed {
			self.footer.push_back(format!("highlight {} to {} #Gold;line:DimGrey : 割り込みつぶれ({})", log_cpu_time_begin, log_cpu_time_end, name));
		}
	}

	pub fn make_footer(&mut self) {
		self.footer.push_back("@enduml".to_string());
	}
}
