use crate::module::rae_exec::error::RaeExecError;
use crate::module::rae_exec::*;
use ::macro_rules_attribute::macro_rules_attribute;
use log::{info, warn};
use ompas_lisp::core::structs::lenv::LEnv;
use ompas_lisp::core::structs::lerror::LError::WrongNumberOfArgument;
use ompas_lisp::core::structs::lerror::{LError, LResult};
use ompas_lisp::core::structs::lnumber::LNumber;
use ompas_lisp::core::structs::lvalue::LValue;
use ompas_lisp::core::{eval, parse};
use ompas_lisp::modules::utils::contains;
use ompas_utils::dyn_async;
use std::convert::{TryFrom, TryInto};

pub const RAE_EXEC_COMMAND: &str = "rae-exec-command";
pub const RAE_LAUNCH_PLATFORM: &str = "rae-launch-platform";
pub const RAE_OPEN_COM_PLATFORM: &str = "rae-open-com-platform";
pub const RAE_START_PLATFORM: &str = "rae-start-platform";
pub const RAE_IS_PLATFORM_DEFINED: &str = "rae-platform?";

pub fn is_platform_defined(_: &[LValue], env: &LEnv) -> LResult {
    let ctx = env.get_context::<CtxRaeExec>(MOD_RAE_EXEC)?;
    Ok(ctx.platform_interface.is_some().into())
}

#[macro_rules_attribute(dyn_async!)]
pub async fn exec_command<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    let ctx = env.get_context::<CtxRaeExec>(MOD_RAE_EXEC)?;

    let eval_model = || async {
        let string = format!("((get-action-model '{}) {})", args[0], {
            let mut str = "".to_string();
            for e in &args[1..] {
                str.push_str(format!("'{}", e).as_str())
            }
            str
        });
        //println!("in eval model, string: {}", string);
        let mut env = env.clone();
        eval(&parse(&string, &mut env).await?, &mut env).await
    };

    let mode: String = env
        .get_symbol("rae-mode")
        .expect("rae-mode should be defined, default value is exec mode")
        .try_into()?;
    match mode.as_str() {
        SYMBOL_EXEC_MODE => {
            match &ctx.platform_interface {
                Some(platform) => {
                    let parent_task: usize = env
                        .get_ref_symbol(PARENT_TASK)
                        .map(|n| LNumber::try_from(n).unwrap().into())
                        .unwrap();
                    let debug: LValue = args.into();
                    let (action_id, mut rx) = ctx.agenda.add_action(args.into(), parent_task).await;
                    info!("exec command {}: {}", action_id, debug);
                    platform.exec_command(args, action_id).await?;

                    //println!("await on action (id={})", action_id);
                    info!("waiting on action {}", action_id);

                    let mut action_status: TaskStatus = ctx.agenda.get_status(action_id).await;

                    loop {
                        //println!("waiting on status:");
                        match action_status {
                            TaskStatus::Pending => {
                                //println!("not triggered");
                            }
                            TaskStatus::Running => {
                                //println!("running");
                            }
                            TaskStatus::Failure => {
                                warn!("Command {} is a failure.", action_id);
                                ctx.agenda.set_end_time(action_id).await;
                                return Ok(RaeExecError::ActionFailure.into());
                            }
                            TaskStatus::Done => {
                                info!("Command {} is a success.", action_id);
                                ctx.agenda.set_end_time(action_id).await;
                                return Ok(true.into());
                            }
                        }
                        action_status = rx.recv().await.unwrap();
                    }
                }
                None => eval_model().await,
            }
        }
        SYMBOL_SIMU_MODE => eval_model().await,
        _ => unreachable!(
            "{} should have either {} or {} value.",
            SYMBOL_RAE_MODE, SYMBOL_EXEC_MODE, SYMBOL_SIMU_MODE
        ),
    }
}

#[macro_rules_attribute(dyn_async!)]
pub async fn launch_platform<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    let ctx = env.get_context::<CtxRaeExec>(MOD_RAE_EXEC)?;

    if let Some(platform) = &ctx.platform_interface {
        platform.launch_platform(args).await
    } else {
        Ok("No platform defined".into())
    }
}

