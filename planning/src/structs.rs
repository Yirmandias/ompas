use crate::structs::Sym::Unique;
use crate::union_find::{Forest, Node, NodeId};
use ompas_acting::rae::context::rae_env::DomainEnv;
use ompas_acting::rae::module::rae_exec::platform::RAE_INSTANCE;
use ompas_acting::rae::module::rae_exec::{RAE_ASSERT, RAE_RETRACT};
use ompas_lisp::core::root_module::language::get_scheme_primitives;
use ompas_lisp::core::structs::lenv::LEnv;
use ompas_lisp::core::structs::lerror::LError;
use ompas_lisp::core::structs::lerror::LError::SpecialError;
use ompas_lisp::core::structs::lnumber::LNumber;
use ompas_lisp::core::structs::lvalue::LValue;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Atom {
    Bool(bool),
    Number(LNumber),
    Sym(Sym),
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Bool(true) => write!(f, "true"),
            Atom::Bool(false) => write!(f, "nil"),
            Atom::Number(n) => write!(f, "{}", n),
            Atom::Sym(s) => write!(f, "{}", s),
        }
    }
}

impl Default for Atom {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Self::Sym(s.into())
    }
}

impl From<bool> for Atom {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<&LNumber> for Atom {
    fn from(n: &LNumber) -> Self {
        Self::Number(n.clone())
    }
}

impl From<LNumber> for Atom {
    fn from(n: LNumber) -> Self {
        (&n).into()
    }
}

impl From<&Sym> for Atom {
    fn from(sym: &Sym) -> Self {
        Self::Sym(sym.clone())
    }
}

impl From<Sym> for Atom {
    fn from(sym: Sym) -> Self {
        (&sym).into()
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Sym {
    Unique(String),
    Several(String, usize),
}

impl Default for Sym {
    fn default() -> Self {
        Unique("".to_string())
    }
}

impl Sym {
    pub fn get_string(&self) -> &String {
        match self {
            Sym::Unique(s) => s,
            Sym::Several(s, _) => s,
        }
    }
}

impl From<String> for Sym {
    fn from(s: String) -> Self {
        Self::Unique(s)
    }
}

impl From<&str> for Sym {
    fn from(s: &str) -> Self {
        Self::Unique(s.to_string())
    }
}

impl Display for Sym {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unique(s) => write!(f, "{}", s),
            Self::Several(s, i) => write!(f, "{}_{}", s, i),
        }
    }
}

type AtomId = NodeId;

pub trait Absorb {
    fn absorb(&mut self, other: Self);
}

pub trait FormatWithSymTable {
    fn format_with_sym_table(&self, st: &SymTable) -> String;
}

pub enum ExpressionType {
    Pure,
    Lisp,
    Action,
    Task,
    StateFunction,
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub enum AtomType {
    Number,
    Boolean,
    Timepoint,
    Result,
    Symbol,
    Object,
    Action,
    StateFunction,
    Method,
    Task,
    Function,
    Lambda,
}

impl Display for AtomType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomType::Timepoint => write!(f, "timepoint"),
            AtomType::Result => write!(f, "return"),
            AtomType::Object => write!(f, "object"),
            AtomType::Action => write!(f, "action"),
            AtomType::StateFunction => write!(f, "state-function"),
            AtomType::Method => write!(f, "method"),
            AtomType::Task => write!(f, "task"),
            AtomType::Function => write!(f, "function"),
            AtomType::Number => write!(f, "number"),
            AtomType::Boolean => write!(f, "boolean"),
            AtomType::Symbol => write!(f, "symbol"),
            AtomType::Lambda => write!(f, "lambda"),
        }
    }
}

#[derive(Default, Clone)]
pub struct SymbolTypes {
    inner: HashMap<AtomId, AtomType>,
    types_number: TypesNumber,
}

impl SymbolTypes {
    pub fn get_type(&self, atom_id: &AtomId) -> Option<&AtomType> {
        self.inner.get(atom_id)
    }

    pub fn get_number_of_type(&self, atom_type: &AtomType) -> usize {
        self.types_number.get_number_of_type(atom_type)
    }

    pub fn add_new_atom(&mut self, id: AtomId, atom_type: AtomType) {
        self.inner.insert(id, atom_type);
        self.types_number.increase_number_of_type(&atom_type);
    }
}

#[derive(Clone)]
pub struct TypesNumber {
    inner: HashMap<AtomType, usize>,
}

