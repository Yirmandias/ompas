use crate::lisp_root::lisp_struct::LError::*;
use crate::lisp_root::lisp_struct::*;
use crate::lisp_root::RefLEnv;
use aries_utils::input::Sym;
use im::HashMap;

//Mathematical functions
pub fn add(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    let mut result = LValue::Number(LNumber::Float(0.0));
    for value in values {
        result = (&result + value)?;
    }
    Ok(result)
}

pub fn sub(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    match values.len() {
        2 => values.get(0).unwrap() - values.get(1).unwrap(),
        i => Err(WrongNumberOfArgument(values.into(), i, 2..2)),
    }
}

pub fn mul(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    let mut result = LValue::Number(LNumber::Float(1.0));
    for value in values {
        result = (&result * value)?;
    }
    Ok(result)
}

pub fn div(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    match values.len() {
        2 => values.get(0).unwrap() / values.get(0).unwrap(),
        i => Err(WrongNumberOfArgument(values.into(), i, 2..2)),
    }
}

//Comparison functions
pub fn gt(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Bool(values[0] > values[1])),
        i => Err(WrongNumberOfArgument(values.into(), i, 2..2)),
    }
}

pub fn lt(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Bool(values[0] < values[1])),
        i => Err(WrongNumberOfArgument(values.into(), i, 2..2)),
    }
}

pub fn ge(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Bool(values[0] >= values[1])),
        i => Err(WrongNumberOfArgument(values.into(), i, 2..2)),
    }
}

pub fn le(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Bool(values[0] <= values[1])),
        i => Err(WrongNumberOfArgument(values.into(), i, 2..2)),
    }
}

pub fn eq(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Bool(values[0] == values[1])),
        i => Err(WrongNumberOfArgument(values.into(), i, 2..2)),
    }
}

pub fn begin(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    match values.last() {
        None => Err(LError::SpecialError("no SExpr after begin".to_string())),
        Some(v) => Ok(v.clone()),
    }
}

pub fn default(_values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    Ok(LValue::String("default function".to_string()))
}

pub fn list(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    Ok(LValue::List(values.to_vec()))
}

pub fn map(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    let mut facts: HashMap<LValue, LValue> = Default::default();
    if values.len() != 1 {
        return Err(WrongNumberOfArgument(values.into(), values.len(), 1..1));
    }
    match values.get(0).unwrap() {
        LValue::List(list_sv) => {
            //println!("list_sv : {:?}", list_sv);
            for sv in list_sv {
                match sv {
                    LValue::List(val_sv) => {
                        //println!("sv: {:?}", val_sv);
                        if val_sv
                            .get(1)
                            .unwrap()
                            .as_sym()
                            .unwrap_or_else(|_| Sym::from(""))
                            .as_str()
                            .eq(".")
                        {
                            //println!("insert a new fact");
                            let key = val_sv.get(0).unwrap().clone();
                            let value = val_sv.get(2).unwrap().clone();
                            facts.insert(key, value);
                        } else {
                            //println!("doesn't match pattern")
                        }
                    }
                    lv => return Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::List)),
                }
            }
        }
        lv => return Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::List)),
    }
    Ok(LValue::Map(facts))
}

//TODO: Define set behaviour for other type of LValue
pub fn set(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.len() != 2 {
        return Err(WrongNumberOfArgument(values.into(), values.len(), 2..2));
    }
    let lv = values.get(0).unwrap();
    match lv {
        LValue::Map(s) => {
            let mut facts = s.clone();
            if let LValue::List(list) = values.get(1).unwrap() {
                if list.len() != 2 {
                    return Err(WrongNumberOfArgument(values.into(), list.len(), 2..2));
                }
                let key = list.get(0).unwrap();
                let value = list.get(1).unwrap();
                facts.insert(key.clone(), value.clone());
            }
            Ok(LValue::Map(facts))
        }
        lv => Err(LError::SpecialError(format!(
            "Cannot set a {}",
            NameTypeLValue::from(lv)
        ))),
    }
}

pub fn get(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.is_empty() {
        return Err(WrongNumberOfArgument(values.into(), 0, 1..std::usize::MAX));
    }
    let lv = values.get(0).unwrap();
    match lv {
        LValue::Map(map) => {
            if values.len() == 2 {
                let key = values.get(1).unwrap();
                let value = map.get(key).unwrap_or(&LValue::None);
                Ok(value.clone())
            } else if values.len() == 1 {
                Ok(LValue::Map(map.clone()))
            } else {
                Err(WrongNumberOfArgument(lv.clone(), values.len(), 1..2))
            }
        }
        lv => {
            if values.len() > 1 {
                return Err(WrongNumberOfArgument(values.into(), values.len(), 1..1));
            }
            Ok(lv.clone())
        }
    }
}

/*pub fn print(values: &[LValue], _:& RefLEnv, _: & ()) -> Result<LValue, LError> {
    if values.is_empty() {
        return Err(WrongNumberOfArgument(values.into(), 0, 1..std::usize::MAX));
    }
    return Ok(values.get(0).unwrap().clone());
}*/

