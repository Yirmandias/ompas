//TODO: Vérifier si les fonctions ne doivent prendre que deux paramètres
use crate::lisp::lisp_struct::LError::*;
use crate::lisp::lisp_struct::*;
use crate::lisp::lisp_language::TYPE_OBJECT;

pub fn get(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        1 => Ok(values[0].clone()),
        len => Err(WrongNumerOfArgument(len, 1..1)),
    }
}



//Mathematical functions
pub fn add(values: Vec<LValue>) -> Result<LValue, LError> {
    let mut result = LValue::Atom(LAtom::Number(LNumber::Float(0.0)));
    for value in values {
        result = (result + value)?;
    }
    Ok(result)
}

pub fn sub(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        2 => {
            let mut first_val: f64 = 0.0;
            let mut second_val: f64 = 0.0;
            for (i, val) in values.iter().enumerate() {
                match val {
                    LValue::Atom(LAtom::Number(LNumber::Int(int))) => match i {
                        0 => first_val = *int as f64,
                        1 => second_val = *int as f64,
                        _ => panic!("Strong error"),
                    },
                    LValue::Atom(LAtom::Number(LNumber::Float(float))) => match i {
                        0 => first_val = *float,
                        1 => second_val = *float,
                        _ => panic!("Strong error"),
                    },
                    lv => {
                        return Err(LError::WrongType(
                            lv.clone().into(),
                            NameTypeLValue::NAtom,
                        ))
                    }
                };
            }

            Ok(LValue::Atom(LAtom::Number(LNumber::Float(
                first_val - second_val,
            ))))
        }
        i => Err(WrongNumerOfArgument(i, 2..2)),
    }
}

pub fn mul(values: Vec<LValue>) -> Result<LValue, LError> {
    let mut result: f64 = 1.0;
    for value in values {
        match value {
            LValue::Atom(LAtom::Number(LNumber::Int(i))) => result *= i as f64,
            LValue::Atom(LAtom::Number(LNumber::Float(f))) => result *= f,
            l => return Err(LError::WrongType(l.into(), NameTypeLValue::NAtom)),
        }
    }
    Ok(LValue::Atom(LAtom::Number(LNumber::Float(result))))
}

pub fn div(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        2 => {
            let mut first_val: f64 = 0.0;
            let mut second_val: f64 = 0.0;
            for (i, val) in values.iter().enumerate() {
                match val {
                    LValue::Atom(LAtom::Number(LNumber::Int(int))) => match i {
                        0 => first_val = *int as f64,
                        1 => second_val = *int as f64,
                        _ => panic!("Strong error"),
                    },
                    LValue::Atom(LAtom::Number(LNumber::Float(float))) => match i {
                        0 => first_val = *float,
                        1 => second_val = *float,
                        _ => panic!("Strong error"),
                    },
                    lv => {
                        return Err(LError::WrongType(
                            lv.clone().into(),
                            NameTypeLValue::NAtom,
                        ))
                    }
                };
            }

            Ok(LValue::Atom(LAtom::Number(LNumber::Float(
                first_val / second_val,
            ))))
        }
        i => Err(WrongNumerOfArgument(i, 2..2)),
    }
}

//Comparison functions
pub fn gt(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Atom(LAtom::Bool(values[0] > values[1]))),
        i => Err(WrongNumerOfArgument(i, 2..2)),
    }
}

pub fn lt(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Atom(LAtom::Bool(values[0] < values[1]))),
        i => Err(WrongNumerOfArgument(i, 2..2)),
    }
}

pub fn ge(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Atom(LAtom::Bool(values[0] >= values[1]))),
        i => Err(WrongNumerOfArgument(i, 2..2)),
    }
}

pub fn le(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Atom(LAtom::Bool(values[0] <= values[1]))),
        i => Err(WrongNumerOfArgument(i, 2..2)),
    }
}

pub fn eq(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        2 => Ok(LValue::Atom(LAtom::Bool(values[0] == values[1]))),
        i => Err(WrongNumerOfArgument(i, 2..2)),
    }
}

//Type verification
pub fn is_none(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        1 => Ok(LValue::Atom(LAtom::Bool(
            NameTypeLValue::from(values[0].clone()) == NameTypeLValue::None,
        ))),
        i => Err(WrongNumerOfArgument(i, 1..1)),
    }
}