impl TypesNumber {
    pub fn increase_number_of_type(&mut self, atom_type: &AtomType) -> usize {
        let n = self.inner.get_mut(atom_type).unwrap();
        let previous = *n;
        *n += 1;
        previous
    }

    pub fn get_number_of_type(&self, atom_type: &AtomType) -> usize {
        *self.inner.get(atom_type).unwrap()
    }
}

impl Default for TypesNumber {
    fn default() -> Self {
        let mut types_number = HashMap::new();
        types_number.insert(AtomType::Number, 0);
        types_number.insert(AtomType::Boolean, 0);
        types_number.insert(AtomType::Symbol, 0);
        types_number.insert(AtomType::Result, 0);
        types_number.insert(AtomType::Timepoint, 0);
        types_number.insert(AtomType::Object, 0);
        types_number.insert(AtomType::Action, 0);
        types_number.insert(AtomType::StateFunction, 0);
        types_number.insert(AtomType::Method, 0);
        types_number.insert(AtomType::Task, 0);
        types_number.insert(AtomType::Function, 0);
        types_number.insert(AtomType::Lambda, 1);
        Self {
            inner: types_number,
        }
    }
}

#[derive(Clone)]
pub struct SymTable {
    symbols: Forest<Atom>,
    ids: HashMap<Sym, AtomId>,
    symbol_types: SymbolTypes,
    multiple_def: HashMap<String, Vec<AtomId>>,
    pointer_to_ver: Vec<HashMap<String, usize>>,
}

impl SymTable {
    pub fn get_sym(&self, id: &AtomId) -> &Atom {
        self.get_atom(&self.get_parent(id)).unwrap()
    }
}

//Forest methods
impl SymTable {
    pub fn union_atom(&mut self, a: &AtomId, b: &AtomId) {
        self.symbols.union(a, b);
    }

    pub fn find_parent(&mut self, a: &AtomId) -> AtomId {
        self.symbols.find(a)
    }

    pub fn get_parent(&self, a: &AtomId) -> AtomId {
        self.symbols.get_parent(a)
    }
}

impl Default for SymTable {
    fn default() -> Self {
        let mut st = Self {
            symbols: Forest::default(),
            ids: Default::default(),
            symbol_types: Default::default(),
            multiple_def: Default::default(),
            pointer_to_ver: vec![Default::default()],
        };

        //Symbols of lisp functions that are useful
        //Not exhaustive
        st.add_list_of_symbols_of_same_type(
            get_scheme_primitives()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            &AtomType::Function,
        )
        .expect("error while adding symbols of scheme primitives");
        st.add_list_of_symbols_of_same_type(
            vec![
                RAE_ASSERT.to_string(),
                RAE_RETRACT.to_string(),
                RAE_INSTANCE.to_string(),
            ],
            &AtomType::Function,
        )
        .expect("error while adding symbols of rae");
        st
    }
}

impl SymTable {
    pub fn add_list_of_symbols_of_same_type(
        &mut self,
        list: Vec<String>,
        sym_type: &AtomType,
    ) -> Result<(), LError> {
        for element in &list {
            if self.it_exists(element) {
                return Err(SpecialError(
                    "add_list_of_symbols_of_same_type",
                    format!("{} already exists", element),
                ));
            }

            let id = self.symbols.new_node(element.as_str().into());
            self.ids.insert(element.as_str().into(), id);
            self.symbol_types.add_new_atom(id, *sym_type);
        }
        Ok(())
    }
}

impl SymTable {
    pub fn get_node(&self, id: &AtomId) -> Option<&Node<Atom>> {
        self.symbols.get_node(id)
    }

    pub fn get_atom(&self, id: &AtomId) -> Option<&Atom> {
        self.symbols.get_value(id)
    }

    pub fn get_type(&self, id: &AtomId) -> Option<&AtomType> {
        self.symbol_types.get_type(id)
    }

    pub fn id(&self, sym: &str) -> Option<&AtomId> {
        //Look before in the multiple_def table, and then looking in self.ids
        if self.multiple_def.contains_key(sym) {
            let ver = self.pointer_to_ver.last().unwrap().get(sym).unwrap();
            let value = self.multiple_def.get(sym).unwrap().get(*ver);
            value
        } else {
            self.ids.get(&sym.into())
        }
    }

