#[derive(Debug, Clone)]
pub struct Lambda(pub (Vec<String>, Box<super::Expression>));

#[derive(Debug, Clone)]
pub enum Primitive {
    Boolean(bool),
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

impl<T: Iterator<Item = Primitive>> From<T> for ConsList {
    fn from(mut value: T) -> Self {
        match value.next() {
            Some(v) => Self::Cons((Box::new(v), Box::new(Self::from(value)))),
            None => Self::Empty,
        }
    }
}

// TODO optimize
impl ConsList {
    pub fn length(&self) -> u32 {
        match self {
            ConsList::Empty => 0,
            ConsList::Cons((_, r)) => 1 + r.length(),
        }
    }
}

impl Primitive {
    pub fn coerce_f32(&self) -> Result<f32, String> {
        match *self {
            Primitive::Natural(n) => Ok(n as f32),
            Primitive::Integer(n) => Ok(n as f32),
            Primitive::Number(n) => Ok(n),
            _ => Err(format!("Unable to coerce value {self} to type f32")),
        }
    }
    pub fn coerce_i32(&self) -> Result<i32, String> {
        match *self {
            Primitive::Natural(n) => Ok(n as i32),
            Primitive::Integer(n) => Ok(n),
            Primitive::Number(n) => {
                if n - n.round() == 0.0 {
                    Ok(n as i32)
                } else {
                    Err(format!("Unable to coerce value {self} to type i32"))
                }
            }
            _ => Err(format!("Unable to coerce value {self} to type i32")),
        }
    }
    pub fn coerce_u32(&self) -> Result<u32, String> {
        match *self {
            Primitive::Natural(n) => Ok(n),
            Primitive::Integer(n) => {
                if n >= 0 {
                    Ok(n as u32)
                } else {
                    Err(format!("Unable to coerce value {self} to type u32"))
                }
            }
            Primitive::Number(n) => {
                if n - n.round() == 0.0 {
                    Ok(n as u32)
                } else {
                    Err(format!("Unable to coerce value {self} to type u32"))
                }
            }
            _ => Err(format!("Unable to coerce value {self} to type u32")),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Primitive::Natural(_) | Primitive::Integer(_) | Primitive::Number(_)
        )
    }

    pub fn try_from_str(input: &str) -> Option<Self> {
        if input == "empty" {
            return Some(Self::List(ConsList::Empty));
        } else if input == "true" {
            return Some(Self::Boolean(true));
        } else if input == "false" {
            return Some(Self::Boolean(false));
        }

        if input.starts_with("\"") && input.ends_with("\"")
            || input.starts_with("'") && input.ends_with("'")
        {
            return Some(Self::String(input[1..input.len() - 1].to_string()));
        }

        match input.parse::<u32>() {
            Ok(n) => Some(Self::Natural(n)),
            Err(_) => match input.parse::<i32>() {
                Ok(n) => Some(Self::Integer(n)),
                Err(_) => match input.parse::<f32>() {
                    Ok(n) => Some(Self::Number(n)),
                    Err(_) => None,
                },
            },
        }
    }
}

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Natural(l0), Self::Natural(r0)) => l0 == r0,
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Lambda(_), Self::Lambda(_)) => false,
            _ => false,
        }
    }
}

impl Eq for Primitive {}

impl PartialEq for ConsList {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Cons((ae, ar)), Self::Cons((be, br))) => {
                if *ae == *be {
                    *ar == *br
                } else {
                    false
                }
            }
            (Self::Empty, Self::Empty) => true,
            _ => false,
        }
    }
}
impl Eq for ConsList {}

impl std::fmt::Display for ConsList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsList::Empty => write!(f, "empty"),
            ConsList::Cons((e, r)) => write!(f, "(cons {} {})", *e, r),
        }
    }
}

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::Boolean(b) => format!("{b} : Boolean"),
                Self::Natural(n) => format!("{n} : Natural"),
                Self::Integer(i) => format!("{i} : Integer"),
                Self::Number(x) => format!("{x} : Number"),
                Self::String(s) => format!("{s} : String"),
                Self::List(cl) => format!("{cl} : List"),
                Primitive::Lambda(Lambda((args, _))) =>
                    format!("λ ({}) (...) : Lambda", args.join(" ")),
            }
        )
    }
}
// #[derive(Debug)]
// pub enum EvalErrType {
//     TypeMismatch(Primitive),
//     TypeCoercion(Primitive),
// }

// #[derive(Debug)]
// pub struct EvalErr {
//     err_type: EvalErrType,
//     msg: String,
//     line_num: u32,
//     col_num: u32,
// }

pub fn add(a: Primitive, b: Primitive) -> Result<Primitive, String> {
    if !a.is_numeric() {
        return Err(format!(
            "(+) expected a Natural or Integer or Number, found {a}"
        ));
    } else if !b.is_numeric() {
        return Err(format!(
            "(+) expected a Natural or Integer or Number, found {b}"
        ));
    }

    if matches!(a, Primitive::Number(_)) || matches!(b, Primitive::Number(_)) {
        Ok(Primitive::Number(a.coerce_f32()? + b.coerce_f32()?))
    } else if matches!(a, Primitive::Integer(_)) || matches!(b, Primitive::Integer(_)) {
        Ok(Primitive::Integer(a.coerce_i32()? + b.coerce_i32()?))
    } else {
        Ok(Primitive::Natural(a.coerce_u32()? + b.coerce_u32()?))
    }
}

