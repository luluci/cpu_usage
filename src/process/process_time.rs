
pub struct ProcessTime {
	pub cycle: i32,
	pub proc: Vec<i32>,
}

impl ProcessTime {
	pub fn new(cycle: i32, proc:&'static [i32]) -> ProcessTime {
		ProcessTime{
			cycle: cycle,
			proc: proc.to_vec(),
		}
	}
}