    pub fn new_bool(&mut self, b: bool) -> AtomId {
        let id = self.symbols.new_node(b.into());
        self.symbol_types.add_new_atom(id, AtomType::Boolean);
        id
    }

    pub fn new_number(&mut self, n: LNumber) -> AtomId {
        let id = self.symbols.new_node(n.into());
        self.symbol_types.add_new_atom(id, AtomType::Number);
        id
    }

    //Declare a new return value
    //The name of the return value will be format!("r_{}", last_return_index)
    pub fn declare_new_result(&mut self) -> ExpressionChronicleResult {
        let n = self.symbol_types.get_number_of_type(&AtomType::Result);
        let sym: Sym = format!("r_{}", n).into();
        let id = self.symbols.new_node((&sym).into());
        self.ids.insert(sym, id);
        self.symbol_types.add_new_atom(id, AtomType::Result);
        ExpressionChronicleResult { id, pure: None }
    }

    pub fn unique_to_several(&mut self, sym: &str) {
        if !self.multiple_def.contains_key(sym) {
            //change value in vec of symbol
            let id = self.ids.remove(&Unique(sym.to_string())).unwrap();
            let value = Sym::Several(sym.to_string(), 0);
            self.symbols.set_value(&id, (&value).into());
            //Update key in hashmap
            self.ids.insert(value, id);
            //Create new entry in multiple_def
            self.multiple_def.insert(sym.to_string(), vec![id]);
            self.pointer_to_ver
                .last_mut()
                .unwrap()
                .insert(sym.to_string(), 0);
        }
    }

    pub fn declare_new_interval(&mut self) -> Interval {
        let n = self.symbol_types.get_number_of_type(&AtomType::Timepoint);
        let start: Sym = format!("t_{}", n).into();
        let end: Sym = format!("t_{}", n + 1).into();
        let id_1 = self.symbols.new_node((&start).into());
        let id_2 = self.symbols.new_node((&end).into());
        self.ids.insert(start, id_1);
        self.symbol_types.add_new_atom(id_1, AtomType::Timepoint);
        self.ids.insert(end, id_2);
        self.symbol_types.add_new_atom(id_2, AtomType::Timepoint);
        Interval {
            start: id_1,
            end: id_2,
        }
    }

    pub fn new_scope(&mut self) {
        self.pointer_to_ver
            .push(self.pointer_to_ver.last().unwrap().clone())
    }

    pub fn revert_scope(&mut self) {
        self.pointer_to_ver.remove(self.pointer_to_ver.len() - 1);
    }

    pub fn declare_new_timepoint(&mut self) -> AtomId {
        let n = self.symbol_types.get_number_of_type(&AtomType::Timepoint);
        let sym: Sym = format!("t_{}", n).into();
        let id = self.symbols.new_node((&sym).into());
        self.ids.insert(sym, id);
        self.symbol_types.add_new_atom(id, AtomType::Timepoint);
        id
    }

    pub fn it_exists(&self, sym: &str) -> bool {
        self.ids.keys().any(|k| k.get_string() == sym)
    }

    pub fn declare_new_symbol(&mut self, symbol: String, if_it_exists_create_new: bool) -> AtomId {
        if self.it_exists(&symbol) {
            return if if_it_exists_create_new {
                self.unique_to_several(&symbol);
                let vec_similar = self.multiple_def.get_mut(&symbol).unwrap();
                let n = vec_similar.len();
                *self
                    .pointer_to_ver
                    .last_mut()
                    .unwrap()
                    .get_mut(&symbol)
                    .unwrap() = n;
                let id = self.symbols.new_node(Sym::Several(symbol, n).into());
                self.symbol_types.add_new_atom(id, AtomType::Symbol);
                vec_similar.push(id);
                id
            } else {
                return *match self.pointer_to_ver.last().unwrap().get(&symbol) {
                    None => self.ids.get(&symbol.into()).unwrap(),
                    Some(i) => self.multiple_def.get(&symbol).unwrap().get(*i).unwrap(),
                };
            };
        } else {
            let sym: Sym = symbol.into();
            let id = self.symbols.new_node((&sym).into());
            self.ids.insert(sym, id);
            self.symbol_types.add_new_atom(id, AtomType::Symbol);
            id
        }
    }

    pub fn declare_new_object(&mut self) -> AtomId {
        let n = self.symbol_types.get_number_of_type(&AtomType::Object);
        let sym: Sym = format!("o_{}", n).into();
        let id = self.symbols.new_node(sym.clone().into());
        self.ids.insert(sym, id);
        self.symbol_types.add_new_atom(id, AtomType::Object);
        id
    }
}