pub fn sub(a: Primitive, b: Primitive) -> Result<Primitive, String> {
    if !a.is_numeric() {
        return Err(format!(
            "(-) expected a Natural or Integer or Number, found {a}"
        ));
    } else if !b.is_numeric() {
        return Err(format!(
            "(-) expected a Natural or Integer or Number, found {b}"
        ));
    }

    // TODO what should happen when Naturals underflow?
    if matches!(a, Primitive::Number(_)) || matches!(b, Primitive::Number(_)) {
        Ok(Primitive::Number(a.coerce_f32()? - b.coerce_f32()?))
    } else if matches!(a, Primitive::Integer(_)) || matches!(b, Primitive::Integer(_)) {
        Ok(Primitive::Integer(a.coerce_i32()? - b.coerce_i32()?))
    } else {
        Ok(Primitive::Natural(a.coerce_u32()? - b.coerce_u32()?))
    }
}

pub fn mult(a: Primitive, b: Primitive) -> Result<Primitive, String> {
    if !a.is_numeric() {
        return Err(format!(
            "(*) expected a Natural or Integer or Number, found {a}"
        ));
    } else if !b.is_numeric() {
        return Err(format!(
            "(*) expected a Natural or Integer or Number, found {b}"
        ));
    }

    // TODO what should happen when Naturals underflow?
    if matches!(a, Primitive::Number(_)) || matches!(b, Primitive::Number(_)) {
        Ok(Primitive::Number(a.coerce_f32()? * b.coerce_f32()?))
    } else if matches!(a, Primitive::Integer(_)) || matches!(b, Primitive::Integer(_)) {
        Ok(Primitive::Integer(a.coerce_i32()? * b.coerce_i32()?))
    } else {
        Ok(Primitive::Natural(a.coerce_u32()? * b.coerce_u32()?))
    }
}

pub fn divide(a: Primitive, b: Primitive) -> Result<Primitive, String> {
    if !a.is_numeric() {
        return Err(format!(
            "(/) expected a Natural or Integer or Number, found {a}"
        ));
    } else if !b.is_numeric() {
        return Err(format!(
            "(/) expected a Natural or Integer or Number, found {b}"
        ));
    }

    if matches!(a, Primitive::Number(_)) || matches!(b, Primitive::Number(_)) {
        Ok(Primitive::Number(a.coerce_f32()? / b.coerce_f32()?))
    } else if matches!(a, Primitive::Integer(_)) || matches!(b, Primitive::Integer(_)) {
        Ok(Primitive::Integer(a.coerce_i32()? / b.coerce_i32()?))
    } else {
        Ok(Primitive::Natural(a.coerce_u32()? / b.coerce_u32()?))
    }
}

pub fn mod_(a: Primitive, b: Primitive) -> Result<Primitive, String> {
    match &(a, b) {
        (Primitive::Natural(n), Primitive::Natural(d)) => Ok(Primitive::Natural(n % d)),
        (a2, b2) => Err(format!("`mod` expects two Naturals, got {a2} and {b2}")),
    }
}

pub fn and(a: Primitive, b: Primitive) -> Result<Primitive, String> {
    match (&a, b) {
        (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(*a && b)),
        _ => Err(format!("`and` expected a Boolean, found {a}")),
    }
}

pub fn or(a: Primitive, b: Primitive) -> Result<Primitive, String> {
    match (&a, b) {
        (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(*a || b)),
        _ => Err(format!("`or` expected a Boolean, found {a}")),
    }
}

pub fn not(a: Primitive) -> Result<Primitive, String> {
    match a {
        Primitive::Boolean(a) => Ok(Primitive::Boolean(!a)),
        _ => Err(format!("`not` expected a Boolean, found {a}")),
    }
}

pub fn cons(e: Primitive, r: Primitive) -> Result<Primitive, String> {
    match r {
        Primitive::List(cl) => Ok(Primitive::List(ConsList::Cons((Box::new(e), Box::new(cl))))),
        _ => Err(format!(
            "second argument to `cons` must be a List, given {r}"
        )),
    }
}

pub fn rest(l: Primitive) -> Result<Primitive, String> {
    match l {
        Primitive::List(ConsList::Cons((_, r))) => Ok(Primitive::List(*r)),
        _ => Err(format!("`rest` expects a non-empty List, given {l}")),
    }
}

pub fn first(l: Primitive) -> Result<Primitive, String> {
    match l {
        Primitive::List(ConsList::Cons((e, _))) => Ok(*e),
        _ => Err(format!("`first` expects a non-empty List, given {l}")),
    }
}

pub fn length(l: Primitive) -> Result<Primitive, String> {
    match l {
        Primitive::List(c) => Ok(Primitive::Natural(c.length())),
        Primitive::String(s) => Ok(Primitive::Natural(s.len() as u32)),
        _ => Err(format!("`length` expects a List or String, given {l}")),
    }
}
