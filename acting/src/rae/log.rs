use crate::rae::structs::TaskId;
use crate::rae::task::TaskId;

pub struct RAEStatus {
    pub task: TaskId,
    pub msg: String,
}
