use crate::structs::chronicle::interval::Interval;
use crate::structs::chronicle::sym_table::SymTable;
use crate::structs::chronicle::type_table::AtomType;
use crate::structs::chronicle::{AtomId, FormatWithParent, FormatWithSymTable, GetVariables};
use im::HashSet;

#[derive(Clone)]
pub struct Effect {
    pub interval: Interval,
    pub sv: Vec<AtomId>,
    pub value: AtomId,
}

impl Effect {
    pub fn get_start(&self) -> &AtomId {
        self.interval.start()
    }

    pub fn get_end(&self) -> &AtomId {
        self.interval.end()
    }
}

impl FormatWithSymTable for Effect {
    fn format(&self, st: &SymTable, sym_version: bool) -> String {
        format!(
            "{} {} <- {}",
            self.interval.format(st, sym_version),
            self.sv.format(st, sym_version),
            self.value.format(st, sym_version),
        )
    }
}

impl FormatWithParent for Effect {
    fn format_with_parent(&mut self, st: &SymTable) {
        self.interval.format_with_parent(st);
        self.sv.format_with_parent(st);
        self.value.format_with_parent(st);
    }
}

impl GetVariables for Effect {
    fn get_variables(&self) -> HashSet<AtomId> {
        let mut union = self.interval.get_variables();
        self.sv.iter().for_each(|a| {
            union.insert(*a);
        });
        union.insert(self.value);
        union
    }

    fn get_variables_of_type(&self, sym_table: &SymTable, atom_type: &AtomType) -> HashSet<AtomId> {
        self.get_variables()
            .iter()
            .filter(|v| sym_table.get_type_of(v).unwrap() == atom_type)
            .cloned()
            .collect()
    }
}
