use crate::structs::flow_graph::assignment::Assignment;
use crate::structs::sym_table::AtomId;

pub type FlowId = usize;

#[derive(Clone)]
pub enum FlowKind {
    Assignment(Assignment),
    Seq(Vec<FlowId>, FlowId),
    Branching(BranchingFlow),
    FlowResult(AtomId),
}

impl From<Assignment> for FlowKind {
    fn from(value: Assignment) -> Self {
        Self::Assignment(value)
    }
}

/*impl From<Vec<FlowId>> for FlowKind {
    fn from(value: Vec<FlowId>) -> Self {
        Self::Seq(value)
    }
}*/

impl From<BranchingFlow> for FlowKind {
    fn from(value: BranchingFlow) -> Self {
        Self::Branching(value)
    }
}

#[derive(Clone)]
pub struct Flow {
    pub valid: bool,
    pub parent: Option<FlowId>,
    pub kind: FlowKind,
}

impl<T> From<T> for Flow
where
    T: Into<FlowKind>,
{
    fn from(value: T) -> Self {
        Self {
            valid: true,
            parent: None,
            kind: value.into(),
        }
    }
}

#[derive(Clone)]
pub struct BranchingFlow {
    pub branch: Option<bool>,
    pub cond_flow: FlowId,
    pub true_flow: FlowId,
    pub false_flow: FlowId,
    pub result: FlowId,
}
