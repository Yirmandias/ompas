use crate::context::rae_env::{Action, StateFunction};
use crate::context::rae_state::{LState, StateType};
use crate::module::rae_exec::RAE_INSTANCE;
use crate::module::{CtxRae, MOD_RAE};
use ::macro_rules_attribute::macro_rules_attribute;
use ompas_lisp::core::language::*;
use ompas_lisp::core::root_module::list::cons;
use ompas_lisp::core::root_module::predicate::language::*;
use ompas_lisp::core::structs::contextcollection::Context;
use ompas_lisp::core::structs::documentation::Documentation;
use ompas_lisp::core::structs::lenv::LEnv;
use ompas_lisp::core::structs::lerror::LError::{SpecialError, WrongNumberOfArgument, WrongType};
use ompas_lisp::core::structs::lerror::{LError, LResult};
use ompas_lisp::core::structs::lvalue::LValue;
use ompas_lisp::core::structs::lvalues::LValueS;
use ompas_lisp::core::structs::module::{IntoModule, Module};
use ompas_lisp::core::structs::purefonction::PureFonctionCollection;
use ompas_lisp::core::structs::typelvalue::TypeLValue;
use ompas_lisp::core::{eval, expand};
use ompas_utils::dyn_async;
use std::convert::TryInto;

/*
LANGUAGE
 */
pub const MOD_RAE_DESCRIPTION: &str = "rae-description";

pub const GENERATE_TASK: &str = "generate-task";
pub const GENERATE_STATE_FUNCTION: &str = "generate-state-function";
pub const GENERATE_ACTION: &str = "generate-action";
pub const GENERATE_ACTION_MODEL: &str = "generate-action-model";
pub const GENERATE_ACTION_OPERATIONAL_MODEL: &str = "generate-action-operational-model";
pub const GENERATE_METHOD: &str = "generate-method";

pub const RAE_DEF_STATE_FUNCTION: &str = "def-state-function";
pub const RAE_DEF_ACTION: &str = "def-action";
pub const RAE_DEF_ACTION_MODEL: &str = "def-action-model";
pub const RAE_DEF_ACTION_OPERATIONAL_MODEL: &str = "def-action-operational-model";
pub const RAE_DEF_TASK: &str = "def-task";
pub const RAE_DEF_METHOD: &str = "def-method";
pub const RAE_DEF_LAMBDA: &str = "def-lambda";
pub const RAE_DEF_INITIAL_STATE: &str = "def-initial-state";
pub const RAE_ADD_CONSTANT: &str = "add-constant";
pub const RAE_ADD_TYPE: &str = "add-type";
pub const RAE_ADD_OBJECT: &str = "add-object";
pub const RAE_DEF_OBJECTS: &str = "def-objects";
pub const RAE_DEF_TYPES: &str = "def-types";
pub const RAE_DEF_CONSTANTS: &str = "def-constants";

pub const DOC_DEF_STATE_FUNCTION: &str = "Insert a state function in RAE environment.";
pub const DOC_DEF_STATE_FUNCTION_VERBOSE: &str =
    "Example:\n(def-state-function robot.coordinates ?r)";
pub const DOC_DEF_ACTION: &str = "Insert an action in RAE environment.";
pub const DOC_DEF_ACTION_VERBOSE: &str = "Example:\n(def-action pick ?r)";
pub const DOC_DEF_TASK: &str = "Insert a task in RAE environment";
pub const DOC_DEF_TASK_VERBOSE: &str = "Example:\n(def-task t_navigate_to ?r ?x ?y)";
pub const DOC_DEF_METHOD: &str = "Insert a method in RAE environment.";
pub const DOC_DEF_METHOD_VERBOSE: &str =
    "Example:\n(def-method m_navigate_to '((:task t_navigate_to)(:params ?r ?x ?y)(:body (begin\n\
        \t(rae-await (navigate_to ?r ?x ?y))\n\
        \t(rae-await (navigate_to ?r (+ ?x 1) (+ ?y 1)))))))";
pub const DOC_DEF_LAMBDA: &str = "Add a lambda to RAE environment";
pub const DOC_DEF_INITIAL_STATE: &str = "Add initial facts in the state.\
Most of the time it is general knowledge and not initialisation of facts.";

/// Macro used to generate code to define a task in the simplified representation in RAE environment.
pub const MACRO_GENERATE_TASK: &str = "(defmacro generate-task 
    (lambda args
    (let* ((label (car args))
          (p_expr (cdr args))
          (params (car (unzip p_expr))))
         `(list ,label 
            (quote ,p_expr)
            (lambda ,params
                ,(cons 'refine (cons `(quote ,label) params)))))))";

/// Macro used to generate code to define a state function in RAE environment.
pub const MACRO_GENERATE_STATE_FUNCTION: &str = "(defmacro generate-state-function (lambda args
    (let* ((label (car args))
          (p_expr (cdr args))
          (params (car (unzip p_expr)))
          (params 
            (if (null? params)
                nil
                (sublist 
                   params
                   0
                   (- (len params)1)))))
        `(list ,label
            (quote ,p_expr)
            (lambda ,params
                    ,(cons 'rae-get-state-variable (cons `(quote ,label) params)))))))";

