#![allow(deprecated)]
use crate::functions::*;
use crate::language::scheme_primitives::*;
use crate::language::*;
use crate::structs::LCoreOperator::Quote;
use crate::structs::LError::*;
use crate::structs::NameTypeLValue::{List, Symbol};
use crate::structs::*;
use aries_planning::parsing::sexpr::SExpr;
use async_recursion::async_recursion;
use im::hashmap::HashMap;
use im::HashSet;
use std::any::Any;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

lazy_static! {
    ///Global variable used to enable debug println.
    /// Mainly used during development.
    static ref DEBUG: AtomicBool = AtomicBool::new(false);
}

/// Enables debugging
/// DEBUG <- true
pub fn activate_debug() {
    DEBUG.store(true, Ordering::Relaxed);
}

/// Returns the value of debug
pub fn get_debug() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

/// Structs used to store the Scheme Environment
/// - It contains a mapping of <symbol(String), LValue>
/// - It also contains macros, special LLambdas used to format LValue expressions.
/// - A LEnv can inherits from an outer environment. It can use symbols from it, but not modify them.
#[derive(Clone, Debug)]
pub struct LEnv {
    symbols: im::HashMap<String, LValue>,
    macro_table: im::HashMap<String, LLambda>,
    //pub(crate) new_entries: Vec<String>, Used to export new entries, but not really important in the end
    outer: Option<Box<LEnv>>,
    //task_handler: TaskHandler
}

impl Display for LEnv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = "".to_string();
        for s in &self.symbols {
            str.push_str(format!("{}: {}\n", s.0, s.1).as_str())
        }
        if let Some(outer) = &self.outer {
            str.push_str(outer.to_string().as_str());
        }
        writeln!(f, "{}", str)
    }
}

impl LEnv {
    pub fn merge_by_symbols(&mut self, other: &Self) {
        self.symbols = self.symbols.clone().union(other.symbols.clone());
    }
}

/// Struct Wrapping contexts (modules) for each library.
#[derive(Clone, Debug)]
pub struct ContextCollection {
    inner: Vec<Arc<dyn Any + Send + Sync>>,
    map_label_usize: HashMap<String, usize>,
    reverse_map: HashMap<usize, String>,
}

impl Default for ContextCollection {
    fn default() -> Self {
        Self {
            inner: vec![],
            map_label_usize: Default::default(),
            reverse_map: Default::default(),
        }
    }
}

impl ContextCollection {
    ///Insert a new context
    pub fn insert(&mut self, ctx: Arc<dyn Any + Send + Sync>, label: String) -> usize {
        self.inner.push(ctx);
        let id = self.inner.len() - 1;
        self.map_label_usize.insert(label.clone(), id);
        self.reverse_map.insert(id, label);
        id
    }

    /// Returns a reference to the context with the corresponding id
    pub fn get_context(&self, id: usize) -> &(dyn Any + Send + Sync) {
        match self.inner.get(id) {
            None => panic!("id {} corresponds to no ctx:\n {:?}", id, self),
            Some(some) => some.deref(),
        }
    }

    /// Returns the context corresponding to the label.
    pub fn get_context_with_label(&self, label: &str) -> &dyn Any {
        let id = match self.map_label_usize.get(label) {
            None => panic!("no context with such label"),
            Some(s) => *s,
        };

        self.get_context(id)
    }

    /// Returns a mutable reference to the context with corresponding id
    pub fn get_mut_context(&mut self, id: usize) -> &mut (dyn Any + Send + Sync) {
        match self.inner.get_mut(id) {
            None => panic!("no context with id {}", 1),
            Some(ctx) => match Arc::get_mut(ctx) {
                None => panic!("Could no get mut ref from Arc of mod {}. This is probably because the reference to the context is shared", self.reverse_map.get(&id).unwrap()),
                Some(ctx) => ctx
            }
        }
    }
}
/*
#[derive(Clone)]
pub struct RefLEnv(Rc<LEnv>);

impl Default for RefLEnv {
    fn default() -> Self {
        RefLEnv(Rc::new(LEnv::default()))
    }
}

impl From<LEnv> for RefLEnv {
    fn from(e: LEnv) -> Self {
        RefLEnv(Rc::new(e))
    }
}

impl RefLEnv {
    pub fn clone_from_root(&self) -> LEnv {
        let mut env = self.0.deref().clone();
        let outer = env.outer.clone();
        match outer {
            None => {}
            Some(s) => env.merge_by_symbols(&s.clone().clone_from_root()),
        };
        env.outer = None;
        env.into()
    }
}

impl RefLEnv {
    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.symbols.keys().cloned().collect();
        keys.append(&mut self.macro_table.keys().cloned().collect());
        if let Some(outer) = self.outer.clone() {
            keys.append(&mut outer.keys())
        }
        keys
    }

    pub fn root() -> Self {
        RefLEnv(Rc::new(LEnv::root()))
    }

    pub fn new(env: LEnv) -> Self {
        RefLEnv(Rc::new(env))
    }

    pub fn new_from_outer(outer: RefLEnv) -> Self {
        RefLEnv(Rc::new(LEnv {
            symbols: Default::default(),
            macro_table: Default::default(),
            //new_entries: vec![],
            //outer: Some(outer),
        }))
    }

    pub fn empty() -> Self {
        RefLEnv(Rc::new(LEnv::empty()))
    }
}

impl Deref for RefLEnv {
    type Target = LEnv;

    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

impl DerefMut for RefLEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Rc::get_mut(&mut self.0).unwrap()
    }
}*/

