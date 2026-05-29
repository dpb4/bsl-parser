use std::{collections::HashMap, rc::Rc};

use crate::{
    primitive::{Lambda, Primitive},
    Expression, Keyword,
};
macro_rules! unpack_args {
    ($vec:expr => $($name:ident),+ $(,)?) => {
        let mut __iter = $vec.into_iter();
        $(
            let $name = __iter.next().unwrap();
        )+
    };
}
macro_rules! check_arg_len {
    ($exprs:expr, $name:expr, $count:expr) => {
        if $exprs.len() != $count {
            return Err(format!(
                "`{n}` expects $count arguments, given {e}",
                n = $name,
                e = $exprs.len()
            ));
        }
    };
}

pub struct EvalEnv {
    parent: Option<Rc<EvalEnv>>,
    entries: HashMap<String, Primitive>,
}

impl EvalEnv {
    pub fn get(&self, name: &str) -> Result<Primitive, String> {
        if let Some(v) = self.entries.get(name) {
            Ok(v.clone())
        } else {
            match self.parent.as_ref() {
                Some(parent) => parent.get(name),
                None => return Err(format!("'{}' is undefined in the current scope", name)),
            }
        }
    }

    pub fn new() -> Self {
        Self {
            parent: None,
            entries: HashMap::new(),
        }
    }

    pub fn new_child(self) -> Self {
        Self {
            parent: Some(Rc::new(self)),
            entries: HashMap::new(),
        }
    }

    pub fn new_child_with(self, env: HashMap<String, Primitive>) -> Self {
        Self {
            parent: Some(Rc::new(self)),
            entries: env,
        }
    }
}

pub fn eval_expression(expr: Expression) -> Result<Primitive, String> {
    let env = EvalEnv::new();
    eval_expression_with_env(expr, Rc::new(env))
}
pub fn eval_expression_with_env(expr: Expression, env: Rc<EvalEnv>) -> Result<Primitive, String> {
    match expr {
        Expression::Literal(primitive) => Ok(primitive),
        Expression::Token(t) => env.get(&t),
        Expression::FunctionCall((name, exprs)) => {
            use Keyword as K;
            match name {
                crate::FunctionName::BuiltIn(keyword) => match keyword {
                    K::If | K::And | K::Or => {}
                    _ => todo!(),
                },
                crate::FunctionName::Custom(name) => {
                    let args: Vec<Primitive> = exprs
                        .into_iter()
                        .map(|e| eval_expression_with_env(e, env.clone()))
                        .collect::<Result<Vec<Primitive>, String>>()?;
                    let lambda = match env.get(&name)? {
                        Primitive::Lambda(lambda) => Ok(lambda),
                        _ => Err("not a lambda"),
                    }?;

                    let (arg_map, body) = apply_lambda(lambda, args)?;
                    let inner_env = EvalEnv {
                        parent: Some(env),
                        entries: arg_map,
                    };
                    eval_expression_with_env(body, Rc::new(inner_env))
                }
            }
        }
        Expression::Cond(_) => todo!(),
    }
}

fn apply_lambda(
    lambda: Lambda,
    args: Vec<Primitive>,
) -> Result<(HashMap<String, Primitive>, Expression), String> {
    let (params, body) = lambda.0;

    if params.len() != args.len() {
        return Err("arg length mismatch".into());
    }

    let arg_map: HashMap<_, _> = params.into_iter().zip(args.into_iter()).collect();
    Ok((arg_map, *body))
}

mod builtin {
    use super::*;

    fn _if(exprs: Vec<Expression>, env: Rc<EvalEnv>) -> Result<Primitive, String> {
        check_arg_len!(exprs, "if", 3);

        unpack_args!(exprs => bool_expr, true_answer, false_answer);

        match eval_expression_with_env(bool_expr, env.clone()) {
            Ok(Primitive::Boolean(m)) => {
                if m {
                    eval_expression_with_env(true_answer, env)
                } else {
                    eval_expression_with_env(false_answer, env)
                }
            }
            Ok(p) => Err(format!(
                "first argument to `if` must be a Boolean (given {p})"
            )),
            Err(e) => Err(e),
        }
    }

    fn or(exprs: Vec<Expression>, env: Rc<EvalEnv>) -> Result<Primitive, String> {
        check_arg_len!(exprs, "or", 2);
        unpack_args!(exprs => bool_a, bool_b);
        todo!()
    }
}
