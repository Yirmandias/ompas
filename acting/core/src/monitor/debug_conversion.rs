use crate::exec::state::ModState;
use crate::monitor::domain::ModDomain;
use ompas_language::exec::refinement::EXEC_TASK;
use ompas_language::monitor::debug_conversion::{
    CONVERT_DOMAIN, DOC_CONVERT_DOMAIN, DOC_MOD_DEBUG_CONVERSION, DOC_PLAN_TASK, DOC_PRE_EVAL_EXPR,
    DOC_PRE_EVAL_TASK, MOD_DEBUG_CONVERSION, PLAN_TASK, PRE_EVAL_EXPR, PRE_EVAL_TASK,
};
use ompas_language::monitor::domain::MOD_DOMAIN;
use ompas_middleware::logger::LogClient;
use ompas_planning::aries::solver::run_solver_for_htn;
use ompas_planning::aries::{generate_chronicles, solver};
use ompas_planning::conversion::flow::p_eval::r#struct::{PConfig, PLValue};
use ompas_planning::conversion::flow::p_eval::{p_eval, p_expand, P_EVAL};
use ompas_planning::conversion::{convert, convert_acting_domain, p_convert, p_convert_task};
use ompas_structs::conversion::chronicle::template::ChronicleTemplate;
use ompas_structs::conversion::context::ConversionContext;
use ompas_structs::planning::domain::PlanningDomain;
use ompas_structs::planning::instance::PlanningInstance;
use ompas_structs::planning::problem::PlanningProblem;
use sompas_language::LOG_TOPIC_INTERPRETER;
use sompas_macros::async_scheme_fn;
use sompas_structs::lenv::LEnv;
use sompas_structs::llambda::LLambda;
use sompas_structs::lmodule::LModule;
use sompas_structs::lruntimeerror::{LResult, LRuntimeError};
use sompas_structs::lvalue::LValue;
use sompas_structs::string;
use std::time::SystemTime;

#[derive(Default)]
pub struct ModDebugConversion {}

impl From<ModDebugConversion> for LModule {
    fn from(m: ModDebugConversion) -> Self {
        let mut m = LModule::new(m, MOD_DEBUG_CONVERSION, DOC_MOD_DEBUG_CONVERSION);
        m.add_async_fn(CONVERT_DOMAIN, convert_domain, DOC_CONVERT_DOMAIN, false);
        m.add_async_fn(PLAN_TASK, plan_task, DOC_PLAN_TASK, false);
        m.add_async_fn(PRE_EVAL_TASK, pre_eval_task, DOC_PRE_EVAL_TASK, false);
        m.add_async_fn(PRE_EVAL_EXPR, pre_eval_expr, DOC_PRE_EVAL_EXPR, false);
        m
    }
}

#[async_scheme_fn]
pub async fn convert_domain(env: &LEnv) -> Result<String, LRuntimeError> {
    let ctx = env.get_context::<ModDomain>(MOD_DOMAIN)?;
    let context: ConversionContext = ctx.get_conversion_context().await;
    let time = SystemTime::now();
    let pd: PlanningDomain = convert_acting_domain(&context).await?;
    let time = time.elapsed().expect("could not get time").as_micros();
    Ok(format!("{}\n\nTime to convert: {} µs.", pd, time))
}

#[async_scheme_fn]
pub async fn plan_task(env: &LEnv, args: &[LValue]) -> LResult {
    let task: LValue = args.into();
    println!("task to plan: {}", task);
    let ctx = env.get_context::<ModDomain>(MOD_DOMAIN)?;
    let mut context: ConversionContext = ctx.get_conversion_context().await;
    context
        .env
        .update_context(ModState::new_from_snapshot(context.state.clone()));
    let instances: Vec<ChronicleTemplate> = p_convert_task(args, &context).await?;
    let pd: PlanningDomain = p_convert(&instances, &context).await?;

    let st = pd.st.clone();
    let problem: PlanningProblem = PlanningProblem {
        domain: pd,
        instance: PlanningInstance {
            state: context.state,
            tasks: vec![task.try_into()?],
            instances,
        },
        st,
    };

    let mut aries_problem = generate_chronicles(&problem)?;

    //println!("{}", aries_problem)
    let result = run_solver_for_htn(&mut aries_problem, true);
    // println!("{}", format_partial_plan(&pb, &x)?);

    let result: LValue = if let Some(x) = &result {
        let plan = solver::extract_plan(x);
        println!("plan:\n{}\n{}", plan.format(), plan.format_hierarchy());
        //let first_task_id = plan.get_first_subtask().unwrap();
        /*let subplan = plan.extract_sub_plan(first_task_id);
        println!(
            "subplan: \n{}\n{}",
            subplan.format(),
            subplan.format_hierarchy()
        );*/
        solver::extract_instantiated_methods(x)?
    } else {
        string!("no solution found".to_string())
    };

    Ok(result)
}

#[async_scheme_fn]
pub async fn pre_eval_task(env: &LEnv, task: &[LValue]) -> Result<(), LRuntimeError> {
    let ctx = env.get_context::<ModDomain>(MOD_DOMAIN)?;
    let mut context: ConversionContext = ctx.get_conversion_context().await;
    context
        .env
        .update_context(ModState::new_from_snapshot(context.state.clone()));

    let t = context
        .domain
        .tasks
        .get(task[0].to_string().as_str())
        .unwrap();

    let params = t.get_parameters().get_params();
    let mut pc = PConfig::default();
    pc.avoid.insert(EXEC_TASK.to_string());
    //pc.avoid.insert(CHECK.to_string());
    assert_eq!(params.len(), task.len() - 1);
    for (param, value) in params.iter().zip(task[1..].iter()) {
        pc.p_table
            .add_instantiated(param.to_string(), value.clone());
    }

    for m_label in t.get_methods() {
        let mut pc = pc.clone();
        let method = context.domain.methods.get(m_label).unwrap();
        for param in &method.parameters.get_params()[params.len()..] {
            pc.p_table.add_param(param.to_string());
        }
        let body = method.get_body();
        let mut env = context.env.clone();
        env.log = LogClient::new(P_EVAL, LOG_TOPIC_INTERPRETER).await;
        let lambda: LLambda = body.try_into()?;
        let lv = lambda.get_body();
        let plv = p_eval(lv, &mut env, &mut pc).await?;
        println!(
            "Pre eval method {m_label} of task {}:\n{}\n->\n{}",
            LValue::from(task).format(0),
            lv.format(0),
            plv.get_lvalue().format(0),
        )
    }
    Ok(())
}

#[async_scheme_fn]
pub async fn pre_eval_expr(env: &LEnv, lv: LValue) -> Result<(), LRuntimeError> {
    let ctx = env.get_context::<ModDomain>(MOD_DOMAIN)?;
    let mut context: ConversionContext = ctx.get_conversion_context().await;
    context
        .env
        .update_context(ModState::new_from_snapshot(context.state.clone()));

    let mut pc = PConfig::default();
    pc.avoid.insert(EXEC_TASK.to_string());
    //pc.avoid.insert(CHECK.to_string());
    let mut env = context.env.clone();
    env.log = LogClient::new(P_EVAL, LOG_TOPIC_INTERPRETER).await;
    let plv: PLValue = p_expand(&lv, true, &mut env, &pc).await?;
    let plv: PLValue = p_eval(&plv.get_lvalue(), &mut env, &mut pc).await?;
    println!(
        "Pre eval expr:\n{}\n->\n{}",
        lv.format(0),
        plv.get_lvalue().format(0),
    );

    Ok(())
}