#[derive(Clone, Default)]
pub struct Chronicle {
    name: Lit,
    task: Lit,
    partial_chronicle: PartialChronicle,
    debug: Option<LValue>,
}

impl FormatWithSymTable for Chronicle {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        let mut s = String::new();
        //name
        s.push_str(format!("-name: {}\n", self.name.format_with_sym_table(st)).as_str());
        //task
        s.push_str(format!("-task: {}\n", self.task.format_with_sym_table(st)).as_str());
        s.push_str(self.partial_chronicle.format_with_sym_table(st).as_str());

        //Debug
        if let Some(exp) = &self.debug {
            s.push_str(format!("debug: {}", exp.format("debug: ".len())).as_str());
            //s.push_str(format!("debug: {:?}", exp).as_str());
        }

        s
    }
}

impl Chronicle {
    pub fn set_debug(&mut self, debug: Option<LValue>) {
        self.debug = debug;
    }
}

impl Chronicle {
    pub fn absorb_expression_chronicle(&mut self, ec: ExpressionChronicle) {
        self.partial_chronicle.absorb(ec.partial_chronicle);
        //add result
        self.add_var(&ec.result.get_id());

        //add interval
        self.add_interval(&ec.interval);

        //add new subtask
    }
}

impl Chronicle {
    pub fn add_var(&mut self, sym_id: &AtomId) {
        self.partial_chronicle.add_var(sym_id);
    }
    pub fn add_interval(&mut self, interval: &Interval) {
        self.partial_chronicle.add_interval(interval);
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.partial_chronicle.add_constraint(constraint);
    }

    pub fn add_condition(&mut self, cond: Condition) {
        self.partial_chronicle.add_condition(cond)
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.partial_chronicle.add_effect(effect)
    }

    pub fn add_subtask(&mut self, sub_task: Expression) {
        self.partial_chronicle.add_subtask(sub_task)
    }

    pub fn set_name(&mut self, name: Lit) {
        self.name = name;
    }

    pub fn set_task(&mut self, task: Lit) {
        self.task = task;
    }
}

#[derive(Clone, Default)]
pub struct PartialChronicle {
    variables: HashSet<AtomId>,
    constraints: Vec<Constraint>,
    conditions: Vec<Condition>,
    effects: Vec<Effect>,
    subtasks: Vec<Expression>,
}

impl FormatWithSymTable for PartialChronicle {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        let mut s = String::new();
        s.push_str("-variable(s): {");

        let mut variables = self
            .variables
            .iter()
            .map(|id| st.get_sym(id).to_string())
            .collect::<Vec<String>>();
        variables.sort();

        for (i, sym) in variables.iter().enumerate() {
            if i != 0 {
                s.push(',');
            }
            s.push_str(sym);
        }
        s.push_str("}\n");

        s.push_str("-constraint(s): {\n");
        for c in &self.constraints {
            s.push('\t');
            s.push_str(c.format_with_sym_table(st).as_str());
            s.push('\n');
        }
        s.push_str("}\n");

        //conditions
        s.push_str("-conditon(s): {\n");
        for e in &self.conditions {
            s.push('\t');
            s.push_str(e.format_with_sym_table(st).as_str());
            s.push('\n');
        }
        s.push_str("}\n");
        //effects
        s.push_str("-effect(s): {\n");
        for e in &self.effects {
            s.push('\t');
            s.push_str(e.format_with_sym_table(st).as_str());
            s.push('\n');
        }
        s.push_str("}\n");
        //substasks
        s.push_str("-subtask(s): {\n");
        for e in &self.subtasks {
            s.push('\t');
            s.push_str(e.format_with_sym_table(st).as_str());
            s.push('\n');
        }
        s.push_str("}\n");
        s
    }
}

impl Absorb for PartialChronicle {
    fn absorb(&mut self, mut other: Self) {
        self.variables = self.variables.union(&other.variables).copied().collect();
        self.constraints.append(&mut other.constraints);
        self.conditions.append(&mut other.conditions);
        self.effects.append(&mut other.effects);
        self.subtasks.append(&mut other.subtasks);
    }
}

impl PartialChronicle {
    pub fn rm_var(&mut self, sym_id: &AtomId) {
        self.variables.remove(sym_id);
    }