/// Macro used to generate code to define an action in RAE environment.
pub const MACRO_GENERATE_ACTION: &str = "(defmacro generate-action
    (lambda args
        (let* ((label (car args))
               (p_expr (cdr args))
               (p_unzip (unzip p_expr))
               (params (car p_unzip))
               (types (cadr p_unzip)))
             `(list ,label
                 (quote ,p_expr)
                 (lambda ,params 
                    ,(cons 'rae-exec-command (cons `(quote ,label) params)))))))";

pub const MACRO_GENERATE_ACTION_MODEL: &str = "
(defmacro generate-action-model
    (lambda (label def)
        (let* ((p_expr (cdar def))
               (p_unzip (unzip p_expr))
               (params (car p_unzip))
               (conds (cadr (get def 1)))
               (effs (cadr (get def 2))))
              `(list ,label (lambda ,params
                    (do
                        ,(gtpc p_expr)
                        ;(check ,conds)
                        ,conds
                        ,effs))))))";

pub const MACRO_GENERATE_ACTION_OPERATIONAL_MODEL: &str =
    "(defmacro generate-action-operational-model
    (lambda (label def)
        (let* ((p_expr (cdar def))
               (body (cadr (get def 1)))
               (p_unzip (unzip p_expr))
               (params (car p_unzip)))

              `(list ,label (lambda ,params
                    (do 
                        ,(gtpc p_expr)
                        ,body))))))";

