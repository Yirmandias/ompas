use ompas_lisp::structs::LValue;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

type Sym = String;

type SymId = usize;

pub trait Absorb {
    fn absorb(self, other: Self) -> Self;
}

pub trait FormatWithSymTable {
    fn format_with_sym_table(&self, st: &SymTable) -> String;
}

#[derive(Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub enum SymType {
    Timepoint,
    Result,
    Object,
}

impl Display for SymType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SymType::Timepoint => write!(f, "timepoint"),
            SymType::Result => write!(f, "return"),
            SymType::Object => write!(f, "object"),
        }
    }
}

#[derive(Clone)]
pub struct SymTable {
    symbols: Vec<Sym>,
    ids: HashMap<Sym, SymId>,
    symbol_types: HashMap<SymId, SymType>,
    types_number: HashMap<SymType, usize>,
}

impl Default for SymTable {
    fn default() -> Self {
        let mut types_number = HashMap::new();
        types_number.insert(SymType::Result, 0);
        types_number.insert(SymType::Timepoint, 0);
        types_number.insert(SymType::Object, 0);
        Self {
            symbols: vec![],
            ids: Default::default(),
            symbol_types: Default::default(),
            types_number,
        }
    }
}

impl SymTable {
    pub fn get(&self, id: &SymId) -> Option<&Sym> {
        self.symbols.get(*id)
    }

    pub fn id(&self, sym: &Sym) -> Option<&SymId> {
        self.ids.get(sym)
    }

    //Declare a new return value
    //The name of the return value will be format!("r_{}", last_return_index)
    pub fn declare_new_result(&mut self) -> SymId {
        let n = self.types_number.get_mut(&SymType::Result).unwrap();
        let sym = format!("r_{}", n);
        *n = *n + 1 as usize;
        let id = self.symbols.len();
        self.symbols.push(sym.clone());
        self.ids.insert(sym, id);
        self.symbol_types.insert(id, SymType::Result);
        id
    }

    pub fn declare_new_interval(&mut self) -> Interval {
        let n = self.types_number.get_mut(&SymType::Timepoint).unwrap();
        let start = format!("t_{}", n);
        let end = format!("t_{}", *n + 1 as usize);
        *n = *n + 2 as usize;
        let id_1 = self.symbols.len();
        let id_2 = id_1 + 1;
        self.symbols.push(start.clone());
        self.symbols.push(end.clone());
        self.ids.insert(start, id_1);
        self.symbol_types.insert(id_1, SymType::Timepoint);
        self.ids.insert(end, id_2);
        self.symbol_types.insert(id_2, SymType::Timepoint);
        Interval {
            start: id_1,
            end: id_2,
        }
    }

    pub fn declare_new_timepoint(&mut self) -> SymId {
        let n = self.types_number.get_mut(&SymType::Timepoint).unwrap();
        let sym = format!("t_{}", n);
        *n = *n + 1 as usize;
        let id = self.symbols.len();
        self.symbols.push(sym.clone());
        self.ids.insert(sym, id);
        self.symbol_types.insert(id, SymType::Timepoint);
        id
    }

    pub fn declare_new_object(&mut self, obj: Option<Sym>) -> SymId {
        let n = self.types_number.get_mut(&SymType::Object).unwrap();
        let sym = match obj {
            Some(s) => s,
            None => {
                format!("o_{}", n)
            }
        };
        *n = *n + 1 as usize;
        let id = self.symbols.len();
        self.symbols.push(sym.clone());
        self.ids.insert(sym, id);
        self.symbol_types.insert(id, SymType::Object);
        id
    }
}

impl FormatWithSymTable for PartialChronicle {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        let get_sym = |id: &SymId| {
            st.get(id)
                .expect("error in the definition of the symbol_table")
        };

