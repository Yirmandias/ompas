use ompas_middleware::Master;
use sompas_language::HELP;
use sompas_macros::{async_scheme_fn, scheme_fn};
use sompas_structs::documentation::Documentation;
use sompas_structs::kindlvalue::KindLValue;
use sompas_structs::lenv::LEnv;
use sompas_structs::lruntimeerror::{LResult, LRuntimeError};
use sompas_structs::lvalue::{LValue, Sym};
use sompas_structs::string;

/// Returns a list of all the keys present in the environment
#[scheme_fn]
pub fn env_get_keys(env: &LEnv) -> Vec<LValue> {
    env.keys()
        .iter()
        .map(|x| LValue::from(x.clone()))
        .collect::<Vec<LValue>>()
}

/// Return the list of macros present in the environment
#[scheme_fn]
pub fn env_get_macros(env: &LEnv) -> Vec<LValue> {
    env.macros()
        .iter()
        .map(|x| LValue::from(x.clone()))
        .collect::<Vec<LValue>>()
}

/// Return the expression of a given macro
#[scheme_fn]
pub fn env_get_macro(env: &LEnv, m: Sym) -> LValue {
    match env.get_macro(&m).cloned() {
        Some(l) => l.into(),
        None => LValue::Nil,
    }
}

/// Return a list of help elements
/// Takes 0 or 1 parameter.
/// 0 parameter: gives the list of all the functions
/// 1 parameter: write the help of

pub fn help(env: &LEnv, args: &[LValue]) -> LResult {
    let documentation: Documentation = env.get_documentation();

    match args.len() {
        0 => Ok(documentation.get_all().into()),
        1 => match &args[0] {
            LValue::Fn(fun) => Ok(string!(documentation.get(fun.get_label()))),
            LValue::Symbol(s) => Ok(string!(documentation.get(s))),
            LValue::CoreOperator(co) => Ok(string!(documentation.get(&co.to_string()))),
            lv => Err(LRuntimeError::wrong_type(HELP, lv, KindLValue::Symbol)),
        },
        _ => Err(LRuntimeError::wrong_number_of_args(HELP, args, 0..1)),
    }
}

/// Return the list of all modules loaded in the environment
#[scheme_fn]
pub fn get_contexts(env: &LEnv) -> String {
    let list = env.get_contexts_labels();
    let mut str = '{'.to_string();
    for (i, s) in list.iter().enumerate() {
        if i != 0 {
            str.push(',')
        }
        str.push_str(s)
    }

    str.push(')');

    str
}

/// Return the list of processes and process topics along their dependencies
#[async_scheme_fn]
pub async fn get_process_hierarchy() -> String {
    Master::format_process_hierarchy().await
}
