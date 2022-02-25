use crate::module::rae_exec::{
    RAE_ASSERT, RAE_ASSERT_SHORT, RAE_INSTANCE, RAE_RETRACT, RAE_RETRACT_SHORT,
};
use crate::planning::conversion::post_processing::post_processing;
use crate::planning::structs::atom::{AtomType, Sym};
use crate::planning::structs::chronicle::{Chronicle, ExpressionChronicle};
use crate::planning::structs::condition::Condition;
use crate::planning::structs::constraint::Constraint;
use crate::planning::structs::effect::Effect;
use crate::planning::structs::expression::Expression;
use crate::planning::structs::interval::Interval;
use crate::planning::structs::lit::{lvalue_to_lit, Lit};
use crate::planning::structs::symbol_table::{AtomId, ExpressionType};
use crate::planning::structs::traits::{Absorb, FormatWithSymTable, GetVariables};
use crate::planning::structs::transition::Transition;
use crate::planning::structs::{ChronicleHierarchy, ConversionContext, TaskType};
use ompas_lisp::core::structs::lcoreoperator::language::EVAL;
use ompas_lisp::core::structs::lcoreoperator::LCoreOperator;
use ompas_lisp::core::structs::lerror::LError;
use ompas_lisp::core::structs::lerror::LError::{SpecialError, WrongNumberOfArgument};
use ompas_lisp::core::structs::lvalue::LValue;
use ompas_lisp::core::structs::typelvalue::TypeLValue;
use ompas_lisp::static_eval::{eval_static, parse_static};
use std::convert::{TryFrom, TryInto};

//Names of the functions

//const PRE_PROCESSING: &str = "pre_processing";
const CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE: &str = "convert_lvalue_to_expression_chronicle";