pub fn is_number(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        1 => Ok(LValue::Atom(LAtom::Bool(
            NameTypeLValue::from(values[0].clone()) == NameTypeLValue::NAtom,
        ))),
        i => Err(WrongNumerOfArgument(i, 1..1)),
    }
}

pub fn is_bool(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        1 => Ok(LValue::Atom(LAtom::Bool(
            NameTypeLValue::from(values[0].clone()) == NameTypeLValue::BAtom,
        ))),
        i => Err(WrongNumerOfArgument(i, 1..1)),
    }
}

pub fn is_fn(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        1 => Ok(LValue::Atom(LAtom::Bool(
            NameTypeLValue::from(values[0].clone()) == NameTypeLValue::LFn,
        ))),
        i => Err(WrongNumerOfArgument(i, 1..1)),
    }
}

pub fn begin(values: Vec<LValue>) -> Result<LValue, LError> {
    Ok(values.last().unwrap().clone())
}

pub fn default(_values: Vec<LValue>) -> Result<LValue, LError> {
    Ok(LValue::String("default function".to_string()))
}

pub fn var(values: Vec<LValue>) -> Result<LValue, LError> {
    //println!("in function var");
    match values.len() {
        2 => {
            let atom_type = match values[0].clone() {
                LValue::Type(ltype) => ltype,
                lv => return Err(WrongType(lv.into(), NameTypeLValue::Type))
            };
            let atom_value:LAtom = match values[1].clone() {
                LValue::Variable(lvar) => {
                    if lvar.v_type == atom_type {
                        lvar.value
                    }else {
                        return Err(SpecialError(format!("Expected an atom of type {}, got {}", atom_type,lvar.v_type)))
                    }
                },
                LValue::Atom(latom) => {
                    latom
                }
                lv => return Err(WrongType(lv.into(), NameTypeLValue::Variable)),
            };
            Ok(LValue::Variable(LVariable {
                v_type: atom_type,
                value: atom_value,
            }))
        }
        len => Err(WrongNumerOfArgument(len, 2..2))
    }
}

pub fn object(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        1 => {
            //only the name of the symbol
            let l_value = values[0].clone();
            let t_type = LType::Symbol(TYPE_OBJECT.into());
            let value = match l_value {
                LValue::Atom(LAtom::Symbol(s)) => LAtom::Symbol(s),
                lv => return Err(WrongType(lv.into(), NameTypeLValue::SAtom))
            };
            Ok(LValue::Variable(LVariable {
                v_type: t_type,
                value
            }))
        }
        2 => {
            //TODO: Add binding to type for object
            let sym_type = values[0].clone();
            let sym_value = values[1].clone();
            let t_type = match sym_type {
                LValue::Type(LType::Symbol(s)) => LType::Symbol(s),
                lv => return Err(WrongType(lv.into(), NameTypeLValue::Type))
            };
            let value = match sym_value {
                LValue::Atom(LAtom::Symbol(s)) => LAtom::Symbol(s),
                lv => return Err(WrongType(lv.into(), NameTypeLValue::SAtom))
            };
            Ok(LValue::Variable(LVariable {
                v_type: t_type,
                value
            }))
        }
        len => Err(WrongNumerOfArgument(len, 1..2))
    }
}

pub fn state_function(values: Vec<LValue>) -> Result<LValue, LError> {
    let mut vec_params:Vec<LType> = Vec::new();
    let mut t_value:LType = LType::Symbol(TYPE_OBJECT.into());
    for (i,value) in values.iter().enumerate() {
        match value {
            LValue::Type(ltype) => {
                if i == values.len()-1 {
                    t_value = ltype.clone();
                }
                else {
                    vec_params.push(ltype.clone())
                }
            }
            lv => return Err(WrongType(lv.clone().into(), NameTypeLValue::Type))
        }
    }
    Ok(LValue::StateFunction(LStateFunction {
        t_params: vec_params,
        t_value: t_value
    }))
}

pub fn def_type(values: Vec<LValue>) -> Result<LValue, LError> {
    match values.len() {
        1 => {
            //Just the label
            match values[0].clone() {
                LValue::Atom(LAtom::Symbol(s)) => {
                    Ok(LValue::Type(LType::Symbol(s)))
                }
                lv => Err(WrongType(lv.into(), NameTypeLValue::SAtom))
            }
        }
        len => Err(WrongNumerOfArgument(len, 1..1))
    }
}