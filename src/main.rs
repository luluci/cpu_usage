
use cpu_usage::profile_if::ProfileIF;
/* 
fn log(name: &'static str, id: i32, state: &ProcessState, log_cpu_time_begin: i32, log_cpu_time_end: i32, log_cycle_delayed: bool,) {
	let delay = {
		if log_cycle_delayed {
			"Delayed!"
		} else {
			""
		}
	};
	println!("{:6}-{:6} us, [{}] [{}] : {}", log_cpu_time_begin, log_cpu_time_end, state, name, delay)
} */

fn main() {
	/*
	let mut procs_vec :Vec<Process> = Vec::new();
	procs_vec.push(Process::new(ProcessKind::INTR, "proc1", 1, true, 500, [100].to_vec(), log));
	procs_vec.push(Process::new(ProcessKind::INTR, "proc2", 2, true, 500, [100].to_vec(), log));
	let mut profiler = PlantUML::new();
	profiler.make_header(&procs_vec);
	let mut tracer = ProcessTracer::new(procs_vec);
	tracer.run();
	println!("CPU占有率: {}", tracer.cpu_use_rate);
	*/

	let mut profiler = ProfileIF::new("test".to_string());
	profiler.run();
}
