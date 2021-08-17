
use cpu_usage::profile_if::ProfileIF;

use std::path::Path;
use std::ffi::OsStr;

fn get_args_in_out() -> (String, String) {
	//let args: Vec<String> = std::env::args_os().into_iter().map(|arg| arg.to_string_lossy().to_string()).collect();
	//let args_os: Vec<std::ffi::OsString> = std::env::args_os().collect();
	//let input = args_os[1].to_string_lossy().to_string();
	let args: Vec<String> = std::env::args().collect();
	let input = args[1].clone();

	// Path作成補助クロージャ
	let fn_opt_path = |opt: Option<&Path>| {
		match opt {
			Some(path) => path.display().to_string(),
			None => "<failed>".to_string(),
		}
	};
	let fn_opt_osstr = |opt: Option<&OsStr>| {
		match opt {
			Some(osstr) => osstr.to_string_lossy().to_string(),
			None => "<failed>".to_string(),
		}
	};
	// Path作成
	let input_path = Path::new(&input);
	let inp_parent = fn_opt_path(input_path.parent());
	let inp_stem = fn_opt_osstr(input_path.file_stem());
	let output = format!("{}/{}.plantuml", inp_parent, inp_stem);
	//let output_path = Path::new(&output);

	return (input, output);
}


fn main() {
	let (ip, op) = get_args_in_out();

	let mut profiler = ProfileIF::new(ip, op);
	profiler.run();
}