#[macro_rules_attribute(dyn_async!)]
pub async fn start_platform<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    let ctx = env.get_context::<CtxRaeExec>(MOD_RAE_EXEC)?;

    if let Some(platform) = &ctx.platform_interface {
        platform.start_platform(args).await
    } else {
        Ok("No platform defined".into())
    }
}
#[macro_rules_attribute(dyn_async!)]
pub async fn open_com<'a>(args: &'a [LValue], env: &'a LEnv) -> Result<LValue, LError> {
    let ctx = env.get_context::<CtxRaeExec>(MOD_RAE_EXEC)?;

    if let Some(platform) = &ctx.platform_interface {
        platform.open_com(args).await
    } else {
        Ok("No platform defined".into())
    }
}

#[macro_rules_attribute(dyn_async!)]
pub async fn cancel_command<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    let ctx = env.get_context::<CtxRaeExec>(MOD_RAE_EXEC)?;

    let mode: String = env
        .get_symbol("rae-mode")
        .expect("rae-mode should be defined, default value is exec mode")
        .try_into()?;
    match mode.as_str() {
        SYMBOL_EXEC_MODE => {
            if let Some(platform) = &ctx.platform_interface {
                platform.cancel_command(args).await
            } else {
                Ok("cannot cancel command in internal platform (action is instantaneous)".into())
            }
        }
        SYMBOL_SIMU_MODE => Ok("No cancellation of action in simulation mode".into()),
        _ => unreachable!(
            "{} should have either {} or {} value.",
            SYMBOL_RAE_MODE, SYMBOL_EXEC_MODE, SYMBOL_SIMU_MODE
        ),
    }
}

#[macro_rules_attribute(dyn_async!)]
pub async fn instance<'a>(args: &'a [LValue], env: &'a LEnv) -> LResult {
    let ctx = env.get_context::<CtxRaeExec>(MOD_RAE_EXEC)?;
    let mode: String = env
        .get_symbol("rae-mode")
        .expect("rae-mode should be defined, default value is exec mode")
        .try_into()?;
    match mode.as_str() {
        SYMBOL_EXEC_MODE => {
            if let Some(platform) = &ctx.platform_interface {
                platform.instance(args).await
            } else {
                let facts: im::HashMap<LValue, LValue> = get_facts(&[], env).await?.try_into()?;
                match args.len() {
                    1 => {
                        let key = vec![RAE_INSTANCE.into(), args[0].clone()].into();
                        let value = facts.get(&key).unwrap_or(&LValue::Nil);
                        Ok(value.clone())
                    }
                    2 => {
                        let key = vec![RAE_INSTANCE.into(), args[1].clone()].into();
                        let instances = facts.get(&key).unwrap_or(&LValue::Nil);
                        contains(&[instances.clone(), args[0].clone()], env)
                    }
                    _ => Err(WrongNumberOfArgument(
                        RAE_INSTANCE,
                        args.into(),
                        args.len(),
                        1..2,
                    )),
                }
            }
        }
        SYMBOL_SIMU_MODE => {
            let state: im::HashMap<LValue, LValue> = env.get_symbol(STATE).unwrap().try_into()?;
            match args.len() {
                1 => {
                    let key = vec![RAE_INSTANCE.into(), args[0].clone()].into();
                    let value = state.get(&key).unwrap_or(&LValue::Nil);
                    Ok(value.clone())
                }
                2 => {
                    let key = vec![RAE_INSTANCE.into(), args[1].clone()].into();
                    let instances = state.get(&key).unwrap_or(&LValue::Nil);
                    contains(&[instances.clone(), args[0].clone()], env)
                }
                _ => Err(WrongNumberOfArgument(
                    RAE_INSTANCE,
                    args.into(),
                    args.len(),
                    1..2,
                )),
            }
        }
        _ => unreachable!(
            "{} should have either {} or {} value.",
            SYMBOL_RAE_MODE, SYMBOL_EXEC_MODE, SYMBOL_SIMU_MODE
        ),
    }
}
