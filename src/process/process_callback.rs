use crate::process::process_state::ProcessState;

/*
type LogCallback = fn(
	name: &'static str,				// プロセス名
	id: i32,						// プロセスID
	state: &ProcessState,			// プロセス状態
	log_cpu_time_begin: i32,		// 状態開始時CPU時間
	log_cpu_time_end: i32,			// 状態終了時CPU時間
	log_cycle_delayed: bool,		// 処理遅延有無
) -> ();
 */

pub trait ProcessCallback: FnMut(&'static str, i32, ProcessState, i32, i32, bool) -> ()
{}

impl<T> ProcessCallback for T
	where T: FnMut(&'static str, i32, ProcessState, i32, i32, bool) -> ()
{}

