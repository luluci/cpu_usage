
use std::io::Write;

use std::collections::{HashMap, LinkedList};

use crate::process::process::Process;
use crate::process::process_state::ProcessState;
use crate::process::process_callback::ProcessCallback;


type BuffContainer = LinkedList<String>;

pub struct PlantUML {
	// 出力ファイル
	out_file_base: String,
	output_fs: Option<std::io::BufWriter<std::fs::File>>,
	// ファイル分割
	div_enable: bool,
	div_time: i32,
	div_count: i32,
	div_next: i32,
	div_width: i32,
	// PlantUMLデータ
	pub header: BuffContainer,
	pub body: BuffContainer,
	pub footer: BuffContainer,
	// 制御データ
	last_time_proc: HashMap<i32,i32>,		// プロセス毎の最新CPU時間
	last_time: i32,							// 前回CPU時間
}

impl PlantUML {

	pub fn new(_inp_base: &str, div_time: i32, trace_time: i32) -> PlantUML {
		let mut div_enable = false;
		let mut div_next = 0;
		let mut div_max = 0;
		let mut div_width: i32 = 1;
		let div_count = 1;
		let mut temp = 0;
		if div_time > 0 {
			div_enable = true;
			div_next = div_time;
			// 分割最大数計算
			div_max = trace_time / div_time;
			if trace_time % div_time != 0 {
				div_max += 1;
			}
			// 分割数の桁数
			temp = div_max;
			while temp > 10 {
				div_width += 1;
				temp /= 10;
			}
		}

		PlantUML{
			out_file_base: _inp_base.to_string(),
			output_fs: None,
			div_enable,
			div_time,
			div_count,
			div_next,
			div_width,
			header: BuffContainer::new(),
			body: BuffContainer::new(),
			footer: BuffContainer::new(),
			last_time_proc: HashMap::new(),
			last_time: -1,
		}
	}

	pub fn make_header<T: ProcessCallback>(&mut self, _procs: &Vec<Process<T>>) {
		// 初期値設定
		let mut init_value = BuffContainer::new();
		// ヘッダ初期化
		self.header.push_back("@startuml CPUusage".to_string());
		self.header.push_back("scale 5 as 5 pixels".to_string());
		for _proc in _procs.iter() {
			self.header.push_back(format!("robust \"{}\" as W{}", _proc.name, _proc.id));
			init_value.push_back(format!("W{} is WAITING", _proc.id));
			self.last_time_proc.insert(_proc.id, -1);
		}
		self.header.push_back("".to_string());
		self.header.append(&mut init_value);
	}

	fn open_file(&mut self) -> Result<(),String> {
		// ファイル名作成
		let out_file: String;
		if self.div_enable {
			out_file = format!("{0}_{1:02$}.plantuml", self.out_file_base, self.div_count, self.div_width as usize);
		} else {
			out_file = format!("{}.plantuml", self.out_file_base);
		}
		// ファイルオープンチェック
		let out_path = std::path::Path::new(&out_file);
		match std::fs::File::create(out_path) {
			Ok(file) => {
				self.output_fs = Some(std::io::BufWriter::new(file));
				Ok(())
			}
			Err(why) => {
				return Err(format!("couldn't open {}: {}", out_path.display(), why.to_string()));
			}
		}
	}

	pub fn start(&mut self) -> Result<(),String> {
		// ファイルオープン
		match self.open_file() {
			Ok(_) => (),
			Err(why) => {
				return Err(why);
			},
		}
		// ヘッダを先に出力
		if let Some(writer) = self.output_fs.as_mut() {
			for data in self.header.iter() {
				writeln!(writer, "{}", data);
			}
		}

		Ok(())
	}

	pub fn profile(&mut self, name: &String, id: i32, state: ProcessState, log_cpu_time_begin: i32, log_cpu_time_end: i32, log_cycle_delayed: bool,) {
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

		// ログ出力
		self.output_body();

		// ファイル分割チェック
		if self.div_enable {
			if log_cpu_time_end >= self.div_next {
				// 分割情報更新
				self.div_next += self.div_time;
				self.div_count += 1;
				// ファイルを閉じる
				self.finish();
				// ファイルを再度開く
				self.start();
			}
		}
	}

	pub fn get_body(&mut self) -> BuffContainer {
		let new_body = BuffContainer::new();
		std::mem::replace(&mut self.body, new_body)
	}

	pub fn output_body(&mut self) {
		// データを出力
		let mut buff = self.get_body();
		if let Some(writer) = self.output_fs.as_mut() {
			for data in buff.iter_mut() {
				writeln!(writer, "{}", data);
			}
		}
	}

	pub fn finish(&mut self) {
		// footer作成
		self.make_footer();
		// footer取得
		let mut buff = self.get_footer();
		// footer出力
		if let Some(writer) = self.output_fs.as_mut() {
			// 空行を挟む
			writeln!(writer, "");
			for data in buff.iter_mut() {
				//println!("{}", data);
				writeln!(writer, "{}", data);
			}
		}
		// ファイルを閉じる
		self.output_fs = None;
	}

	pub fn make_footer(&mut self) {
		self.footer.push_back("@enduml".to_string());
	}

	pub fn get_footer(&mut self) -> BuffContainer {
		let new_footer = BuffContainer::new();
		std::mem::replace(&mut self.footer, new_footer)
	}

}