pub fn convert_lvalue_to_expression_chronicle(
    exp: &LValue,
    context: &ConversionContext,
    ch: &mut ChronicleHierarchy,
) -> Result<ExpressionChronicle, LError> {
    let mut ec = ExpressionChronicle::new(exp.clone(), &mut ch.sym_table);

    match exp {
        LValue::Symbol(s) => {
            //Generale case
            let symbol = ch.sym_table.declare_new_symbol(s, false, false);
            if ch.sym_table.get_type(&symbol).unwrap() == &AtomType::Variable {
                ec.add_var(&symbol);
            }
            ec.set_pure_result(symbol.into());
        }
        LValue::Nil
        | LValue::True
        | LValue::Number(_)
        | LValue::String(_)
        | LValue::Character(_) => {
            ec.set_pure_result(lvalue_to_lit(exp, &mut ch.sym_table)?);
        }
        LValue::List(l) => match &l[0] {
            LValue::CoreOperator(co) => match co {
                LCoreOperator::Define => {
                    //Todo : handle the case when the first expression is not a symbol, but an expression that must be evaluated
                    if let LValue::Symbol(s) = &l[1] {
                        let var = ch.sym_table.declare_new_symbol(s, true, false);
                        let val = convert_lvalue_to_expression_chronicle(&l[2], context, ch)?;
                        if val.is_result_pure() {
                            ec.add_constraint(Constraint::Eq(
                                ec.get_interval().start().into(),
                                ec.get_interval().end().into(),
                            ))
                        }
                        ec.add_constraint(Constraint::Eq(val.get_result(), var.into()));
                        ec.set_pure_result(ch.sym_table.new_bool(false).into());
                        ec.absorb(val);
                    } else {
                        return Err(SpecialError(
                            CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE,
                            format!(
                                "Define should take as first argument a symbol and {} is not.",
                                l[1]
                            ),
                        ));
                    }
                }
                LCoreOperator::Begin | LCoreOperator::Do => {
                    ch.sym_table.new_scope();
                    let mut literal: Vec<Lit> = vec![ch
                        .sym_table
                        .id(co.to_string().as_str())
                        .unwrap_or_else(|| panic!("{} is not defined in the symbol table", co))
                        .into()];
                    let mut previous_interval: Interval = *ec.get_interval();

                    for (i, e) in l[1..].iter().enumerate() {
                        let ec_i = convert_lvalue_to_expression_chronicle(e, context, ch)?;

                        literal.push(ec_i.get_result());
                        if i == 0 {
                            ec.add_constraint(Constraint::Eq(
                                previous_interval.start().into(),
                                ec_i.get_interval().start().into(),
                            ));
                        } else {
                            ec.add_constraint(Constraint::Eq(
                                previous_interval.end().into(),
                                ec_i.get_interval().start().into(),
                            ))
                        }

                        previous_interval = *ec_i.get_interval();

                        if i == l.len() - 2 {
                            if ec_i.is_result_pure() {
                                ec.set_pure_result(ec_i.get_result())
                            } else {
                                ec.add_effect(Effect {
                                    interval: *ec.get_interval(),
                                    transition: Transition::new(ec.get_result(), ec_i.get_result()),
                                });
                            }
                        }
                        ec.absorb(ec_i);
                    }

                    ec.add_constraint(Constraint::Eq(
                        previous_interval.end().into(),
                        ec.get_interval().end().into(),
                    ));

                    ch.sym_table.revert_scope();
                }
                LCoreOperator::If => {
                    return convert_if(exp, context, ch);
                }
                LCoreOperator::Quote => {
                    ec.set_pure_result(lvalue_to_lit(&l[1], &mut ch.sym_table)?);
                }
                co => {
                    return Err(SpecialError(
                        CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE,
                        format!("{} not supported yet\nexp : {}", co, exp),
                    ))
                }
            },
            _ => {
                let mut expression_type = ExpressionType::Lisp;

                ch.sym_table.new_scope();
                let mut literal: Vec<Lit> = vec![];

                match &l[0] {
                    LValue::Symbol(_) | LValue::Fn(_) => {
                        let s = l[0].to_string();
                        match s.as_str() {
                            RAE_ASSERT | RAE_ASSERT_SHORT => {
                                if l.len() != 3 {
                                    return Err(WrongNumberOfArgument(
                                        CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE,
                                        exp.clone(),
                                        l.len(),
                                        3..3,
                                    ));
                                }

                                let state_variable =
                                    convert_lvalue_to_expression_chronicle(&l[1], context, ch)?;
                                let value =
                                    convert_lvalue_to_expression_chronicle(&l[2], context, ch)?;

                                ec.add_effect(Effect {
                                    interval: *ec.get_interval(),
                                    transition: Transition::new(
                                        state_variable.get_result(),
                                        value.get_result(),
                                    ),
                                });

                                ec.absorb(state_variable);
                                ec.absorb(value);

                                ec.set_pure_result(ch.sym_table.new_bool(false).into());

                                return Ok(ec);
                            }
                            RAE_INSTANCE => {
                                expression_type = ExpressionType::StateFunction;
                                literal.push(
                                    ch.sym_table
                                        .id(RAE_INSTANCE)
                                        .unwrap_or_else(|| {
                                            panic!("{} is undefined in symbol table", RAE_INSTANCE)
                                        })
                                        .into(),
                                )
                            }
                            RAE_RETRACT | RAE_RETRACT_SHORT => {
                                return Err(SpecialError(
                                    CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE,
                                    "not yet supported".to_string(),
                                ))
                            }
                            _ => {
                                if let Some(id) = ch.sym_table.id(&s) {
                                    match ch.sym_table
                                        .get_type(id)
                                        .expect("a defined symbol should have a type")
                                    {
                                        AtomType::Action => {
                                            expression_type = ExpressionType::Action;
                                            println!("{} is an action", s);
                                        }
                                        AtomType::Function => {}
                                        AtomType::Method => return Err(SpecialError(CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE, format!("{} is method and can not be directly called into the body of a method.\
                                \nPlease call the task that use the method instead", s))),
                                        AtomType::StateFunction => {
                                            expression_type = ExpressionType::StateFunction
                                        }
                                        AtomType::Task => {
                                            expression_type = ExpressionType::Task
                                        }
                                        _ => return Err(SpecialError(CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE, format!("{}: first symbol should be a function, task, action or state function", s))),
                                    }
                                    literal.push(id.into())
                                } else {
                                    return Err(SpecialError(
                                        CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE,
                                        format!("function {} is not defined", s),
                                    ));
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(SpecialError(
                            CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE,
                            format!("{} is not yet supported", TypeLValue::from(&l[0])),
                        ))
                    }
                }

                let mut sub_expression_pure = true;

                let mut previous_interval = *ec.get_interval();
                for (i, e) in l[1..].iter().enumerate() {
                    let ec_i = convert_lvalue_to_expression_chronicle(e, context, ch)?;

                    literal.push(ec_i.get_result());
                    sub_expression_pure &= ec_i.is_result_pure();
                    if i != 0 {
                        ec.add_constraint(Constraint::Eq(
                            previous_interval.end().into(),
                            ec_i.get_interval().start().into(),
                        ));
                    } else {
                        ec.add_constraint(Constraint::Eq(
                            previous_interval.start().into(),
                            ec_i.get_interval().start().into(),
                        ));
                    }

                    previous_interval = *ec_i.get_interval();
                    ec.absorb(ec_i);
                }

                ec.add_constraint(Constraint::LEq(
                    previous_interval.end().into(),
                    ec.get_interval().end().into(),
                ));

                if sub_expression_pure && expression_type == ExpressionType::Lisp {
                    expression_type = ExpressionType::Pure;
                }

                match expression_type {
                    ExpressionType::Pure => {
                        let mut env = context.env.clone();
                        let mut is_pure = false;

                        let mut string = "(".to_string();
                        for (i, element) in literal.iter().enumerate() {
                            if i == 0 {
                                string.push_str(
                                    element.format_with_sym_table(&ch.sym_table).as_str(),
                                );
                                string.push(' ');
                            } else {
                                string.push_str(
                                    format!(
                                        "(quote {})",
                                        element.format_with_sym_table(&ch.sym_table)
                                    )
                                    .as_str(),
                                );
                            }
                        }
                        string.push(')');

                        //println!("expression to be evaluated : {}", string);

                        let result = parse_static(string.as_str(), &mut env);
                        if let Ok(result) = result {
                            if result.is_pure() {
                                //println!("parsing of result is pure");
                                let result = eval_static(result.get_lvalue(), &mut env);

                                /*match result {
                                    Ok(result) => {
                                        if result.is_pure() {
                                            ec.set_pure_result(lvalue_to_lit(
                                                result.get_lvalue(),
                                                &mut ch.sym_table,
                                            )?);
                                            is_pure = true;
                                            /*println!(
                                                "eval static is a success! result is: {}",
                                                result.get_lvalue()
                                            );*/
                                        } else {
                                            //println!("result is not pure: ");
                                        }
                                    }
                                    Err(_) => {
                                        //println!("Error in static evaluation: {}", e);
                                    }
                                }*/

                                if let Ok(result) = result {
                                    if result.is_pure() {
                                        ec.set_pure_result(lvalue_to_lit(
                                            result.get_lvalue(),
                                            &mut ch.sym_table,
                                        )?);
                                        is_pure = true;
                                    }
                                }
                            }
                        } else {
                        }
                        if !is_pure {
                            let literal: Lit = vec![
                                ch.sym_table
                                    .id(EVAL)
                                    .expect("Eval not defined in symbol table")
                                    .into(),
                                Lit::from(literal),
                            ]
                            .into();

                            ec.add_effect(Effect {
                                interval: *ec.get_interval(),
                                transition: Transition::new(ec.get_result(), literal),
                            });
                        }
                    }
                    ExpressionType::Lisp => {
                        let literal: Lit = vec![
                            ch.sym_table
                                .id(EVAL)
                                .expect("Eval not defined in symbol table")
                                .into(),
                            Lit::from(literal),
                        ]
                        .into();

                        ec.add_effect(Effect {
                            interval: *ec.get_interval(),
                            transition: Transition::new(ec.get_result(), literal),
                        });
                    }
                    ExpressionType::Action | ExpressionType::Task => {
                        literal.push(ec.get_result());
                        ec.add_subtask(Expression {
                            interval: *ec.get_interval(),
                            lit: literal.into(),
                        })
                    }
                    ExpressionType::StateFunction => {
                        ec.add_effect(Effect {
                            interval: *ec.get_interval(),
                            transition: Transition::new(ec.get_result(), literal.into()),
                        });
                    }
                };
                ch.sym_table.revert_scope();
            }
        },
        lv => {
            return Err(SpecialError(
                CONVERT_LVALUE_TO_EXPRESSION_CHRONICLE,
                format!(
                    "{} not supported yet\n exp: {}",
                    TypeLValue::from(lv),
                    exp.format(" exp: ".len())
                ),
            ))
        }
    }

    Ok(ec)
}