impl PartialEq for LEnv {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Default for LEnv {
    fn default() -> Self {
        Self::empty()
    }
}
#[derive(Default)]
struct CtxRoot(());

impl GetModule for CtxRoot {
    /// Returns all basic functions, macros, and lambdas
    ///
    fn get_module(self) -> Module {
        let mut module = Module {
            ctx: Arc::new(()),
            prelude: vec![],
            raw_lisp: Default::default(),
            label: MOD_ROOT.into(),
        };

        module.add_prelude(DEFINE, LCoreOperator::Define.into());
        module.add_prelude(NIL, LValue::Nil);

        //Core Operators
        module.add_prelude(DEFINE, LCoreOperator::Define.into());
        module.add_prelude(IF, LCoreOperator::If.into());
        module.add_prelude(LAMBDA, LCoreOperator::DefLambda.into());
        module.add_prelude(DEF_MACRO, LCoreOperator::DefMacro.into());
        module.add_prelude(BEGIN, LCoreOperator::Begin.into());
        module.add_prelude(QUASI_QUOTE, LCoreOperator::QuasiQuote.into());
        module.add_prelude(QUOTE, LCoreOperator::Quote.into());
        module.add_prelude(UNQUOTE, LCoreOperator::UnQuote.into());

        module.add_fn_prelude(ENV_GET_KEYS, env_get_keys);
        module.add_fn_prelude(ENV_GET_MACROS, env_get_macros);
        module.add_fn_prelude(ENV_GET_MACRO, env_get_macro);

        //Special entry
        module.add_fn_prelude(GET, get);
        module.add_fn_prelude(SET, set);
        //State is an alias for map

        /*
         * LIST FUNCTIONS
         */
        module.add_fn_prelude(LIST, list);
        module.add_fn_prelude(FIRST, first);
        module.add_fn_prelude(SECOND, second);
        module.add_fn_prelude(THIRD, third);
        module.add_fn_prelude(REST, rest);
        module.add_fn_prelude(CAR, car);
        module.add_fn_prelude(CDR, cdr);
        module.add_fn_prelude(LAST, last);
        module.add_fn_prelude(CONS, cons);
        module.add_fn_prelude(LEN, length);
        module.add_fn_prelude(EMPTY, empty);
        module.add_fn_prelude(GET_LIST, get_list);
        module.add_fn_prelude(SET_LIST, set_list);

        //Map functions
        module.add_fn_prelude(MAP, map);
        module.add_fn_prelude(GET_MAP, get_map);
        module.add_fn_prelude(SET_MAP, set_map);
        module.add_fn_prelude(UNION_MAP, union_map);
        module.add_fn_prelude(REMOVE_MAP, remove_map);
        module.add_fn_prelude(REMOVE_KEY_VALUE_MAP, remove_key_value_map);

        module.add_fn_prelude(NOT, not);
        module.add_fn_prelude(NOT_SHORT, not);

        module.add_fn_prelude(APPEND, append);

        module.add_fn_prelude(MEMBER, member);

        module.add_fn_prelude(REVERSE, reverse);

        module.add_fn_prelude(ADD, add);
        module.add_fn_prelude(SUB, sub);
        module.add_fn_prelude(MUL, mul);
        module.add_fn_prelude(DIV, div);
        module.add_fn_prelude(GT, gt);
        module.add_fn_prelude(GE, ge);
        module.add_fn_prelude(LT, lt);
        module.add_fn_prelude(LE, le);
        module.add_fn_prelude(EQ, eq);

        //predicates
        module.add_fn_prelude(IS_NUMBER, is_number);
        module.add_fn_prelude(IS_INT, is_integer);
        module.add_fn_prelude(IS_FLOAT, is_float);
        module.add_fn_prelude(IS_NIL, is_nil);
        module.add_fn_prelude(IS_NUMBER, is_number);
        module.add_fn_prelude(IS_BOOL, is_bool);
        module.add_fn_prelude(IS_SYMBOL, is_symbol);
        module.add_fn_prelude(IS_STRING, is_string);
        module.add_fn_prelude(IS_FN, is_fn);
        module.add_fn_prelude(IS_MUT_FN, is_mut_fn);
        //module.add_fn_prelude(IS_QUOTE, is_quote);
        module.add_fn_prelude(IS_MAP, is_map);
        module.add_fn_prelude(IS_LIST, is_list);
        module.add_fn_prelude(IS_LAMBDA, is_lambda);

        module.add_fn_prelude(IS_PAIR, is_pair);
        module.add_fn_prelude(IS_EQUAL, is_equal);
        module
    }
}

impl LEnv {
    /// Returns the env with all the basic functions, the ContextCollection with CtxRoot
    /// and InitialLisp containing the definition of macros and lambdas,
    pub async fn root() -> (Self, ContextCollection) {
        // let map = im::hashmap::HashMap::new();
        // map.ins
        let mut env = LEnv::default();
        let mut ctxs = ContextCollection::default();
        import(
            &mut env,
            &mut ctxs,
            CtxRoot::default(),
            ImportType::WithoutPrefix,
        )
        .await
        .expect("error while loading module root");
        (env, ctxs)
    }

