use crate::rae_user::{CtxRae, MOD_RAE};
use ::macro_rules_attribute::macro_rules_attribute;
use ompas_rae_planning::binding_aries::solver::run_solver_for_htn;
use ompas_rae_planning::binding_aries::{generate_chronicles, solver};
use ompas_rae_planning::conversion::convert_domain_to_chronicle_hierarchy;
use ompas_rae_structs::planning::{ConversionContext, Problem};
use sompas_structs::lenv::LEnv;
use sompas_structs::lerror::LResult;
use sompas_structs::lvalue::LValue;
use sompas_utils::dyn_async;

#[macro_rules_attribute(dyn_async!)]
pub async fn plan_task<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    let task: LValue = args.into();
    println!("task to plan: {}", task);
    let ctx = env.get_context::<CtxRae>(MOD_RAE)?;
    let context: ConversionContext = ctx.get_conversion_context().await;
    let mut problem: Problem = (&context).into();
    let cc = convert_domain_to_chronicle_hierarchy(context)?;
    //println!("cc: {}", cc);
    problem.cc = cc;
    problem.goal_tasks.push(task.into());

    let mut aries_problem = generate_chronicles(&problem)?;

    let result = run_solver_for_htn(&mut aries_problem, true);
    // println!("{}", format_partial_plan(&pb, &x)?);

    let result: LValue = if let Some(x) = &result {
        let plan = solver::extract_plan(x);
        println!("plan:\n{}\n{}", plan.format(), plan.format_hierarchy());
        let first_task_id = plan.get_first_subtask().unwrap();
        let subplan = plan.extract_sub_plan(first_task_id);
        println!(
            "subplan: \n{}\n{}",
            subplan.format(),
            subplan.format_hierarchy()
        );
        solver::extract_instantiated_methods(x)?
    } else {
        LValue::String("no solution found".to_string())
    };

    Ok(result)
}
