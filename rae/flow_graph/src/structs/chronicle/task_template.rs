use crate::structs::chronicle::chronicle::ChronicleTemplate;
use crate::structs::chronicle::AtomId;

pub struct TaskTemplate {
    pub name: Vec<AtomId>,
    pub methods: Vec<ChronicleTemplate>,
}