/// Macro used to generate code to define a method in REA environment.
pub const MACRO_GENERATE_METHOD: &str = "(defmacro generate-method
    (lambda (m_label def)
        (let ((t_label (cadar def))
            (p_expr (cdr (get def 1)))
            (conds (cadr (get def 2)))
            (score (cadr (get def 3)))
            (body (cadr (get def 4))))

            (let* ((p_unzip (unzip p_expr))
                (params (car p_unzip))
                (types (cadr p_unzip)))
    
            `(list ,m_label 
                (quote ,t_label)
                (quote ,p_expr)
                ;lambda for preconditons
                (lambda ,params
                    (do 
                        ,(gtpc p_expr)
                        ,conds))
                (lambda ,params ,score)
                (lambda ,params ,body))))))";

const GENERATE_TYPE_TEST_EXPR: &str = "generate-type-test-expr";
const F_AND_EFFECT: &str = "f-and-effect";
const F_AND_COND: &str = "f-and-cond";

const LAMBDA_GENERATE_TYPE_PRE_CONDITIONS: &str =
    "(define gtpc (lambda (l) (parse (generate-type-test-expr l))))";

const MACRO_AND_COND: &str = "(defmacro and-cond (lambda args (parse (f-and-cond args))))";
const MACRO_AND_EFFECT: &str = "(defmacro and-effect (lambda args (parse (f-and-effect args))))";

#[derive(Default)]
pub struct CtxRaeDescription {}

impl IntoModule for CtxRaeDescription {
    fn into_module(self) -> Module {
        let mut module = Module {
            ctx: Context::new(()),
            prelude: vec![],
            raw_lisp: vec![
                MACRO_GENERATE_TASK,
                MACRO_GENERATE_STATE_FUNCTION,
                MACRO_GENERATE_ACTION,
                MACRO_GENERATE_ACTION_MODEL,
                MACRO_GENERATE_ACTION_OPERATIONAL_MODEL,
                MACRO_GENERATE_METHOD,
                //MACRO_ENUMERATE_PARAMS,
                LAMBDA_GENERATE_TYPE_PRE_CONDITIONS,
                MACRO_AND_COND,
                MACRO_AND_EFFECT,
            ]
            .into(),
            label: MOD_RAE_DESCRIPTION.to_string(),
        };

        module.add_fn_prelude(GENERATE_TYPE_TEST_EXPR, generate_type_test_expr);
        module.add_fn_prelude(F_AND_COND, f_and_cond);
        module.add_fn_prelude(F_AND_EFFECT, f_and_effect);
        module
    }

    fn documentation(&self) -> Documentation {
        Default::default()
    }

    fn pure_fonctions(&self) -> PureFonctionCollection {
        vec![].into()
    }
}

fn f_and_cond(args: &[LValue], _: &LEnv) -> LResult {
    if args.len() != 1 {
        return Err(WrongNumberOfArgument(
            GENERATE_TYPE_TEST_EXPR,
            args.into(),
            args.len(),
            1..1,
        ));
    }

    if let LValue::List(conditions) = &args[0] {
        let mut str = "(do ".to_string();
        for cond in conditions {
            str.push_str(format!("(check {})", cond).as_str());
        }
        str.push(')');
        Ok(LValue::String(str))
    } else if let LValue::Nil = &args[0] {
        Ok(LValue::String("true".to_string()))
    } else {
        Err(WrongType(
            GENERATE_TYPE_TEST_EXPR,
            args[0].clone(),
            (&args[0]).into(),
            TypeLValue::List,
        ))
    }
}

fn f_and_effect(args: &[LValue], _: &LEnv) -> LResult {
    if args.len() != 1 {
        return Err(WrongNumberOfArgument(
            GENERATE_TYPE_TEST_EXPR,
            args.into(),
            args.len(),
            1..1,
        ));
    }

    if let LValue::List(effects) = &args[0] {
        let mut str = "(begin ".to_string();
        for eff in effects {
            str.push_str(format!("{}", eff).as_str());
        }
        str.push(')');
        Ok(LValue::String(str))
    } else if let LValue::Nil = &args[0] {
        Ok(LValue::String("true".to_string()))
    } else {
        Err(WrongType(
            GENERATE_TYPE_TEST_EXPR,
            args[0].clone(),
            (&args[0]).into(),
            TypeLValue::List,
        ))
    }
}
/// Takes as input a p_expr of the form ((p1 p1_type) ... (p_n pn_type))
pub fn generate_type_test_expr(args: &[LValue], _: &LEnv) -> LResult {
    if args.len() != 1 {
        return Err(WrongNumberOfArgument(
            GENERATE_TYPE_TEST_EXPR,
            args.into(),
            args.len(),
            1..1,
        ));
    }

    if let LValue::List(params) = &args[0] {
        let mut str = "(do ".to_string();

        for param in params {
            if let LValue::List(param) = &param {
                if param.len() == 2 {
                    if let LValue::Symbol(par) = &param[0] {
                        if let LValue::Symbol(tpe) = &param[1] {
                            let test = match tpe.as_str() {
                                LIST => {
                                    format!("({} {})", IS_LIST, par)
                                }
                                BOOL => format!("({} {})", IS_BOOL, par),
                                INT => format!("({} {})", IS_INT, par),
                                FLOAT => format!("({} {})", IS_FLOAT, par),
                                NUMBER => format!("({} {})", IS_NUMBER, par),
                                SYMBOL => format!("({} {})", IS_SYMBOL, par),
                                _ => format!("(instance {} {})", par, tpe),
                            };

                            str.push_str(format!("(check {})", test).as_str())
                        } else {
                            return Err(WrongType(
                                GENERATE_TYPE_TEST_EXPR,
                                param[1].clone(),
                                (&param[1]).into(),
                                TypeLValue::Symbol,
                            ));
                        }
                    } else {
                        return Err(WrongType(
                            GENERATE_TYPE_TEST_EXPR,
                            param[0].clone(),
                            (&param[0]).into(),
                            TypeLValue::Symbol,
                        ));
                    }
                } else {
                    return Err(WrongNumberOfArgument(
                        GENERATE_TYPE_TEST_EXPR,
                        param.into(),
                        param.len(),
                        2..2,
                    ));
                }
            } else {
                return Err(WrongType(
                    GENERATE_TYPE_TEST_EXPR,
                    param.clone(),
                    param.into(),
                    TypeLValue::List,
                ));
            }
        }
        str.push(')');

        Ok(LValue::String(str))
    } else if let LValue::Nil = &args[0] {
        Ok(LValue::String("true".to_string()))
    } else {
        Err(WrongType(
            GENERATE_TYPE_TEST_EXPR,
            args[0].clone(),
            (&args[0]).into(),
            TypeLValue::List,
        ))
    }
}

/// Defines a lambda in RAE environment.
#[macro_rules_attribute(dyn_async!)]
pub async fn def_lambda<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    if args.len() != 1 {
        return Err(WrongNumberOfArgument(
            RAE_DEF_LAMBDA,
            args.into(),
            args.len(),
            1..1,
        ));
    }

    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;
    let mut env = ctx.get_rae_env().read().await.env.clone();

    if let LValue::List(list) = &args[0] {
        if let LValue::Symbol(label) = &list[0] {
            let expanded = expand(&list[1], true, &mut env).await?;
            let mut e = LEnv::root().await;
            let result = eval(&expanded, &mut e).await?;
            if let LValue::Lambda(_) = &result {
                ctx.get_rae_env()
                    .write()
                    .await
                    .add_lambda(label.clone(), result);
            }
        }
    }
    Ok(LValue::Nil)
}

/// Defines a state function in RAE environment.
#[macro_rules_attribute(dyn_async!)]
pub async fn def_state_function<'a>(args: &'a [LValue], env: &'a LEnv) -> Result<LValue, LError> {
    if args.is_empty() {
        return Err(WrongNumberOfArgument(
            RAE_DEF_STATE_FUNCTION,
            args.into(),
            args.len(),
            1..1,
        ));
    }

    let lvalue = cons(&[GENERATE_STATE_FUNCTION.into(), args.into()], env)?;
    let mut e = LEnv::root().await;

    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;
    let mut env = ctx.get_rae_env().read().await.env.clone();

    let lvalue = eval(&expand(&lvalue, true, &mut env).await?, &mut e).await?;

    if let LValue::List(list) = &lvalue {
        if list.len() != 3 {
            return Err(WrongNumberOfArgument(
                RAE_DEF_STATE_FUNCTION,
                lvalue.clone(),
                list.len(),
                3..3,
            ));
        } else if let LValue::Symbol(sf_label) = &list[0] {
            if let LValue::List(_) | LValue::Nil = &list[1] {
                if let LValue::Lambda(_) = &list[2] {
                    ctx.get_rae_env().write().await.add_state_function(
                        sf_label.to_string(),
                        StateFunction::new((&list[1]).try_into()?, list[2].clone()),
                    )?;
                } else {
                    return Err(WrongType(
                        RAE_DEF_STATE_FUNCTION,
                        list[2].clone(),
                        list[2].clone().into(),
                        TypeLValue::Lambda,
                    ));
                }
            } else {
                return Err(WrongType(
                    RAE_DEF_STATE_FUNCTION,
                    list[1].clone(),
                    (&list[1]).into(),
                    TypeLValue::List,
                ));
            }
        } else {
            return Err(WrongType(
                RAE_DEF_STATE_FUNCTION,
                list[0].clone(),
                list[0].clone().into(),
                TypeLValue::Symbol,
            ));
        }
    }

    Ok(LValue::Nil)
}

/// Defines an action in RAE environment.
#[macro_rules_attribute(dyn_async!)]
pub async fn def_action_model<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    if args.is_empty() {
        return Err(WrongNumberOfArgument(
            RAE_DEF_ACTION_MODEL,
            args.into(),
            args.len(),
            1..std::usize::MAX,
        ));
    }

    let lvalue = cons(&[GENERATE_ACTION_MODEL.into(), args.into()], env)?;

    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;
    let mut env = ctx.get_rae_env().read().await.env.clone();
    let lvalue: LResult = expand(&lvalue, true, &mut env).await;
    let lvalue = match lvalue {
        Ok(e) => e,
        Err(e) => panic!("{}", e),
    };
    //println!("expanded: {}", lvalue);
    let lvalue = eval(&lvalue, &mut env).await?;
    //println!("evaluated: {}", lvalue);

    if let LValue::List(list) = &lvalue {
        if list.len() != 2 {
            return Err(WrongNumberOfArgument(
                RAE_DEF_ACTION_MODEL,
                lvalue.clone(),
                list.len(),
                2..2,
            ));
        } else if let LValue::Symbol(action_label) = &list[0] {
            if let LValue::Lambda(_) = &list[1] {
                ctx.get_rae_env()
                    .write()
                    .await
                    .add_action_sample_fn(action_label.into(), list[1].clone())?;
            } else {
                return Err(WrongType(
                    RAE_DEF_ACTION_MODEL,
                    list[1].clone(),
                    list[1].clone().into(),
                    TypeLValue::Lambda,
                ));
            }
        } else {
            return Err(WrongType(
                RAE_DEF_ACTION_MODEL,
                list[0].clone(),
                list[0].clone().into(),
                TypeLValue::Symbol,
            ));
        }
    }

    Ok(LValue::Nil)
}

/// Defines an action in RAE environment.
#[macro_rules_attribute(dyn_async!)]
pub async fn def_action_operational_model<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    if args.is_empty() {
        return Err(WrongNumberOfArgument(
            RAE_DEF_ACTION_OPERATIONAL_MODEL,
            args.into(),
            args.len(),
            1..std::usize::MAX,
        ));
    }

    let lvalue = cons(
        &[GENERATE_ACTION_OPERATIONAL_MODEL.into(), args.into()],
        env,
    )?;

    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;
    let mut env = ctx.get_rae_env().read().await.env.clone();

    let lvalue = eval(&expand(&lvalue, true, &mut env).await?, &mut env).await?;

    if let LValue::List(list) = &lvalue {
        if list.len() != 2 {
            return Err(WrongNumberOfArgument(
                RAE_DEF_ACTION_OPERATIONAL_MODEL,
                lvalue.clone(),
                list.len(),
                2..2,
            ));
        } else if let LValue::Symbol(action_label) = &list[0] {
            if let LValue::Lambda(_) = &list[1] {
                ctx.get_rae_env()
                    .write()
                    .await
                    .add_action_sample_fn(action_label.into(), list[1].clone())?;
            } else {
                return Err(WrongType(
                    RAE_DEF_ACTION_OPERATIONAL_MODEL,
                    list[1].clone(),
                    list[1].clone().into(),
                    TypeLValue::Lambda,
                ));
            }
        } else {
            return Err(WrongType(
                RAE_DEF_ACTION_OPERATIONAL_MODEL,
                list[0].clone(),
                list[0].clone().into(),
                TypeLValue::Symbol,
            ));
        }
    }

    Ok(LValue::Nil)
}

/// Defines an action in RAE environment.
#[macro_rules_attribute(dyn_async!)]
pub async fn def_action<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    if args.is_empty() {
        return Err(WrongNumberOfArgument(
            RAE_DEF_ACTION,
            args.into(),
            args.len(),
            1..std::usize::MAX,
        ));
    }

    let lvalue = cons(&[GENERATE_ACTION.into(), args.into()], env)?;
    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;

    let mut env = ctx.get_rae_env().read().await.env.clone();
    let lvalue = eval(&expand(&lvalue, true, &mut env).await?, &mut env).await?;

    if let LValue::List(list) = &lvalue {
        if list.len() != 3 {
            return Err(WrongNumberOfArgument(
                RAE_DEF_ACTION,
                lvalue.clone(),
                list.len(),
                3..3,
            ));
        } else if let LValue::Symbol(action_label) = &list[0] {
            if let LValue::List(_) | LValue::Nil = &list[1] {
                if let LValue::Lambda(_) = &list[2] {
                    ctx.get_rae_env().write().await.add_action(
                        action_label.to_string(),
                        Action::new(
                            action_label,
                            (&list[1]).try_into()?,
                            list[2].clone(),
                            LValue::Nil,
                        ),
                    )?;
                } else {
                    return Err(WrongType(
                        RAE_DEF_ACTION,
                        list[2].clone(),
                        list[2].clone().into(),
                        TypeLValue::Lambda,
                    ));
                }
            } else {
                return Err(WrongType(
                    RAE_DEF_ACTION,
                    list[1].clone(),
                    list[1].clone().into(),
                    TypeLValue::List,
                ));
            }
        } else {
            return Err(WrongType(
                RAE_DEF_ACTION,
                list[0].clone(),
                list[0].clone().into(),
                TypeLValue::Symbol,
            ));
        }
    }

    Ok(LValue::Nil)
}

/// Defines a method in RAE environment.
#[macro_rules_attribute(dyn_async!)]
pub async fn def_method<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    if args.is_empty() {
        return Err(WrongNumberOfArgument(
            RAE_DEF_METHOD,
            args.into(),
            args.len(),
            1..std::usize::MAX,
        ));
    }

    let lvalue = cons(&[GENERATE_METHOD.into(), args.into()], env)?;

    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;

    let mut env = ctx.get_rae_env().read().await.env.clone();

    let lvalue = eval(&expand(&lvalue, true, &mut env).await?, &mut env).await?;

    //println!("lvalue: {}", lvalue);

    if let LValue::List(list) = &lvalue {
        if list.len() != 6 {
            return Err(WrongNumberOfArgument(
                RAE_DEF_METHOD,
                lvalue.clone(),
                list.len(),
                6..6,
            ));
        } else if let LValue::Symbol(method_label) = &list[0] {
            if let LValue::Symbol(task_label) = &list[1] {
                match &list[2] {
                    LValue::List(_) | LValue::Nil => {
                        if let LValue::Lambda(_) = &list[3] {
                            if let LValue::Lambda(_) = &list[4] {
                                if let LValue::Lambda(_) = &list[5] {
                                    ctx.get_rae_env().write().await.add_method(
                                        method_label.to_string(),
                                        task_label.to_string(),
                                        list[2].clone().try_into()?,
                                        list[3].clone(),
                                        list[4].clone(),
                                        list[5].clone(),
                                    )?;
                                } else {
                                    return Err(WrongType(
                                        RAE_DEF_METHOD,
                                        list[5].clone(),
                                        list[5].clone().into(),
                                        TypeLValue::Lambda,
                                    ));
                                }
                            } else {
                                return Err(WrongType(
                                    RAE_DEF_METHOD,
                                    list[4].clone(),
                                    list[4].clone().into(),
                                    TypeLValue::Lambda,
                                ));
                            }
                        } else {
                            return Err(WrongType(
                                RAE_DEF_METHOD,
                                list[3].clone(),
                                list[3].clone().into(),
                                TypeLValue::Lambda,
                            ));
                        }
                    }
                    _ => {
                        return Err(WrongType(
                            RAE_DEF_METHOD,
                            list[2].clone(),
                            list[2].clone().into(),
                            TypeLValue::List,
                        ))
                    }
                }
            } else {
                return Err(WrongType(
                    RAE_DEF_METHOD,
                    list[1].clone(),
                    list[1].clone().into(),
                    TypeLValue::Symbol,
                ));
            }
        } else {
            return Err(WrongType(
                RAE_DEF_METHOD,
                list[0].clone(),
                list[0].clone().into(),
                TypeLValue::Symbol,
            ));
        }
    }

    Ok(LValue::Nil)
}

#[macro_rules_attribute(dyn_async!)]
pub async fn def_task<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    if args.is_empty() {
        return Err(WrongNumberOfArgument(
            RAE_DEF_TASK,
            args.into(),
            args.len(),
            1..std::usize::MAX,
        ));
    }

    let lvalue = cons(&[GENERATE_TASK.into(), args.into()], env)?;

    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;

    let mut env = ctx.get_rae_env().read().await.env.clone();

    let lvalue = eval(&expand(&lvalue, true, &mut env).await?, &mut env).await?;

    //println!("new_task: {}", lvalue);

    if let LValue::List(list) = &lvalue {
        if list.len() != 3 {
            return Err(WrongNumberOfArgument(
                RAE_DEF_TASK,
                lvalue.clone(),
                list.len(),
                3..3,
            ));
        } else if let LValue::Symbol(task_label) = &list[0] {
            if let LValue::Lambda(_) = &list[2] {
                ctx.get_rae_env().write().await.add_task(
                    task_label.to_string(),
                    list[2].clone(),
                    (&list[1]).try_into()?,
                )?;
            } else {
                return Err(WrongType(
                    RAE_DEF_TASK,
                    list[2].clone(),
                    list[2].clone().into(),
                    TypeLValue::Lambda,
                ));
            }
        } else {
            return Err(WrongType(
                RAE_DEF_TASK,
                list[0].clone(),
                list[0].clone().into(),
                TypeLValue::Symbol,
            ));
        }
    } else {
        return Err(WrongType(
            RAE_DEF_TASK,
            lvalue.clone(),
            lvalue.into(),
            TypeLValue::List,
        ));
    }

    Ok(LValue::Nil)
}

///Takes in input a list of initial facts that will be stored in the inner world part of the State.
#[macro_rules_attribute(dyn_async!)]
pub async fn def_initial_state<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    if args.len() != 1 {
        return Err(WrongNumberOfArgument(
            RAE_DEF_INITIAL_STATE,
            args.into(),
            args.len(),
            1..1,
        ));
    }

    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;

    if let LValue::Map(map) = &args[0] {
        let mut inner_world = LState {
            inner: Default::default(),
            _type: Some(StateType::InnerWorld),
        };
        let mut instance = LState {
            inner: Default::default(),
            _type: Some(StateType::Instance),
        };

        for (k, v) in map {
            let mut is_instance = false;
            if let LValue::List(list) = k {
                if list[0] == LValue::from(RAE_INSTANCE) {
                    instance.insert(k.into(), v.into());
                    is_instance = true;
                }
            }
            if !is_instance {
                inner_world.insert(k.into(), v.into());
            }
        }

        ctx.get_rae_env()
            .write()
            .await
            .state
            .update_state(inner_world)
            .await;

        ctx.get_rae_env()
            .write()
            .await
            .state
            .update_state(instance)
            .await;

        Ok(LValue::Nil)
    } else {
        Err(WrongType(
            RAE_DEF_INITIAL_STATE,
            args[0].clone(),
            (&args[0]).into(),
            TypeLValue::Map,
        ))
    }
}
#[macro_rules_attribute(dyn_async!)]
pub async fn def_types<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    for arg in args {
        add_type(&[arg.clone()], env).await?;
    }
    Ok(LValue::Nil)
}
#[macro_rules_attribute(dyn_async!)]
pub async fn def_objects<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    for arg in args {
        let list: Vec<LValue> = arg.try_into()?;
        if list.len() < 2 {
            return Err(SpecialError(
                RAE_DEF_CONSTANTS,
                format!("an objects is defined by a symbol and a type, got {}", arg),
            ));
        }
        let last = list.last().unwrap();
        for obj in &list[0..list.len() - 1] {
            add_object(&[obj.clone(), last.clone()], env).await?;
        }
    }
    Ok(LValue::Nil)
}

/*pub async fn def_objects(args: &[LValue], env: &LEnv) -> LResult {
    def_constants(args, env).await
}*/

#[macro_rules_attribute(dyn_async!)]
pub async fn add_type<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    let ctx = env.get_context::<CtxRae>(MOD_RAE).unwrap();

    let arg: LValueS = (&args[0]).into();

    let mut instance = LState {
        inner: Default::default(),
        _type: Some(StateType::Instance),
    };

    instance.insert(vec![RAE_INSTANCE.into(), arg].into(), LValueS::List(vec![]));

    ctx.get_rae_env()
        .write()
        .await
        .state
        .update_state(instance)
        .await;

    Ok(LValue::Nil)
}

#[macro_rules_attribute(dyn_async!)]
pub async fn add_object<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    if args.len() != 2 {
        return Err(WrongNumberOfArgument(
            RAE_ADD_OBJECT,
            args.into(),
            args.len(),
            2..2,
        ));
    }

    let ctx = env.get_context::<CtxRae>(MOD_RAE).unwrap();

    let constant: LValueS = (&args[0]).into();
    let t: LValueS = (&args[1]).into();

    let mut instances: LState = ctx
        .get_rae_env()
        .read()
        .await
        .state
        .get_state(Some(StateType::Instance))
        .await;
    let key = vec![RAE_INSTANCE.into(), t].into();

    let objects: &mut LValueS = match instances.get_mut(&key) {
        Some(obj) => obj,
        None => {
            return Err(SpecialError(
                RAE_ADD_OBJECT,
                format!("type {} is undefined", args[1]),
            ))
        }
    };

    if let LValueS::List(l) = objects {
        if !l.contains(&constant) {
            l.push(constant)
        } else {
            return Err(SpecialError(
                RAE_ADD_OBJECT,
                format!("{} already defined", constant),
            ));
        }
    }

    instances._type = Some(StateType::Instance);

    ctx.get_rae_env()
        .write()
        .await
        .state
        .set_state(instances)
        .await;

    Ok(LValue::Nil)
}

/*pub async fn add_object(args: &[LValue], env: &LEnv) -> LResult {
    add_constant(args, env).await
}*/

#[cfg(test)]
mod test {
    use crate::module::rae_description::*;
    use crate::module::rae_exec::CtxRaeExec;
    use ompas_lisp::core::structs::lenv::ImportType::WithoutPrefix;
    use ompas_lisp::core::structs::lenv::LEnv;
    use ompas_lisp::core::structs::lerror::LError;
    use ompas_lisp::modules::advanced_math::CtxMath;
    use ompas_lisp::modules::io::CtxIo;
    use ompas_lisp::modules::utils::CtxUtils;
    use ompas_lisp::test_utils::{test_expression_with_env, TestExpression};

    async fn init_env_and_ctxs() -> LEnv {
        let mut env = LEnv::root().await;

        env.import(CtxUtils::default(), WithoutPrefix)
            .await
            .expect("error loading utils");

        env.import(CtxMath::default(), WithoutPrefix)
            .await
            .expect("error loading math");

        env.import(CtxRaeExec::default(), WithoutPrefix)
            .await
            .expect("error loading rae exec");

        env.import(CtxRaeDescription::default(), WithoutPrefix)
            .await
            .expect("error loading rae description");

        env.import(CtxIo::default(), WithoutPrefix)
            .await
            .expect("error loading io");

        env
    }

    #[tokio::test]
    async fn test_macro_generate_task() -> Result<(), LError> {
        let macro_to_test = TestExpression {
            inner: MACRO_GENERATE_TASK,
            dependencies: vec![],
            expression: "(generate-task t_navigate_to (?r robot) (?x int) (?y int))",
            expected: "(list \
                        t_navigate_to
                        '((?r robot) (?x int) (?y int))
                        (lambda (?r ?x ?y)
                            (refine 't_navigate_to ?r ?x ?y)))",
            result: "(list \
                        t_navigate_to
                        '((?r robot) (?x int) (?y int))
                        (lambda (?r ?x ?y)
                            (refine 't_navigate_to ?r ?x ?y)))",
        };

        let mut env = init_env_and_ctxs().await;
        test_expression_with_env(macro_to_test, &mut env, true).await
    }

    #[tokio::test]
    async fn test_macro_generate_state_function() -> Result<(), LError> {
        let macro_to_test = TestExpression {
            inner: MACRO_GENERATE_STATE_FUNCTION,
            dependencies: vec![],
            expression: "(generate-state-function sf (?a object) (?b object) (?c object))",
            expected: "(list sf
                            '((?a object) (?b object) (?c object))
                            (lambda (?a ?b)
                                (rae-get-state-variable 'sf ?a ?b)))",
            result: "(list sf
                            '((?a object) (?b object) (?c object))
                            (lambda (?a ?b)
                                (rae-get-state-variable 'sf ?a ?b)))",
        };

        let mut env = init_env_and_ctxs().await;
        test_expression_with_env(macro_to_test, &mut env, true).await?;

        let macro_to_test_2 = TestExpression {
            inner: MACRO_GENERATE_STATE_FUNCTION,
            dependencies: vec![],
            expression: "(generate-state-function sf)",
            expected: "(list sf
                            'nil
                            (lambda nil
                                (rae-get-state-variable 'sf)))",
            result: "(list sf
                            'nil
                            (lambda nil
                                (rae-get-state-variable 'sf)))",
        };
        test_expression_with_env(macro_to_test_2, &mut env, true).await
    }

    #[tokio::test]
    async fn test_macro_generate_action() -> Result<(), LError> {
        let macro_to_test = TestExpression {
            inner: MACRO_GENERATE_ACTION,
            dependencies: vec![],
            expression: "(generate-action pick_package (?r robot) (?p package))",
            expected: "(list pick_package
                            '((?r robot) (?p package))
                            (lambda (?r ?p)
                                (rae-exec-command (quote pick_package) ?r ?p)))",
            result: "(list pick_package
                            '((?r robot) (?p package))
                            (lambda (?r ?p)
                                (rae-exec-command (quote pick_package) ?r ?p)))",
        };

        let mut env = init_env_and_ctxs().await;
        test_expression_with_env(macro_to_test, &mut env, true).await
    }

    #[tokio::test]
    async fn test_macro_generate_action_model() -> Result<(), LError> {
        let macro_to_test = TestExpression {
            inner: MACRO_GENERATE_ACTION_MODEL,
            dependencies: vec![],
            expression: "(generate-action-model pick
                ((:params (?r robot))
                  (:pre-conditions (check (> (robot.battery ?r) 0.4)))
                  (:effects
                        (assert (robot.busy ?r) true))))",
            expected: "(list pick
                            (lambda (?r)
                                (do
                                    (do
                                        (check (instance ?r robot)))
                                    
                                        (check (> (robot.battery ?r) 0.4))
                                  (assert (robot.busy ?r) true))))",
            result: "(list pick
                            (lambda (?r)
                                (do
                                    (do
                                        (check (instance ?r robot)))
                                    
                                        (check (> (robot.battery ?r) 0.4))
                                  (assert (robot.busy ?r) true))))",
        };

        let mut env = init_env_and_ctxs().await;
        test_expression_with_env(macro_to_test, &mut env, true).await
    }

    #[tokio::test]
    async fn test_macro_generate_action_operational_model() -> Result<(), LError> {
        let macro_to_test = TestExpression {
            inner: MACRO_GENERATE_ACTION_OPERATIONAL_MODEL,
            dependencies: vec![],
            expression: "(generate-action-operational-model place
                        ((:params (?r robot))
                          (:body
                            (if (> (robot.battery ?r) 0.4)
                                (assert (robot.busy ?r) false)
                                (err 0)))))",
            expected: "(list place
                            (lambda (?r)
                                (do
                                    (do
                                        (check (instance ?r robot)))
                                    (if (> (robot.battery ?r) 0.4)
                                        (assert (robot.busy ?r) false)
                                        (err 0)))))",
            result: "(list place
                            (lambda (?r)
                                (do
                                    (do
                                        (check (instance ?r robot)))
                                    (if (> (robot.battery ?r) 0.4)
                                        (assert (robot.busy ?r) false)
                                        (err 0)))))",
        };

        let mut env = init_env_and_ctxs().await;
        test_expression_with_env(macro_to_test, &mut env, true).await
    }

    #[tokio::test]
    async fn test_lambda_generate_type_pre_conditions() -> Result<(), LError> {
        let lambda_test = TestExpression {
            inner: LAMBDA_GENERATE_TYPE_PRE_CONDITIONS,
            dependencies: vec![],
            expression:
                "(gtpc '((?r robot) (?f float ) (?i int) (?b bool) (?s symbol) (?n number) (?l tlist)))",
            expected: "(gtpc '((?r robot) (?f float ) (?i int) (?b bool) (?s symbol) (?n number) (?l tlist)))",
            result: "(do 
                        (check (instance ?r robot))
                        (check (float? ?f))
                        (check (int? ?i))
                        (check (bool? ?b))
                        (check (symbol? ?s))
                        (check (number? ?n))
                        (check(list? ?l)))",
        };
        let mut env = init_env_and_ctxs().await;

        test_expression_with_env(lambda_test, &mut env, false).await
    }

    #[tokio::test]
    async fn test_macro_generate_method() -> Result<(), LError> {
        let macro_to_test = TestExpression {
            inner: MACRO_GENERATE_METHOD,
            dependencies: vec![LAMBDA_GENERATE_TYPE_PRE_CONDITIONS],
            expression: "(generate-method m_navigate_to ((:task t_navigate_to)
            (:params (?r robot) (?x float) (?y float))
            (:pre-conditions (and-cond (robot.available ?r) (< ?x 10) (< ?y 10)))
            (:score 0)
            (:body
            (begin
                (navigate_to ?r ?x ?y)))))",
            expected: "(list m_navigate_to
    't_navigate_to
    '((?r robot) (?x float) (?y float))
    (lambda (?r ?x ?y)
    (do
        (do 
            (check (instance ?r robot))
            (check (float? ?x))
            (check (float? ?y)))
        (do 
            (check (robot.available ?r))
            (check (< ?x 10))
            (check (< ?y 10))))) 
    (lambda (?r ?x ?y) 0 )
    (lambda (?r ?x ?y)
        (begin
            (navigate_to ?r ?x ?y))))",
            result: "(list m_navigate_to
    't_navigate_to
    '((?r robot) (?x float) (?y float))
    (lambda (?r ?x ?y)
    (do
        (do 
            (check (instance ?r robot))
            (check (float? ?x))
            (check (float? ?y)))
        (do 
            (check (robot.available ?r))
            (check (< ?x 10))
            (check (< ?y 10))))) 
    (lambda (?r ?x ?y) 0 )
    (lambda (?r ?x ?y)
        (begin
            (navigate_to ?r ?x ?y))))",
        };

        let mut env = init_env_and_ctxs().await;
        test_expression_with_env(macro_to_test, &mut env, true).await
    }
}