pub fn convert_if(
    exp: &LValue,
    context: &ConversionContext,
    ch: &mut ChronicleHierarchy,
) -> Result<ExpressionChronicle, LError> {
    ch.sym_table.new_scope();

    let task_label = ch.local_tasks.new_label(TaskType::IfTask);
    //println!("task_label: {}", task_label);
    //println!("({}) expression: \n{}", task_label, exp.format(0));

    let mut ec: ExpressionChronicle = ExpressionChronicle::new(exp.clone(), &mut ch.sym_table);

    let exp: Vec<LValue> = exp
        .try_into()
        .expect("could not transform expression into list");
    let cond = &exp[1];
    let b_true = &exp[2];
    let b_false = &exp[3];

    let ec_cond = convert_lvalue_to_expression_chronicle(cond, context, ch)?;
    let ec_b_true = convert_lvalue_to_expression_chronicle(b_true, context, ch)?;
    let ec_b_false = convert_lvalue_to_expression_chronicle(b_false, context, ch)?;

    let variables_b_true = ec_b_true.get_variables_of_type(&ch.sym_table, &AtomType::Variable);

    let variables_b_false = ec_b_false.get_variables_of_type(&ch.sym_table, &AtomType::Variable);

    let union = variables_b_true.clone().union(variables_b_false.clone());

    let mut union_string: Vec<String> = vec![];
    for v in &union {
        union_string.push(
            Sym::try_from(ch.sym_table.get_atom(v).unwrap())?
                .get_string()
                .clone(),
        )
    }

    /*let complement = variables_b_false
        .clone()
        .relative_complement(variables_b_true.clone());
    let variables_b_true: Vec<AtomId> = variables_b_true.iter().cloned().collect();
    let complement: Vec<AtomId> = complement.iter().cloned().collect();*/

    //All used variables in methods of the task

    /*let mut variables: Vec<AtomId> = variables_b_true.clone();
    variables.append(&mut complement.clone());
    let mut variable_string: Vec<String> = vec![];
    for v in &variables {
        variable_string.push(
            Sym::try_from(ch.sym_table.get_atom(v).unwrap())?
                .get_string()
                .clone(),
        )
    }*/

    /*println!(
        "({}) variables: {:#?}\n union : {:#?}",
        task_label, variable_string, union_string
    );*/

    //CREATION OF THE TASK

    let cond_label = format!("{}_cond", task_label);
    let return_label = format!("{}_r", task_label);

    let mut task_string = vec![task_label.clone()];
    task_string.push(cond_label.clone());
    task_string.push(return_label.clone());
    task_string.append(&mut union_string);

    //println! {"({}) task string: {:#?}", task_label, task_string};

    let mut task_lit: Vec<Lit> = vec![];
    for (i, s) in task_string.iter().enumerate() {
        if i == 0 {
            task_lit.push(ch.sym_table.declare_new_symbol(s, false, false).into());
        } else {
            task_lit.push(ch.sym_table.declare_new_symbol(s, false, true).into());
        }
    }

    //println!("({}) subtask lit: {:#?}", task_label, task_lit);

    let sub_task_interval = ch.sym_table.declare_new_interval();

    ec.add_constraint(Constraint::LEq(
        sub_task_interval.start().into(),
        sub_task_interval.end().into(),
    ));

    /* Temporal constraints between expression that computes the condition
    And the task to execute. */
    ec.add_constraint(Constraint::Eq(
        ec.get_interval().start().into(),
        ec_cond.get_interval().start().into(),
    ));

    ec.add_constraint(Constraint::Eq(
        ec_cond.get_interval().end().into(),
        sub_task_interval.start().into(),
    ));

    ec.add_constraint(Constraint::Eq(
        ec.get_interval().end().into(),
        sub_task_interval.end().into(),
    ));

    /* Bindings between result of the if and the result of the task*/
    ec.add_constraint(Constraint::Eq(
        ch.sym_table.id(&return_label).unwrap().into(),
        ec.get_result(),
    ));

    /* Binding between result of condition and parameter of the task*/
    ec.add_constraint(Constraint::Eq(
        ch.sym_table.id(&cond_label).unwrap().into(),
        ec_cond.get_result(),
    ));
    ec.absorb(ec_cond);
    ec.add_subtask(Expression {
        interval: sub_task_interval,
        lit: task_lit.into(),
    });
    ec.add_variables(union.clone());
    ec.add_interval(&sub_task_interval);

    let mut task_lit: Vec<Lit> = vec![];
    for (i, s) in task_string.iter().enumerate() {
        if i == 0 {
            task_lit.push(ch.sym_table.declare_new_symbol(s, false, true).into());
        } else {
            task_lit.push(ch.sym_table.declare_new_symbol(s, true, true).into());
        }
    }
    //println!("({}) task lit: {:#?}", task_label, task_lit);
    ch.tasks.push(task_lit.into());

    let create_method = |mut ec_branch: ExpressionChronicle,
                         local_variables: &im::HashSet<AtomId>,
                         ch: &mut ChronicleHierarchy,
                         branch: bool,
                         debug: LValue|
     -> Result<(), LError> {
        let mut method = Chronicle::new(ch);
        let mut task_lit: Vec<AtomId> = vec![];
        for (i, s) in task_string.iter().enumerate() {
            if i == 0 {
                task_lit.push(ch.sym_table.declare_new_symbol(s, false, true));
            } else {
                task_lit.push(ch.sym_table.declare_new_symbol(s, true, true))
            };
        }
        let method_label = format!("m_{}_{}", task_label, branch);
        let method_id = ch.sym_table.declare_new_symbol(&method_label, false, false);
        let method_cond_var =
            ch.sym_table
                .declare_new_symbol(&format!("{}_cond", method_label), false, true);
        ec_branch.add_var(&method_cond_var);
        //Bindings of variables of task and method
        ec_branch.add_constraint(Constraint::Eq(method_cond_var.into(), task_lit[1].into()));
        let method_result_var =
            ch.sym_table
                .declare_new_symbol(&format!("{}_r", method_label), false, true);
        ec_branch.add_var(&method_result_var);
        ec_branch.add_constraint(Constraint::Eq(method_result_var.into(), task_lit[2].into()));

        let mut method_name: Vec<Lit> = vec![
            method_id.into(),
            method_cond_var.into(),
            method_result_var.into(),
        ];
        ec_branch.add_constraint(Constraint::Eq(
            ec_branch.get_result(),
            method_result_var.into(),
        ));

        for (i, var) in union.iter().enumerate() {
            let var = if local_variables.contains(var) {
                method_name.push(var.into());
                *var
            } else {
                let sym: Sym = ch.sym_table.get_atom(var).unwrap().try_into()?;
                let new_var = ch
                    .sym_table
                    .declare_new_symbol(sym.get_string(), true, true);
                ec_branch.add_var(&new_var);
                method_name.push(new_var.into());
                new_var
            };
            ec_branch.add_constraint(Constraint::Eq(var.into(), task_lit[3 + i].into()));
        }

        method.set_debug(Some(debug));
        method.set_task(task_lit.into());
        method.set_name(method_name.into());
        ec_branch.add_condition(Condition {
            interval: Interval::new(
                &ec_branch.get_interval().start(),
                &ec_branch.get_interval().start(),
            ),
            constraint: Constraint::Eq(method_cond_var.into(), ch.sym_table.new_bool(true).into()),
        });
        post_processing(&mut ec_branch, context, ch)?;
        method.absorb_expression_chronicle(ec_branch, &mut ch.sym_table);
        ch.methods.push(method);
        Ok(())
    };

    create_method(ec_b_true, &variables_b_true, ch, true, b_true.clone())?;
    create_method(ec_b_false, &variables_b_false, ch, false, b_false.clone())?;

    ch.sym_table.revert_scope();
    Ok(ec)

    //Construction of the first method;
}