        let mut s = String::new();
        s.push_str("-variable(s): {");
        for (i, id) in self.variables.iter().enumerate() {
            if i != 0 {
                s.push(',');
            }
            s.push_str(get_sym(id).as_str());
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

impl FormatWithSymTable for Chronicle {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        let get_sym = |id: &SymId| {
            st.get(id)
                .expect("error in the definition of the symbol_table")
        };

        let mut s = String::new();
        //name
        s.push_str("-name: ");
        for e in &self.name {
            s.push_str(get_sym(e).as_str());
            s.push(' ');
        }
        s.push('\n');
        //task
        s.push_str("-task: ");
        for e in &self.task {
            s.push_str(get_sym(e).as_str());
            s.push(' ');
        }
        s.push('\n');
        s.push_str(self.partial_chronicle.format_with_sym_table(st).as_str());
        s
    }
}

#[derive(Clone, Default)]
pub struct Chronicle {
    name: Vec<SymId>,
    task: Vec<SymId>,
    partial_chronicle: PartialChronicle,
}

#[derive(Clone, Default)]
pub struct PartialChronicle {
    variables: HashSet<SymId>,
    constraints: Vec<Constraint>,
    conditions: Vec<Condition>,
    effects: Vec<Effect>,
    subtasks: Vec<Expression>,
}

impl Absorb for PartialChronicle {
    fn absorb(mut self, mut other: Self) -> Self {
        self.variables.union(&other.variables);
        self.constraints.append(&mut other.constraints);
        self.conditions.append(&mut other.conditions);
        self.effects.append(&mut other.effects);
        self.subtasks.append(&mut other.subtasks);
        self
    }
}

impl PartialChronicle {
    pub fn add_var(&mut self, sym_id: &SymId) {
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
}

impl Chronicle {
    pub fn add_var(&mut self, sym_id: &SymId) {
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

pub struct ExpressionChronicle {
    interval: Interval,
    result: SymId,
    partial_chronicle: PartialChronicle,
    value: Lit,
    debug: LValue,
}

impl ExpressionChronicle {
    pub fn get_interval(&self) -> &Interval {
        &self.interval
    }

    pub fn get_result(&self) -> &SymId {
        &self.result
    }
}

impl ExpressionChronicle {
    pub fn new(lv: LValue, st: &mut SymTable) -> Self {
        let interval = st.declare_new_interval();
        let result = st.declare_new_result();
        Self {
            interval,
            result,
            partial_chronicle: Default::default(),
            value: Lit::Exp(vec![]),
            debug: lv,
        }
    }

    pub fn set_lit(&mut self, lit: Lit) {
        self.value = lit;
    }

    pub fn get_lit(&mut self) -> Lit {
        self.value.clone()
    }
}

impl ExpressionChronicle {
    pub fn add_var(&mut self, sym_id: &SymId) {
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
    fn absorb(self, other: Self) -> Self {
        let mut p_c = self.partial_chronicle.absorb(other.partial_chronicle);
        p_c.add_interval(&other.interval);
        p_c.add_var(&other.result);
        Self {
            interval: self.interval,
            result: self.result,
            partial_chronicle: p_c,
            value: self.value,
            debug: self.debug,
        }
    }
}

impl FormatWithSymTable for ExpressionChronicle {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        let get_sym = |id: &SymId| {
            st.get(id)
                .expect("error in the definition of the symbol_table")
        };
        let mut s = String::new();

        s.push_str(
            format!(
                "{} {} <- {}\n",
                self.interval.format_with_sym_table(st),
                get_sym(&self.result),
                self.debug
            )
            .as_str(),
        );
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

#[derive(Clone)]
pub struct TransitionInterval {
    interval: Interval,
    persistence: SymId,
}

impl TransitionInterval {
    pub fn new(interval: Interval, persistence: SymId) -> Self {
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
            st.get(&self.interval.start).unwrap(),
            st.get(&self.interval.end).unwrap(),
            st.get(&self.persistence).unwrap()
        )
    }
}

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
    pub interval: TransitionInterval,
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
    Atom(SymId),
    LValue(LValue),
    Constraint(Box<Constraint>),
    Exp(Vec<Lit>),
}

impl From<&SymId> for Lit {
    fn from(s: &SymId) -> Self {
        Self::Atom(*s)
    }
}

impl From<SymId> for Lit {
    fn from(s: SymId) -> Self {
        (&s).into()
    }
}

impl From<&LValue> for Lit {
    fn from(lv: &LValue) -> Self {
        Self::LValue(lv.clone())
    }
}

impl From<LValue> for Lit {
    fn from(lv: LValue) -> Self {
        (&lv).into()
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

impl From<Vec<Lit>> for Lit {
    fn from(vec: Vec<Lit>) -> Self {
        Self::Exp(vec.clone())
    }
}

impl FormatWithSymTable for Lit {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        match self {
            Lit::Atom(a) => st.symbols.get(*a).unwrap().clone(),
            Lit::Constraint(c) => c.format_with_sym_table(st),
            Lit::Exp(vec) => {
                let mut str = "(".to_string();
                for (i, e) in vec.iter().enumerate() {
                    if i != 0 {
                        str.push(' ');
                    }
                    str.push_str(e.format_with_sym_table(&st).as_str())
                }
                str.push(')');
                str
            }
            Lit::LValue(lv) => lv.to_string(),
        }
    }
}

#[derive(Clone)]
pub enum Constraint {
    Eq(Lit, Lit),
    Neg(Lit),
    LT(Lit, Lit),
}

impl FormatWithSymTable for Constraint {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        match self {
            Constraint::Eq(l1, l2) => format!(
                "{} = {}",
                l1.format_with_sym_table(st),
                l2.format_with_sym_table(st)
            ),
            Constraint::Neg(l1) => format!("! {}", l1.format_with_sym_table(st)),
            Constraint::LT(l1, l2) => format!(
                "{} < {}",
                l1.format_with_sym_table(st),
                l2.format_with_sym_table(st)
            ),
        }
    }
}

#[derive(Clone)]
pub struct Expression {
    pub interval: Interval,
    pub lit: Lit,
}

impl Expression {}

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
    start: SymId,
    end: SymId,
}

impl Interval {
    pub fn start(&self) -> SymId {
        self.start
    }

    pub fn end(&self) -> SymId {
        self.end
    }
}

impl FormatWithSymTable for Interval {
    fn format_with_sym_table(&self, st: &SymTable) -> String {
        format!(
            "[{},{}]",
            st.symbols.get(self.start).unwrap(),
            st.symbols.get(self.end).unwrap()
        )
    }
}

pub struct Problem {}

type Action = Chronicle;
type Method = Chronicle;

struct Domain {
    actions: Vec<Action>,
    tasks: Vec<Lit>,
    methods: Vec<Method>,
}
