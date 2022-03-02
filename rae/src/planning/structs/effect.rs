use crate::planning::structs::atom::AtomType;
use crate::planning::structs::interval::Interval;
use crate::planning::structs::symbol_table::{AtomId, SymTable};
use crate::planning::structs::traits::{FormatWithSymTable, GetVariables};
use crate::planning::structs::transition::Transition;
use im::HashSet;

#[derive(Clone)]
pub struct Effect {
    pub interval: Interval,
    pub transition: Transition,
}

impl FormatWithSymTable for Effect {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        format!(
            "{} {}",
            self.interval.format_with_sym_table(st),
            self.transition.format_with_sym_table(st)
        )
    }
}

impl GetVariables for Effect {
    fn get_variables(&self) -> HashSet<AtomId> {
        self.interval
            .get_variables()
            .union(self.transition.get_variables())
    }

    fn get_variables_of_type(&self, sym_table: &SymTable, atom_type: &AtomType) -> HashSet<AtomId> {
        self.get_variables()
            .iter()
            .filter(|v| sym_table.get_type(v).unwrap() == atom_type)
            .cloned()
            .collect()
    }
}