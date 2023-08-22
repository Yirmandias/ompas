use crate::model::acting_domain::parameters::Parameters;
use sompas_structs::lvalue::LValue;
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Clone)]
pub struct Method {
    pub label: String,
    pub task_label: String,
    pub parameters: Parameters,
    pub lambda_pre_conditions: LValue,
    pub lambda_score: LValue,
    pub lambda_body: LValue,
}

//Getters
impl Method {
    pub fn get_task_label(&self) -> &String {
        &self.task_label
    }

    pub fn get_parameters(&self) -> &Parameters {
        &self.parameters
    }

    pub fn get_pre_conditions(&self) -> &LValue {
        &self.lambda_pre_conditions
    }

    pub fn get_body(&self) -> &LValue {
        &self.lambda_body
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }
}

impl Method {
    pub fn new(
        label: String,
        task_label: String,
        parameters: Parameters,
        conds: LValue,
        score: LValue,
        body: LValue,
    ) -> Self {
        Self {
            label,
            task_label,
            parameters,
            lambda_pre_conditions: conds,
            lambda_score: score,
            lambda_body: body,
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "-task: {}\n\
            -parameters: {}\n\
            -pre-conditions: {}\n\
            -score: {}\n\
            -body: {}\n",
            self.task_label,
            self.parameters,
            self.lambda_pre_conditions.format("pre-conditions: ".len()),
            self.lambda_score.format("score: ".len()),
            self.lambda_body.format("body: ".len())
        )
    }
}
