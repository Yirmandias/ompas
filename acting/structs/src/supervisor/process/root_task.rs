use crate::supervisor::process::ActingProcessInner;
use crate::supervisor::ActingProcessId;
use std::fmt::{Display, Formatter};

pub struct RootProcess {
    _id: ActingProcessId,
    pub tasks: Vec<ActingProcessId>,
}

impl RootProcess {
    pub fn new() -> Self {
        Self {
            _id: 0,
            tasks: vec![],
        }
    }

    pub fn n_task(&self) -> usize {
        self.tasks.len()
    }

    pub fn nth_task(&self, rank: usize) -> Option<ActingProcessId> {
        self.tasks.get(rank).copied()
    }

    pub fn add_top_level_task(&mut self, id: ActingProcessId) {
        self.tasks.push(id)
    }
}

impl From<RootProcess> for ActingProcessInner {
    fn from(value: RootProcess) -> Self {
        Self::RootTask(value)
    }
}

impl Display for RootProcess {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(0) Root")
    }
}