    pub fn rm_set_var(&mut self, ids: Vec<AtomId>) {
        for id in ids {
            self.rm_var(&id);
        }
    }

    pub fn rm_constraint(&mut self, index: usize) {
        self.constraints.remove(index);
    }

    pub fn rm_set_constraint(&mut self, mut indexes: Vec<usize>) {
        indexes.reverse();
        for index in indexes {
            self.rm_constraint(index);
        }
    }
}

impl PartialChronicle {
    pub fn add_var(&mut self, sym_id: &AtomId) {
        self.variables.insert(*sym_id);
    }
    pub fn add_interval(&mut self, interval: &Interval) {
        self.variables.insert(interval.start);
        self.variables.insert(interval.end);
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn add_condition(&mut self, cond: Condition) {
        self.conditions.push(cond);
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn add_subtask(&mut self, sub_task: Expression) {
        self.subtasks.push(sub_task);
    }

    pub fn get_constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }

    pub fn get_variables(&self) -> &HashSet<AtomId> {
        &self.variables
    }
}

#[derive(Clone, Default)]
pub struct ExpressionChronicleResult {
    id: AtomId,
    pure: Option<Lit>,
}

impl ExpressionChronicleResult {
    pub fn get_id(&self) -> &AtomId {
        &self.id
    }

    pub fn set_pure(&mut self, lit: Lit) {
        self.pure = Some(lit)
    }

    pub fn is_pure(&self) -> bool {
        self.pure.is_some()
    }

    pub fn get_pure(&self) -> &Option<Lit> {
        &self.pure
    }
}

impl From<ExpressionChronicleResult> for Lit {
    fn from(ecr: ExpressionChronicleResult) -> Self {
        match ecr.pure {
            Some(lit) => lit.clone(),
            None => ecr.id.into(),
        }
    }
}

pub struct ExpressionChronicle {
    interval: Interval,
    result: ExpressionChronicleResult,
    partial_chronicle: PartialChronicle,
    value: Lit,
    debug: LValue,
}

impl ExpressionChronicle {
    pub fn rm_var(&mut self, sym_id: &AtomId) {
        self.partial_chronicle.rm_var(sym_id);
    }

    pub fn rm_set_var(&mut self, ids: Vec<AtomId>) {
        self.partial_chronicle.rm_set_var(ids)
    }

    pub fn rm_constraint(&mut self, index: usize) {
        self.partial_chronicle.rm_constraint(index);
    }

    pub fn rm_set_constraint(&mut self, indexes: Vec<usize>) {
        self.partial_chronicle.rm_set_constraint(indexes)
    }

    pub fn set_pure_result(&mut self, result: Lit) {
        self.result.set_pure(result)
    }
}

impl ExpressionChronicle {
    pub fn get_interval(&self) -> &Interval {
        &self.interval
    }

    pub fn get_result(&self) -> Lit {
        self.result.clone().into()
    }

    pub fn is_result_pure(&self) -> bool {
        self.result.is_pure()
    }

    pub fn get_result_id(&self) -> &AtomId {
        &self.result.get_id()
    }

    pub fn get_constraints(&self) -> &Vec<Constraint> {
        self.partial_chronicle.get_constraints()
    }

    pub fn get_variables(&self) -> &HashSet<AtomId> {
        self.partial_chronicle.get_variables()
    }
}

//Creates a new expression chronicle, declaring an interval and a result variable in the symbol table.
//The LValue is used for debug
impl ExpressionChronicle {
    pub fn new(lv: LValue, st: &mut SymTable) -> Self {
        let interval = st.declare_new_interval();
        let result = st.declare_new_result();
        let mut ec = Self {
            interval,
            result,
            partial_chronicle: Default::default(),
            value: Lit::Exp(vec![]),
            debug: lv,
        };

        ec.add_constraint(Constraint::LEq(interval.start.into(), interval.end.into()));
        ec
    }

    pub fn set_lit(&mut self, lit: Lit) {
        self.value = lit;
    }

    pub fn get_lit(&self) -> Lit {
        self.value.clone()
    }
}

impl ExpressionChronicle {
    pub fn add_var(&mut self, sym_id: &AtomId) {
        self.partial_chronicle.add_var(sym_id);
    }
    pub fn add_interval(&mut self, interval: &Interval) {
        self.partial_chronicle.add_interval(interval);
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.partial_chronicle.add_constraint(constraint);
    }