pub fn read(_: &[LValue], _: &mut RefLEnv, _: &()) -> Result<LValue, LError> {
    unimplemented!()
    /*if values.len() != 1 {
        return Err(WrongNumberOfArgument(values.into(), values.len(), 1..1))
            }
    let file_name = match values.get(0).unwrap() {
        LValue::Symbol(s) => s.to_string(),
        lv => return Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::Symbol)),
    };
    let mut file = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => return Err(SpecialError(e.to_string())),
    };
    let mut buffer = String::new();
    match file.read_to_string(&mut buffer) {
        Ok(_) => {}
        Err(e) => return Err(SpecialError(e.to_string())),
    };
    let lv = parse(buffer.as_str(), env)?;
    eval(&lv, env)*/
}

pub fn write(values: &[LValue], env: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.len() != 1 {
        return Err(WrongNumberOfArgument(values.into(), values.len(), 1..1));
    }
    let file_name = match values.get(0).unwrap() {
        LValue::Symbol(s) => s.to_string(),
        lv => return Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::Symbol)),
    };
    env.to_file(file_name);
    Ok(LValue::None)
}

///It takes two arguments, an element and a list and returns a list with the element inserted at the first place.
pub fn cons(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.len() != 2 {
        return Err(WrongNumberOfArgument(values.into(), values.len(), 2..2));
    }
    let first = values.first().unwrap();
    let second = values.get(1).unwrap();
    match (first, second) {
        (lv_first, LValue::List(list)) => {
            let mut new_list = vec![lv_first.clone()];
            new_list.append(&mut list.clone());
            Ok(new_list.into())
        }
        (lv_first, LValue::None) => Ok(lv_first.clone()),
        (lv_f, lv_s) => Ok(vec![lv_f.clone(), lv_s.clone()].into()),
    }
}

///It takes a list as argument, and returns its first element.
pub fn car(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.len() == 1 {
        match values.first().unwrap() {
            LValue::List(list) => {
                if !list.is_empty() {
                    Ok(list.first().unwrap().clone())
                } else {
                    Ok(LValue::None)
                }
            }
            lv => Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::List)),
        }
    } else {
        Err(WrongNumberOfArgument(values.into(), values.len(), 1..1))
    }
}

///It takes a list as argument, and returns a list without the first element
pub fn cdr(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.len() == 1 {
        match values.first().unwrap() {
            LValue::List(list) => {
                if list.len() > 2 {
                    Ok(LValue::None)
                } else {
                    let mut new_list = list.clone();
                    new_list.remove(0);
                    Ok(new_list.into())
                }
            }
            lv => Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::List)),
        }
    } else {
        Err(WrongNumberOfArgument(values.into(), values.len(), 1..1))
    }
}

///It merges two or more list into one.
pub fn append(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    let mut new_list = Vec::new();
    for element in values {
        match element {
            LValue::List(list) => new_list.append(&mut list.clone()),
            _ => {
                return Err(WrongType(
                    element.clone(),
                    element.into(),
                    NameTypeLValue::List,
                ))
            }
        }
    }
    Ok(new_list.into())
}

///It takes a list and returns the last element.
pub fn last(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.len() == 1 {
        match values.first().unwrap() {
            LValue::List(list) => {
                if !list.is_empty() {
                    Ok(list.last().unwrap().clone())
                } else {
                    Ok(LValue::None)
                }
            }
            lv => Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::List)),
        }
    } else {
        Err(WrongNumberOfArgument(values.into(), values.len(), 1..1))
    }
}

///It takes two arguments of which the second must be a list,
/// if the first argument is a member of the second argument,
/// and then it returns the remainder of the list beginning with the first argument.
pub fn member(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.len() != 2 {
        return Err(WrongNumberOfArgument(values.into(), values.len(), 2..2));
    }
    let value_to_find = values.get(1).unwrap();
    match values.get(1).unwrap() {
        LValue::List(list) => {
            for (k, element) in list.iter().enumerate() {
                if element == value_to_find {
                    return Ok(list[k - 1..].into());
                }
            }
            Ok(LValue::None)
        }
        lv => Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::List)),
    }
}

/// It takes a list and returns a list with the top elements in reverse order.
pub fn reverse(values: &[LValue], _: &RefLEnv, _: &()) -> Result<LValue, LError> {
    if values.len() == 1 {
        match values.first().unwrap() {
            LValue::List(list) => {
                let mut new_list = list.clone();
                new_list.reverse();
                Ok(new_list.into())
            }
            lv => Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::List)),
        }
    } else {
        Err(WrongNumberOfArgument(values.into(), values.len(), 1..1))
    }
}

/// return the length of the object if it is a table or a list.
pub fn length(_values: &[LValue], _: &mut RefLEnv, _: &()) -> Result<LValue, LError> {
    unimplemented!()
}
