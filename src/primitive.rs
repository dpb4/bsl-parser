use std::collections::HashMap;

use crate::Expression;

#[derive(Debug, Clone)]
pub struct Lambda(pub (Vec<String>, Box<super::Expression>));

#[derive(Debug, Clone)]
pub enum Primitive {
    Natural(u32),
    Integer(i32),
    Number(f32),
    String(String),
    List(ConsList),
    Lambda(Lambda),
}

#[derive(Debug, Clone)]
pub enum ConsList {
    Empty,
    Cons((Box<Primitive>, Box<ConsList>)),
}

impl Primitive {
    pub fn coerce_f32(&self) -> Result<f32, EvalErr> {
        match *self {
            Primitive::Natural(n) => Ok(n as f32),
            Primitive::Integer(n) => Ok(n as f32),
            Primitive::Number(n) => Ok(n),
            _ => Err(EvalErr {
                err_type: EvalErrType::TypeCoercion(self.clone()),
                msg: format!("Unable to coerce value {self} to type f32"),
                line_num: 0,
                col_num: 0,
            }),
        }
    }
    pub fn coerce_i32(&self) -> Result<i32, EvalErr> {
        match *self {
            Primitive::Natural(n) => Ok(n as i32),
            Primitive::Integer(n) => Ok(n),
            Primitive::Number(n) => {
                if n - n.round() == 0.0 {
                    Ok(n as i32)
                } else {
                    Err(EvalErr {
                        err_type: EvalErrType::TypeCoercion(self.clone()),
                        msg: format!("Unable to coerce value {self} to type i32"),
                        line_num: 0,
                        col_num: 0,
                    })
                }
            }
            _ => Err(EvalErr {
                err_type: EvalErrType::TypeCoercion(self.clone()),
                msg: format!("Unable to coerce value {self} to type i32"),
                line_num: 0,
                col_num: 0,
            }),
        }
    }
    pub fn coerce_u32(&self) -> Result<u32, EvalErr> {
        match *self {
            Primitive::Natural(n) => Ok(n),
            Primitive::Integer(n) => {
                if n >= 0 {
                    Ok(n as u32)
                } else {
                    Err(EvalErr {
                        err_type: EvalErrType::TypeCoercion(self.clone()),
                        msg: format!("Unable to coerce value {self} to type u32"),
                        line_num: 0,
                        col_num: 0,
                    })
                }
            }
            Primitive::Number(n) => {
                if n - n.round() == 0.0 {
                    Ok(n as u32)
                } else {
                    Err(EvalErr {
                        err_type: EvalErrType::TypeCoercion(self.clone()),
                        msg: format!("Unable to coerce value {self} to type u32"),
                        line_num: 0,
                        col_num: 0,
                    })
                }
            }
            _ => Err(EvalErr {
                err_type: EvalErrType::TypeCoercion(self.clone()),
                msg: format!("Unable to coerce value {self} to type u32"),
                line_num: 0,
                col_num: 0,
            }),
        }
    }

    pub fn try_from_str(input: &str) -> Result<Self, &str> {
        if input == "empty" {
            return Ok(Self::List(ConsList::Empty));
        }

        if input.starts_with("\"") && input.ends_with("\"")
            || input.starts_with("'") && input.ends_with("'")
        {
            return Ok(Self::String(input[1..input.len() - 1].to_string()));
        }

        match input.parse::<u32>() {
            Ok(n) => Ok(Self::Natural(n)),
            Err(_) => match input.parse::<i32>() {
                Ok(n) => Ok(Self::Integer(n)),
                Err(_) => match input.parse::<f32>() {
                    Ok(n) => Ok(Self::Number(n)),
                    Err(_) => Err(input),
                },
            },
        }
    }
}

impl ConsList {
    fn to_string(&self) -> String {
        match self {
            ConsList::Empty => String::from("empty"),
            ConsList::Cons((e, r)) => format!("(cons {} {})", *e, r.to_string()),
        }
    }
}

impl std::fmt::Display for ConsList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::Natural(n) => format!("Natural: {n}"),
                Self::Integer(i) => format!("Integer: {i}"),
                Self::Number(x) => format!("Number: {x}"),
                Self::String(s) => format!("String: {s}"),
                Self::List(cl) => cl.to_string(),
                Primitive::Lambda(_) => todo!(),
            }
        )
    }
}
#[derive(Debug)]
pub enum EvalErrType {
    TypeMismatch(Primitive),
    TypeCoercion(Primitive),
}

#[derive(Debug)]
pub struct EvalErr {
    err_type: EvalErrType,
    msg: String,
    line_num: u32,
    col_num: u32,
}

fn add(a: &Primitive, b: &Primitive) -> Result<Primitive, EvalErr> {
    if matches!(a, Primitive::String(_) | Primitive::List(_)) {
        return Err(EvalErr {
            err_type: EvalErrType::TypeMismatch(a.clone()),
            msg: format!("(+) expected a Natural or Integer or Number, found {b}"),
            line_num: 0,
            col_num: 0,
        });
    } else if matches!(b, Primitive::String(_) | Primitive::List(_)) {
        return Err(EvalErr {
            err_type: EvalErrType::TypeMismatch(b.clone()),
            msg: format!("(+) expected a Natural or Integer or Number, found {b}"),
            line_num: 0,
            col_num: 0,
        });
    }

    if matches!(a, Primitive::Number(_)) || matches!(b, Primitive::Number(_)) {
        Ok(Primitive::Number(a.coerce_f32()? + b.coerce_f32()?))
    } else if matches!(a, Primitive::Integer(_)) || matches!(b, Primitive::Integer(_)) {
        Ok(Primitive::Integer(a.coerce_i32()? + b.coerce_i32()?))
    } else {
        Ok(Primitive::Natural(a.coerce_u32()? + b.coerce_u32()?))
    }
}