    pub fn add_condition(&mut self, cond: Condition) {
        self.partial_chronicle.add_condition(cond)
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.partial_chronicle.add_effect(effect)
    }

    pub fn add_subtask(&mut self, sub_task: Expression) {
        self.partial_chronicle.add_subtask(sub_task)
    }
}

impl Absorb for ExpressionChronicle {
    fn absorb(&mut self, other: Self) {
        /*self.add_constraint(Constraint::LT(
            self.get_interval().start.into(),
            other.get_interval().start.into(),
        ));
        self.add_constraint(Constraint::LT(
            other.get_interval().end.into(),
            self.get_interval().end.into(),
        ));*/
        self.partial_chronicle.absorb(other.partial_chronicle);
        self.add_interval(&other.interval);
        self.add_var(&other.result.get_id());
    }
}

impl FormatWithSymTable for ExpressionChronicle {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        let mut s = if self.result.is_pure() {
            format!(
                "{} {}\n",
                self.interval.format_with_sym_table(st),
                self.result
                    .get_pure()
                    .clone()
                    .unwrap()
                    .format_with_sym_table(st),
            )
        } else {
            format!(
                "{} {} <- {}\n",
                self.interval.format_with_sym_table(st),
                st.get_sym(self.result.get_id()),
                self.debug
            )
        };
        s.push_str(
            format!(
                "subchronicle: \n{}",
                self.partial_chronicle.format_with_sym_table(st)
            )
            .as_str(),
        );

        s
    }
}
#[derive(Clone)]
pub struct Condition {
    pub interval: Interval,
    pub constraint: Constraint,
}

impl FormatWithSymTable for Condition {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        format!(
            "{} {}",
            self.interval.format_with_sym_table(st),
            self.constraint.format_with_sym_table(st)
        )
    }
}

/*#[derive(Clone)]
pub struct TransitionInterval {
    interval: Interval,
    persistence: AtomId,
}

impl TransitionInterval {
    pub fn new(interval: Interval, persistence: AtomId) -> Self {
        Self {
            interval,
            persistence,
        }
    }
}

impl FormatWithSymTable for TransitionInterval {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        format!(
            "[{},{},{}]",
            st.get_atom(&st.get_parent(&self.interval.start)).unwrap(),
            st.get_atom(&st.get_parent(&self.interval.end)).unwrap(),
            st.get_atom(&st.get_parent(&self.persistence)).unwrap()
        )
    }
}*/

#[derive(Clone)]
pub struct Transition {
    variable: Lit,
    value: Lit,
}

impl Transition {
    pub fn new(var: Lit, val: Lit) -> Self {
        Self {
            variable: var,
            value: val,
        }
    }
}

impl FormatWithSymTable for Transition {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        format!(
            "{} <- {}",
            self.variable.format_with_sym_table(st),
            self.value.format_with_sym_table(st)
        )
    }
}

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

#[derive(Clone)]
pub enum Lit {
    Atom(AtomId),
    Constraint(Box<Constraint>),
    Exp(Vec<Lit>),
}

impl Default for Lit {
    fn default() -> Self {
        Self::Exp(vec![])
    }
}

impl From<&AtomId> for Lit {
    fn from(s: &AtomId) -> Self {
        Self::Atom(*s)
    }
}

impl From<AtomId> for Lit {
    fn from(s: AtomId) -> Self {
        (&s).into()
    }
}

impl From<&Constraint> for Lit {
    fn from(c: &Constraint) -> Self {
        Self::Constraint(Box::new(c.clone()))
    }
}

impl From<Constraint> for Lit {
    fn from(c: Constraint) -> Self {
        (&c).into()
    }
}

impl<T: Clone + Into<Lit>> From<&Vec<T>> for Lit {
    fn from(v: &Vec<T>) -> Self {
        Lit::Exp(v.iter().map(|e| e.clone().into()).collect())
    }
}

impl<T: Clone + Into<Lit>> From<Vec<T>> for Lit {
    fn from(v: Vec<T>) -> Self {
        (&v).into()
    }
}