    pub fn empty() -> Self {
        LEnv {
            symbols: Default::default(),
            macro_table: Default::default(),
            //new_entries: vec![],
            outer: None,
        }
    }

    pub fn set_outer(&mut self, outer: Self) {
        self.outer = Some(Box::new(outer));
    }

    pub fn get_symbol(&self, s: &str) -> Option<LValue> {
        match self.symbols.get(s) {
            None => match &self.outer {
                None => None,
                Some(outer) => outer.get_symbol(s),
            },
            Some(s) => Some(s.clone()),
        }
    }

    pub fn get_ref_symbol(&self, s: &str) -> Option<&LValue> {
        match self.symbols.get(s) {
            None => match &self.outer {
                None => None,
                Some(outer) => outer.get_ref_symbol(s),
            },
            Some(s) => Some(s),
        }
    }

    pub fn insert(&mut self, key: String, exp: LValue) {
        self.symbols = self.symbols.update(key, exp);
    }

    pub fn update(&self, key: String, exp: LValue) -> Self {
        let mut update = self.clone();
        update.symbols.insert(key, exp);
        update
    }

    pub fn set(&mut self, key: String, exp: LValue) -> Result<(), LError> {
        match self.get_symbol(key.as_str()) {
            None => Err(UndefinedSymbol(SET, key)),
            Some(_) => {
                self.symbols.insert(key, exp);
                Ok(())
            }
        }
    }

    pub fn add_macro(&mut self, key: String, _macro: LLambda) {
        self.macro_table.insert(key, _macro);
    }

    pub fn get_macro(&self, key: &str) -> Option<&LLambda> {
        self.macro_table.get(key)
    }

    pub fn keys(&self) -> HashSet<String> {
        let mut keys: HashSet<String> = self.symbols.keys().cloned().collect();
        keys = keys.union(self.macro_table.keys().cloned().collect());
        if let Some(outer) = &self.outer {
            keys = keys.union(outer.keys());
        }
        keys
    }

