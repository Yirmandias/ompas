use sompas_language::*;
use sompas_structs::lenv::LEnv;
use sompas_structs::lerror::LError::*;
use sompas_structs::lerror::LResult;
use sompas_structs::lvalue::LValue;
use sompas_structs::typelvalue::TypeLValue;

pub fn check(args: &[LValue], _: &LEnv) -> LResult {
    if args.len() != 1 {
        return Err(WrongNumberOfArgument(CHECK, args.into(), args.len(), 1..1));
    }
    match &args[0] {
        LValue::True => Ok(LValue::True),
        LValue::Nil => Ok(LValue::Err(Box::new(LValue::Nil))),
        _ => Err(WrongType(
            CHECK,
            args[0].clone(),
            (&args[0]).into(),
            TypeLValue::Bool,
        )),
    }
}

pub fn err(args: &[LValue], _: &LEnv) -> LResult {
    if args.len() != 1 {
        return Err(WrongNumberOfArgument(ERR, args.into(), args.len(), 1..1));
    }
    Ok(LValue::Err(Box::new(args[0].clone())))
}

pub fn is_err(args: &[LValue], _: &LEnv) -> LResult {
    if args.len() != 1 {
        return Err(WrongNumberOfArgument(IS_ERR, args.into(), args.len(), 1..1));
    }
    Ok(matches!(args[0], LValue::Err(_)).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sompas_structs::lerror;

    #[test]
    pub fn test_err() -> lerror::Result<()> {
        let env = LEnv::default();
        let lv = 5.into();
        let result = err(&[lv], &env)?;

        assert_eq!(LValue::Err(Box::new(5.into())), result);
        Ok(())
    }

    #[test]
    pub fn test_is_err() -> lerror::Result<()> {
        let env = LEnv::default();
        let result = is_err(&[LValue::Err(Box::new(5.into()))], &env)?;
        assert_eq!(result, LValue::True);
        let result = is_err(&[5.into()], &env)?;
        assert_eq!(result, LValue::Nil);
        Ok(())
    }

    #[test]
    pub fn test_check() -> lerror::Result<()> {
        let env = LEnv::default();
        let result = check(&[LValue::Nil], &env)?;
        assert_eq!(result, LValue::Err(Box::new(LValue::Nil)));
        let result = check(&[LValue::True], &env)?;
        assert_eq!(result, LValue::True);
        Ok(())
    }
}