pub fn lvalue_to_lit(lv: &LValue, st: &mut SymTable) -> Result<Lit, LError> {
    match lv {
        LValue::List(list) => {
            let mut vec = vec![];
            for e in list {
                vec.push(lvalue_to_lit(e, st)?);
            }
            Ok(vec.into())
        }
        LValue::Map(_) => Err(SpecialError(
            "LValue to lit",
            "Map transformation to lit is not supported yet.".to_string(),
        )),
        LValue::Number(n) => Ok(st.new_number(n.clone()).into()),
        LValue::True => Ok(st.new_bool(true).into()),
        LValue::Nil => Ok(st.new_bool(false).into()),
        lv => Ok(st.declare_new_symbol(lv.to_string(), false).into()),
    }
}

impl FormatWithSymTable for Lit {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        match self {
            Lit::Atom(a) => st.get_sym(a).to_string(),
            Lit::Constraint(c) => c.format_with_sym_table(st),
            Lit::Exp(vec) => {
                let mut str = "(".to_string();
                for (i, e) in vec.iter().enumerate() {
                    if i != 0 {
                        str.push(' ');
                    }
                    str.push_str(e.format_with_sym_table(st).as_str())
                }
                str.push(')');
                str
            }
        }
    }
}

#[derive(Clone)]
pub enum Constraint {
    LEq(Lit, Lit),
    Eq(Lit, Lit),
    Neg(Lit),
    LT(Lit, Lit),
    And(Lit, Lit),
    Or(Lit, Lit),
}

impl FormatWithSymTable for Constraint {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        match self {
            Constraint::Eq(l1, l2) => format!(
                "({} = {})",
                l1.format_with_sym_table(st),
                l2.format_with_sym_table(st)
            ),
            Constraint::Neg(l1) => format!("(! {})", l1.format_with_sym_table(st)),
            Constraint::LT(l1, l2) => format!(
                "({} < {})",
                l1.format_with_sym_table(st),
                l2.format_with_sym_table(st)
            ),
            Constraint::And(l1, l2) => format!(
                "({} && {})",
                l1.format_with_sym_table(st),
                l2.format_with_sym_table(st)
            ),
            Constraint::Or(l1, l2) => format!(
                "({} || {})",
                l1.format_with_sym_table(st),
                l2.format_with_sym_table(st)
            ),
            Constraint::LEq(l1, l2) => {
                format!(
                    "({} <= {})",
                    l1.format_with_sym_table(st),
                    l2.format_with_sym_table(st)
                )
            }
        }
    }
}

#[derive(Clone)]
pub struct Expression {
    pub interval: Interval,
    pub lit: Lit,
}

impl FormatWithSymTable for Expression {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        format!(
            "{} {}",
            self.interval.format_with_sym_table(st),
            self.lit.format_with_sym_table(st)
        )
    }
}

#[derive(Copy, Clone)]
pub struct Interval {
    start: AtomId,
    end: AtomId,
}

impl Interval {
    pub fn new(start: &AtomId, end: &AtomId) -> Self {
        Self {
            start: *start,
            end: *end,
        }
    }
}

impl Interval {
    pub fn start(&self) -> AtomId {
        self.start
    }

    pub fn end(&self) -> AtomId {
        self.end
    }
}

impl FormatWithSymTable for Interval {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        format!("[{},{}]", st.get_sym(&self.start), st.get_sym(&self.end),)
    }
}

#[derive(Default)]
pub struct ConversionContext {
    pub domain: DomainEnv,
    pub env: LEnv,
}

pub struct Problem {}

type Action = Chronicle;
type Method = Chronicle;

#[derive(Default)]
pub struct Domain {
    actions: Vec<Action>,
    tasks: Vec<Lit>,
    methods: Vec<Method>,
}

impl Domain {
    pub fn new(actions: Vec<Action>, tasks: Vec<Lit>, methods: Vec<Method>) -> Self {
        Self {
            actions,
            tasks,
            methods,
        }
    }
}

impl FormatWithSymTable for Domain {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        let mut str = String::new();

        str.push_str("DOMAIN:\n");

        //actions
        str.push_str("ACTIONS: \n");
        for action in &self.actions {
            str.push_str(format!("{}\n", action.format_with_sym_table(st)).as_str());
        }

        //tasks
        str.push_str("TASKS: \n");
        for task in &self.tasks {
            str.push_str(format!("{}\n", task.format_with_sym_table(st)).as_str());
        }

        //methods
        str.push_str("METHODS: \n");
        for method in &self.methods {
            str.push_str(format!("{}\n", method.format_with_sym_table(st)).as_str());
        }

        //str.push_str(format!("FOREST:\n{} \n", st.symbols).as_str());

        str
    }
}