    pub fn macros(&self) -> HashSet<String> {
        let mut macros: HashSet<String> = self.macro_table.keys().cloned().collect();
        if let Some(outer) = &self.outer {
            macros = macros.union(outer.macros());
        }
        macros
    }
}

/// Load a library (module) into the environment so it can be used.
/// *ctx* is moved into *ctxs*.
/*pub async fn load_module(
    env: &mut LEnv,
    ctxs: &mut ContextCollection,
    ctx: impl GetModule,
) -> Result<LValue, LError> {
    let mut module = ctx.get_module();
    let id = ctxs.insert(module.ctx, module.label);
    //println!("id: {}", id);
    for (sym, lv) in &mut module.prelude {
        match lv {
            LValue::Fn(fun) => fun.set_index_mod(id),
            LValue::MutFn(fun) => fun.set_index_mod(id),
            LValue::AsyncFn(fun) => fun.set_index_mod(id),
            LValue::AsyncMutFn(fun) => fun.set_index_mod(id),
            _ => {}
        }
        env.insert(sym.to_string(), lv.clone());
    }

    for element in module.raw_lisp.inner() {
        //println!("Adding {} to rae_env", element);
        let lvalue = parse(element, env, ctxs).await?;

        if lvalue != LValue::Nil {
            eval(&lvalue, env, ctxs).await?;
        }
    }
    Ok(Nil)
}*/

#[derive(Debug)]
pub enum ImportType {
    WithPrefix,
    WithoutPrefix,
}

pub async fn import(
    env: &mut LEnv,
    ctxs: &mut ContextCollection,
    ctx: impl GetModule,
    import_type: ImportType,
) -> Result<(), LError> {
    let mut module = ctx.get_module();
    let id = ctxs.insert(module.ctx, module.label.clone());
    //println!("id: {}", id);
    for (sym, lv) in &mut module.prelude {
        match lv {
            LValue::Fn(fun) => fun.set_index_mod(id),
            LValue::MutFn(fun) => fun.set_index_mod(id),
            LValue::AsyncFn(fun) => fun.set_index_mod(id),
            LValue::AsyncMutFn(fun) => fun.set_index_mod(id),
            _ => {}
        }
        match import_type {
            ImportType::WithPrefix => {
                env.insert(format!("{}::{}", module.label, sym.to_string()), lv.clone());
            }
            ImportType::WithoutPrefix => {
                env.insert(sym.to_string(), lv.clone());
            }
        }
    }

    for element in module.raw_lisp.inner() {
        let lvalue = parse(element, env, ctxs).await?;

        if lvalue != LValue::Nil {
            eval(&lvalue, env, ctxs).await?;
        }
    }
    Ok(())
}

/// Parse an str and returns an expanded LValue
pub async fn parse(
    str: &str,
    env: &mut LEnv,
    ctxs: &mut ContextCollection,
) -> Result<LValue, LError> {
    match aries_planning::parsing::sexpr::parse(str) {
        Ok(se) => expand(&parse_into_lvalue(&se), true, env, ctxs).await,
        Err(e) => Err(SpecialError(
            "parse",
            format!("Error in command: {}", e.to_string()),
        )),
    }
}

/// Transform LValue containing short version of quotations:
/// - quote *'*,
/// - quasiquote *`*,
/// - unquote *,*
/// Returns a LValue::List containing the expansion of quotation.
/// # Examples
/// ``` lisp
/// 'x => (quote x)
/// `x => (qusiquote x)
/// ,x => (unquote x)
pub fn _expand_quoting(mut vec: Vec<LValue>) -> LValue {
    let mut i_point = 0;
    while i_point < vec.len() {
        let temp = vec.clone();
        for lv in &temp[i_point..] {
            if let LValue::Symbol(s) = lv {
                let first: char = s.chars().next().unwrap();
                if s.len() == 1 {
                    if first == QUOTE_CHAR || first == QUASI_QUOTE_CHAR || first == UNQUOTE_CHAR {
                        let new_lv: LValue = vec![
                            match first {
                                QUOTE_CHAR => QUOTE.into(),
                                QUASI_QUOTE_CHAR => QUASI_QUOTE.into(),
                                UNQUOTE_CHAR => UNQUOTE.into(),
                                _ => panic!(
                                    "Should be {}, {} or {}.",
                                    QUOTE_CHAR, QUASI_QUOTE_CHAR, UNQUOTE_CHAR
                                ),
                            },
                            vec.remove(i_point + 1),
                        ]
                        .into();
                        vec[i_point] = new_lv;
                        i_point += 1;
                        break;
                    }
                } else if s.starts_with(|c| c == '\'' || c == '`' || c == ',') {
                    let new_symbol: LValue = s.as_str()[1..].into();
                    vec[i_point] = vec![
                        match first {
                            QUOTE_CHAR => QUOTE.into(),
                            QUASI_QUOTE_CHAR => QUASI_QUOTE.into(),
                            UNQUOTE_CHAR => UNQUOTE.into(),
                            _ => panic!(
                                "Should be {}, {} or {}.",
                                QUOTE_CHAR, QUASI_QUOTE_CHAR, UNQUOTE_CHAR
                            ),
                        },
                        new_symbol,
                    ]
                    .into();
                }
            }
            i_point += 1;
        }
    }

    vec.into()
}

/// Transform literals into LValue of types Symbol, Float, Integer or Boolean
pub fn parse_into_lvalue(se: &SExpr) -> LValue {
    match se {
        SExpr::Atom(atom) => {
            return match atom.as_str().parse::<i64>() {
                Ok(int) => LValue::Number(LNumber::Int(int)),
                Err(_) => match atom.as_str().parse::<f64>() {
                    //Test if its a float
                    Ok(float) => LValue::Number(LNumber::Float(float)),
                    Err(_) => match atom.as_str() {
                        //Test if its a Boolean
                        TRUE => {
                            //println!("atom is boolean true");
                            LValue::True
                        }
                        FALSE | NIL => {
                            //println!("atom is boolean false");
                            LValue::Nil
                        }
                        s => {
                            if s.starts_with('\"') && s.ends_with('\"') {
                                LValue::String(s[1..s.len() - 1].to_string())
                            } else {
                                LValue::Symbol(s.to_string())
                            }
                        }
                    },
                },
            };
        }
        SExpr::List(list) => {
            //println!("expression is a list");
            let list_iter = list.iter();
            if list_iter.is_empty() {
                LValue::Nil
            } else {
                let vec: Vec<LValue> = list_iter.map(|x| parse_into_lvalue(x)).collect();
                //Expand possible quotting
                //expand_quoting(vec)
                LValue::List(vec)
            }
        }
    }
}

/// Expand LValues Expressions as Macros
#[async_recursion]
pub async fn expand(
    x: &LValue,
    top_level: bool,
    env: &mut LEnv,
    ctxs: &mut ContextCollection,
) -> Result<LValue, LError> {
    match x {
        LValue::List(list) => {
            if let Ok(co) = LCoreOperator::try_from(&list[0]) {
                match co {
                    LCoreOperator::Define | LCoreOperator::DefMacro => {
                        //eprintln!("expand: define: Ok!");
                        if list.len() < 3 {
                            return Err(WrongNumberOfArgument(
                                "expand",
                                x.clone(),
                                list.len(),
                                3..std::usize::MAX,
                            ));
                        }
                        let def = LCoreOperator::try_from(&list[0])?;
                        let v = &list[1];
                        let body = &list[2..];
                        match v {
                            LValue::List(v_list) => {
                                if v_list.len() >= 2 {
                                    let f = &v_list[0];
                                    let args = &v_list[1..];
                                    let mut new_body = vec![LCoreOperator::DefLambda.into()];
                                    new_body.append(&mut args.to_vec());
                                    new_body.append(&mut body.to_vec());
                                    return expand(
                                        &vec![def.into(), f.clone(), new_body.into()].into(),
                                        top_level,
                                        env,
                                        ctxs,
                                    )
                                    .await;
                                }
                            }
                            LValue::Symbol(sym) => {
                                if list.len() != 3 {
                                    return Err(WrongNumberOfArgument(
                                        "expand",
                                        x.clone(),
                                        list.len(),
                                        3..3,
                                    ));
                                }
                                let exp = expand(&list[2], top_level, env, ctxs).await?;
                                //println!("after expansion: {}", exp);
                                if def == LCoreOperator::DefMacro {
                                    if !top_level {
                                        return Err(SpecialError(
                                            "expand",
                                            format!("{}: defmacro only allowed at top level", x),
                                        ));
                                    }
                                    let proc = eval(&exp, &mut env.clone(), ctxs).await?;
                                    //println!("new macro: {}", proc);
                                    if !matches!(proc, LValue::Lambda(_)) {
                                        return Err(SpecialError(
                                            "expand",
                                            format!("{}: macro must be a procedure", proc),
                                        ));
                                    } else {
                                        env.add_macro(sym.clone(), proc.try_into()?);
                                    }
                                    //println!("macro added");
                                    //Add to macro_table
                                    return Ok(LValue::Nil);
                                }
                                //We add to the list the expanded body
                                return Ok(
                                    vec![LCoreOperator::Define.into(), v.clone(), exp].into()
                                );
                            }
                            _ => {
                                return Err(WrongType(
                                    "expand",
                                    x.clone(),
                                    x.into(),
                                    NameTypeLValue::Symbol,
                                ))
                            }
                        }
                    }
                    LCoreOperator::DefLambda => {
                        if list.len() < 3 {
                            return Err(WrongNumberOfArgument(
                                "expand",
                                x.clone(),
                                list.len(),
                                3..std::usize::MAX,
                            ));
                        }
                        let vars = &list[1];
                        let body = &list[2..];
                        //Verification of the types of the arguments
                        match vars {
                            LValue::List(vars_list) => {
                                for v in vars_list {
                                    if !matches!(v, LValue::Symbol(_)) {
                                        return Err(SpecialError(
                                            "expand",
                                            format!("illegal lambda argument list: {}", x),
                                        ));
                                    }
                                }
                            }
                            LValue::Symbol(_) | LValue::Nil => {}
                            lv => {
                                return Err(NotInListOfExpectedTypes(
                                    "expand",
                                    lv.clone(),
                                    lv.into(),
                                    vec![List, Symbol],
                                ))
                            }
                        }
                        let exp = if body.len() == 1 {
                            body[0].clone()
                        } else {
                            let mut vec = vec![LCoreOperator::Begin.into()];
                            vec.append(&mut body.to_vec());
                            LValue::List(vec)
                        };
                        return Ok(vec![
                            LCoreOperator::DefLambda.into(),
                            vars.clone(),
                            expand(&exp, top_level, env, ctxs).await?,
                        ]
                        .into());
                    }
                    LCoreOperator::If => {
                        let mut list = list.clone();
                        if list.len() == 3 {
                            list.push(LValue::Nil);
                        }
                        if list.len() != 4 {
                            return Err(WrongNumberOfArgument(
                                "expand",
                                (&list).into(),
                                list.len(),
                                4..4,
                            ));
                        }
                        //return map(expand, x)
                        let mut expanded_list = vec![LCoreOperator::If.into()];
                        for x in &list[1..] {
                            expanded_list.push(expand(x, false, env, ctxs).await?)
                        }
                        return Ok(expanded_list.into());
                    }
                    LCoreOperator::Quote => {
                        //println!("expand: quote: Ok!");
                        if list.len() != 2 {
                            return Err(WrongNumberOfArgument(
                                "expand",
                                list.into(),
                                list.len(),
                                2..2,
                            ));
                        }
                        return Ok(vec![LCoreOperator::Quote.into(), list[1].clone()].into());
                    }
                    LCoreOperator::Begin => {
                        return if list.len() == 1 {
                            Ok(LValue::Nil)
                        } else {
                            let mut expanded_list = vec![LCoreOperator::Begin.into()];
                            for x in &list[1..] {
                                expanded_list.push(expand(x, top_level, env, ctxs).await?)
                            }
                            Ok(expanded_list.into())
                        }
                    }
                    LCoreOperator::QuasiQuote => {
                        return if list.len() != 2 {
                            Err(WrongNumberOfArgument(
                                "expand",
                                list.into(),
                                list.len(),
                                2..2,
                            ))
                        } else {
                            /*let expanded = expand_quasi_quote(&list[1], env)?;
                            //println!("{}", expanded);
                            //to expand quasiquote recursively
                            expand(&expanded, top_level, env, ctxs);*/
                            expand(&expand_quasi_quote(&list[1], env)?, top_level, env, ctxs).await
                            //Ok(expanded)
                        };
                    }
                    LCoreOperator::UnQuote => {
                        return Err(SpecialError(
                            "expand",
                            "unquote must be inside a quasiquote expression".to_string(),
                        ))
                    }
                    LCoreOperator::Async => {
                        return if list.len() != 2 {
                            Err(WrongNumberOfArgument(
                                "expand",
                                list.into(),
                                list.len(),
                                2..2,
                            ))
                        } else {
                            let mut expanded = vec![LCoreOperator::Async.into()];
                            expanded.push(expand(&list[1], top_level, env, ctxs).await?);
                            Ok(expanded.into())
                        }
                    }
                    LCoreOperator::Await => {
                        return if list.len() != 2 {
                            Err(WrongNumberOfArgument(
                                "expand",
                                list.into(),
                                list.len(),
                                2..2,
                            ))
                        } else {
                            let mut expanded = vec![LCoreOperator::Await.into()];
                            expanded.push(expand(&list[1], top_level, env, ctxs).await?);
                            Ok(expanded.into())
                        }
                    }
                    LCoreOperator::Eval => {
                        return if list.len() != 2 {
                            Err(WrongNumberOfArgument(
                                "expand",
                                list.into(),
                                list.len(),
                                2..2,
                            ))
                        } else {
                            let mut expanded = vec![LCoreOperator::Eval.into()];
                            expanded.push(expand(&list[1], top_level, env, ctxs).await?);
                            Ok(expanded.into())
                        }
                    }
                    LCoreOperator::Parse => {
                        return if list.len() != 2 {
                            Err(WrongNumberOfArgument(
                                "expand",
                                list.into(),
                                list.len(),
                                2..2,
                            ))
                        } else {
                            let mut expanded = vec![LCoreOperator::Parse.into()];
                            expanded.push(expand(&list[1], top_level, env, ctxs).await?);
                            Ok(expanded.into())
                        }
                    }
                    LCoreOperator::Expand => {
                        return if list.len() != 2 {
                            Err(WrongNumberOfArgument(
                                "expand",
                                list.into(),
                                list.len(),
                                2..2,
                            ))
                        } else {
                            let mut expanded = vec![LCoreOperator::Expand.into()];
                            expanded.push(expand(&list[1], top_level, env, ctxs).await?);
                            Ok(expanded.into())
                        }
                    }
                }
            } else if let LValue::Symbol(sym) = &list[0] {
                match env.get_macro(sym) {
                    None => {}
                    Some(m) => {
                        let expanded =
                            expand(&m.call(&list[1..], env, ctxs).await?, top_level, env, ctxs)
                                .await?;
                        if get_debug() {
                            println!("In expand: macro expanded: {:?}", expanded);
                        }
                        return Ok(expanded);
                    }
                }
            }

            let mut expanded_list: Vec<LValue> = vec![];
            for e in list {
                expanded_list.push(expand(e, false, env, ctxs).await?);
            }

            /*let expanded_list: Vec<LValue> = list
            .iter()
            .map(|x| expand(x, false, env, ctxs))
            .collect::<Result<_, _>>()?;*/
            Ok(expanded_list.into())
        }
        lv => Ok(lv.clone()),
    }
}

/// Expand quasiquote expressions
pub fn expand_quasi_quote(x: &LValue, env: &LEnv) -> Result<LValue, LError> {
    match x {
        LValue::List(list) => {
            if list.is_empty() {
                Ok(LValue::Nil)
                //Ok(vec![Quote.into(), x.clone()].into())
            } else {
                let first = &list[0];
                if let LValue::Symbol(s) = first {
                    if let Ok(co) = LCoreOperator::try_from(s.as_str()) {
                        if co == LCoreOperator::UnQuote {
                            if list.len() != 2 {
                                return Err(WrongNumberOfArgument(
                                    "expand_quasi_quote",
                                    x.clone(),
                                    list.len(),
                                    2..2,
                                ));
                            }
                            return Ok(list[1].clone());
                        }
                    }
                }
                Ok(vec![
                    env.get_symbol(CONS)
                        .expect("problem in the definition of the root env"),
                    expand_quasi_quote(first, env)?,
                    expand_quasi_quote(&list[1..].to_vec().into(), env)?,
                ]
                .into())
            }
        }
        _ => Ok(vec![Quote.into(), x.clone()].into()),
    }
    //Verify if has unquotesplicing here
}

/// Evaluate a LValue
/// Main function of the Scheme Interpreter
#[async_recursion]
pub async fn eval(
    lv: &LValue,
    env: &mut LEnv,
    ctxs: &mut ContextCollection,
) -> Result<LValue, LError> {
    let mut lv = lv.clone();
    let mut temp_env: LEnv;
    let mut env = env;

    let str = format!("{}", lv);

    loop {
        if let LValue::Symbol(s) = &lv {
            let result = match env.get_symbol(s.as_str()) {
                None => lv.clone(),
                Some(lv) => lv,
            };
            if get_debug() {
                println!("{} => {}", str, result)
            }
            return Ok(result);
        } else if let LValue::List(list) = &lv {
            //println!("expression is a list");
            let list = list.as_slice();
            let proc = &list[0];
            let args = &list[1..];
            //assert!(args.len() >= 2, "Checked in expansion");
            if let LValue::CoreOperator(co) = proc {
                match co {
                    LCoreOperator::Define => {
                        match &args[0] {
                            LValue::Symbol(s) => {
                                let exp = eval(&args[1], &mut env, ctxs).await?;
                                env.insert(s.to_string(), exp);
                            }
                            lv => {
                                return Err(WrongType(
                                    "eval",
                                    lv.clone(),
                                    lv.into(),
                                    NameTypeLValue::Symbol,
                                ))
                            }
                        };
                        if get_debug() {
                            println!("{} => {}", str, LValue::Nil);
                        }
                        return Ok(LValue::Nil);
                    }
                    LCoreOperator::DefLambda => {
                        //println!("it is a lambda");
                        let params = match &args[0] {
                            LValue::List(list) => {
                                let mut vec_sym = Vec::new();
                                for val in list {
                                    match val {
                                        LValue::Symbol(s) => vec_sym.push(s.clone()),
                                        lv => {
                                            return Err(WrongType(
                                                "eval",
                                                lv.clone(),
                                                lv.into(),
                                                NameTypeLValue::Symbol,
                                            ))
                                        }
                                    }
                                }
                                vec_sym.into()
                            }
                            LValue::Symbol(s) => s.clone().into(),
                            LValue::Nil => LambdaArgs::Nil,
                            lv => {
                                return Err(NotInListOfExpectedTypes(
                                    "eval",
                                    lv.clone(),
                                    lv.into(),
                                    vec![NameTypeLValue::List, NameTypeLValue::Symbol],
                                ))
                            }
                        };
                        let body = &args[1];
                        let r_lvalue =
                            LValue::Lambda(LLambda::new(params, body.clone(), env.clone()));
                        if get_debug() {
                            println!("{} => {}", str, r_lvalue);
                        }
                        return Ok(r_lvalue);
                    }
                    LCoreOperator::If => {
                        let test = &args[0];
                        let conseq = &args[1];
                        let alt = &args[2];
                        lv = match eval(test, &mut env, ctxs).await? {
                            LValue::True => conseq.clone(),
                            LValue::Nil => alt.clone(),
                            lv => {
                                return Err(WrongType(
                                    "eval",
                                    lv.clone(),
                                    lv.into(),
                                    NameTypeLValue::Bool,
                                ))
                            }
                        };
                    }
                    LCoreOperator::Quote => {
                        if get_debug() {
                            println!("{} => {}", str, &args[0].clone());
                        }
                        return Ok(args[0].clone());
                    }
                    LCoreOperator::Begin => {
                        let firsts = &args[0..args.len() - 1];
                        let last = args.last().unwrap();

                        for e in firsts {
                            eval(e, &mut env, ctxs).await?;
                        }
                        lv = last.clone();
                    }
                    LCoreOperator::QuasiQuote
                    | LCoreOperator::UnQuote
                    | LCoreOperator::DefMacro => return Ok(LValue::Nil),
                    LCoreOperator::Async => {
                        //println!("async evaluation");
                        let lvalue = args[0].clone();
                        let mut new_env = env.clone();
                        let mut ctxs = ctxs.clone();

                        /*let future: LValue =
                        tokio::spawn(
                            async move { eval(&lvalue, &mut new_env, &mut ctxs).await },
                        )
                        .await
                        .unwrap()?;*/

                        let future: LValue =
                            (Box::pin(async move { eval(&lvalue, &mut new_env, &mut ctxs).await })
                                as FutureResult)
                                .into();
                        let future_2 = future.clone();
                        tokio::spawn(async move {
                            #[allow(unused_must_use)]
                            if let LValue::Future(future_2) = future_2 {
                                future_2.await;
                            }
                        });

                        return Ok(future);
                    }
                    LCoreOperator::Await => {
                        //println!("awaiting on async evaluation");
                        let future = eval(&args[0], env, ctxs).await?;

                        return if let LValue::Future(future) = future {
                            future.await
                        } else {
                            Err(WrongType(
                                EVAL,
                                future.clone(),
                                (&future).into(),
                                NameTypeLValue::Future,
                            ))
                        };
                    }
                    LCoreOperator::Eval => {
                        let arg = &args[0];
                        lv = expand(&eval(arg, env, ctxs).await?, true, env, ctxs).await?;
                    }
                    LCoreOperator::Parse => {
                        return if let LValue::String(s) = eval(&args[0], env, ctxs).await? {
                            parse(s.as_str(), env, ctxs).await
                        } else {
                            Err(WrongType(
                                "eval",
                                args[0].clone(),
                                (&args[0]).into(),
                                NameTypeLValue::String,
                            ))
                        }
                    }
                    LCoreOperator::Expand => {
                        let arg = &args[0];
                        return expand(&eval(arg, env, ctxs).await?, true, env, ctxs).await;
                    }
                }
            } else {
                let mut exps: Vec<LValue> = vec![];

                for x in list {
                    exps.push(eval(x, &mut env, ctxs).await?)
                }

                /*let exps = list
                .iter()
                .map(|x| eval(x, &mut env, ctxs).await)
                .collect::<Result<Vec<LValue>, _>>()?;*/
                let proc = &exps[0];
                let args = &exps[1..];
                match proc {
                    LValue::Lambda(l) => {
                        lv = l.get_body();
                        temp_env = l.get_new_env(args, env.clone())?;
                        env = &mut temp_env;
                    }
                    LValue::Fn(fun) => {
                        let ctx: &dyn Any = match fun.get_index_mod() {
                            None => unreachable!("{} should have a module index", fun.debug_label),
                            Some(u) => ctxs.get_context(u),
                        };
                        let r_lvalue = fun.call(args, env, ctx)?;
                        if get_debug() {
                            println!("{} => {}", str, r_lvalue);
                        }
                        return Ok(r_lvalue);
                    }
                    LValue::MutFn(fun) => {
                        return match fun.get_index_mod() {
                            None => unreachable!("{} should have a module index", fun.debug_label),
                            Some(u) => {
                                let r_lvalue = fun.call(args, env, ctxs.get_mut_context(u))?;
                                if get_debug() {
                                    println!("{} => {}", str, r_lvalue);
                                }
                                Ok(r_lvalue)
                            }
                        };
                    }
                    LValue::AsyncFn(fun) => {
                        let ctx: &AsyncLTrait = match fun.get_index_mod() {
                            None => unreachable!("{} should have a module index", fun.debug_label),
                            Some(u) => ctxs.get_context(u),
                        };
                        let r_lvalue = fun.call(args, env, ctx).await?;
                        if get_debug() {
                            println!("{} => {}", str, r_lvalue);
                        }
                        return Ok(r_lvalue);
                    }
                    LValue::AsyncMutFn(fun) => {
                        return match fun.get_index_mod() {
                            None => unreachable!("{} should have a module index", fun.debug_label),
                            Some(u) => {
                                let r_lvalue = fun.call(args, env, ctxs.get_mut_context(u)).await?;
                                if get_debug() {
                                    println!("{} => {}", str, r_lvalue);
                                }
                                Ok(r_lvalue)
                            }
                        };
                    }
                    lv => {
                        return Err(WrongType("eval", lv.clone(), lv.into(), NameTypeLValue::Fn));
                    }
                };
            }
        } else {
            if get_debug() {
                println!("{} => {}", str, lv.clone());
            }
            return Ok(lv);
        }
    }
}

/// Expand a macro without evaluating it
/// Used mainly for debug.
#[allow(unused)]
pub async fn macro_expand(
    args: &[LValue],
    env: &LEnv,
    ctxs: &mut ContextCollection,
) -> Result<LValue, LError> {
    let env = &mut env.clone();
    if args.len() < 2 {
        return Err(WrongNumberOfArgument(
            "eval",
            args.into(),
            args.len(),
            2..std::usize::MAX,
        ));
    }
    if let LValue::Symbol(sym) = &args[0] {
        let _macro = env.get_macro(sym).cloned();
        match _macro {
            None => Err(SpecialError(
                "eval",
                format!("{} is not a defined macro", sym),
            )),
            Some(m) => expand(&m.call(&args[1..], env, ctxs).await?, true, env, ctxs).await,
        }
    } else {
        Err(WrongType(
            "eval",
            args[0].clone(),
            (&args[0]).into(),
            NameTypeLValue::Symbol,
        ))
    }
}
