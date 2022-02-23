use crate::planning::structs::{ChronicleHierarchy, ConversionContext};
use ompas_lisp::core::structs::lenv::LEnv;
use ompas_lisp::core::structs::lerror::LError::{SpecialError, WrongNumberOfArgument, WrongType};
use ompas_lisp::core::structs::lerror::LResult;
use ompas_lisp::core::structs::llambda::LambdaArgs;
use ompas_lisp::core::structs::lvalue::LValue;
use ompas_lisp::core::structs::typelvalue::TypeLValue;
use ompas_lisp::core::{eval, expand, parse};
use ompas_utils::blocking_async;

pub const TRANSFORM_LAMBDA_EXPRESSION: &str = "transform-lambda-expression";

pub fn pre_processing(
    lv: &LValue,
    context: &ConversionContext,
    _ch: &mut ChronicleHierarchy,
) -> LResult {
    let lv = pre_process_transform_lambda(lv, context)?;

    #[allow(clippy::let_and_return)]
    let lv = pre_eval(&lv, context);

    lv
}

pub fn pre_eval(lv: &LValue, _context: &ConversionContext) -> LResult {
    //let mut env = context.env.clone();
    //let plv = eval_static(lv, &mut env)?;
    //Ok(plv.get_lvalue().clone());
    Ok(lv.clone())
}

pub fn pre_process_transform_lambda(lv: &LValue, context: &ConversionContext) -> LResult {
    let mut lv = match transform_lambda_expression(lv, context.env.clone()) {
        Ok(lv) => lv,
        Err(_) => lv.clone(),
    };

    if let LValue::List(list) = &lv {
        let mut result = vec![];
        for lv in list {
            result.push(pre_process_transform_lambda(lv, context)?)
        }

        lv = result.into()
    }

    Ok(lv)
}

pub fn transform_lambda_expression(lv: &LValue, env: LEnv) -> LResult {
    //println!("in transform lambda");

    if let LValue::List(list) = lv {
        if list.is_empty() {
            return Err(WrongNumberOfArgument(
                TRANSFORM_LAMBDA_EXPRESSION,
                lv.clone(),
                0,
                1..std::usize::MAX,
            ));
        }

        let arg = list[0].clone();
        let mut c_env = env.clone();

        let lambda =
            blocking_async!(eval(&expand(&arg, true, &mut c_env).await?, &mut c_env,).await)
                .expect("Error in thread evaluating lambda")?;
        //println!("evaluating is a success");
        if let LValue::Lambda(l) = lambda {
            let mut lisp = "(begin".to_string();

            let args = &list[1..];

            let params = l.get_params();
            let body = l.get_body();

            match params {
                LambdaArgs::Sym(param) => {
                    let arg = if args.len() == 1 {
                        match &args[0] {
                            LValue::Nil => LValue::Nil,
                            _ => vec![args[0].clone()].into(),
                        }
                    } else {
                        args.into()
                    };
                    lisp.push_str(format!("(define {} '{})", param, arg).as_str());
                }
                LambdaArgs::List(params) => {
                    if params.len() != args.len() {
                        return Err(SpecialError(
                            TRANSFORM_LAMBDA_EXPRESSION,
                            format!(
                                "in lambda {}: ",
                                WrongNumberOfArgument(
                                    TRANSFORM_LAMBDA_EXPRESSION,
                                    args.into(),
                                    args.len(),
                                    params.len()..params.len(),
                                )
                            ),
                        ));
                    }
                    for (param, arg) in params.iter().zip(args) {
                        lisp.push_str(format!("(define {} '{})", param, arg).as_str());
                    }
                }
                LambdaArgs::Nil => {
                    if !args.is_empty() {
                        return Err(SpecialError(
                            TRANSFORM_LAMBDA_EXPRESSION,
                            "Lambda was expecting no args.".to_string(),
                        ));
                    }
                }
            };

            lisp.push_str(body.to_string().as_str());
            lisp.push(')');

            let mut c_env = env;

            blocking_async!(parse(&lisp, &mut c_env).await).expect("error in thread parsing string")
        } else {
            Err(WrongType(
                TRANSFORM_LAMBDA_EXPRESSION,
                list[0].clone(),
                (&list[0]).into(),
                TypeLValue::Lambda,
            ))
        }
    } else {
        Err(WrongType(
            TRANSFORM_LAMBDA_EXPRESSION,
            lv.clone(),
            lv.into(),
            TypeLValue::List,
        ))
    }
}
