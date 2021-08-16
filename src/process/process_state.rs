#[derive(Clone, Copy)]
pub enum ProcessState {
	DORMANT,
	WAITING,
	READY,
	RUNNING,
}
impl std::fmt::Display for ProcessState {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			ProcessState::DORMANT => write!(f,"DORMANT"),
			ProcessState::WAITING => write!(f,"WAITING"),
			ProcessState::READY => write!(f,"READY"),
			ProcessState::RUNNING => write!(f,"RUNNING"),
		}
	}
}
