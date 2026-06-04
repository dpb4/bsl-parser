use std::{collections::HashMap, rc::Rc};

use crate::{
    primitive::{self, Lambda, Primitive},
    Expression, FunctionName, Keyword, TopLevelExpression,
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

#[derive(Debug, Clone)]
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

pub fn eval_nv_expression(expr: Expression) -> Result<Primitive, String> {
    let env = EvalEnv::new();
    eval_nv_expression_with_env(expr, Rc::new(env))
}
pub fn eval_nv_expression_with_env(
    expr: Expression,
    env: Rc<EvalEnv>,
) -> Result<Primitive, String> {
    match expr {
        Expression::Literal(primitive) => Ok(primitive),
        Expression::Identifier(t) => env.get(&t),
        Expression::Cond((cases, default)) => {
            for (question, answer) in cases {
                match eval_nv_expression_with_env(question, env.clone()) {
                    Ok(Primitive::Boolean(m)) => {
                        if m {
                            return eval_nv_expression_with_env(answer, env);
                        } else {
                            continue;
                        }
                    }
                    Ok(p) => {
                        return Err(format!("`cond` question must be a Boolean (given {p})"));
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            eval_nv_expression_with_env(*default, env)
        }
        Expression::FunctionCall((name, exprs)) => {
            use Keyword as K;
            match name {
                crate::FunctionName::BuiltIn(keyword) => {
                    match keyword {
                        K::If => {
                            check_arg_len!(exprs, "if", 3);
                            unpack_args!(exprs => bool_expr, true_answer, false_answer);
                            match eval_nv_expression_with_env(bool_expr, env.clone()) {
                                Ok(Primitive::Boolean(m)) => {
                                    if m {
                                        eval_nv_expression_with_env(true_answer, env)
                                    } else {
                                        eval_nv_expression_with_env(false_answer, env)
                                    }
                                }
                                Ok(p) => Err(format!(
                                    "first argument to `if` must be a Boolean (given {p})"
                                )),
                                Err(e) => Err(e),
                            }
                        }
                        K::And => {
                            check_arg_len!(exprs, "and", 2);
                            unpack_args!(exprs => bool_a, bool_b);

                            match eval_nv_expression_with_env(bool_a, env.clone()) {
                                Ok(Primitive::Boolean(m)) => {
                                    if !m {
                                        Ok(Primitive::Boolean(false))
                                    } else {
                                        match eval_nv_expression_with_env(bool_b, env) {
                                            b @ Ok(Primitive::Boolean(_)) => b,
                                            Ok(p) => Err(format!("second argument to `and` must be a Boolean (given {p})")),
                                            e @ _ => e,
                                        }
                                    }
                                }
                                Ok(p) => Err(format!(
                                    "first argument to `and` must be a Boolean (given {p})"
                                )),
                                e @ _ => e,
                            }
                        }
                        K::Or => {
                            check_arg_len!(exprs, "or", 2);
                            unpack_args!(exprs => bool_a, bool_b);

                            match eval_nv_expression_with_env(bool_a, env.clone()) {
                                Ok(Primitive::Boolean(m)) => {
                                    if m {
                                        Ok(Primitive::Boolean(true))
                                    } else {
                                        match eval_nv_expression_with_env(bool_b, env) {
                                        b @ Ok(Primitive::Boolean(_)) => b,
                                        Ok(p) => Err(format!("second argument to `or` must be a Boolean (given {p})")),
                                        e @ _ => e,
                                    }
                                    }
                                }
                                Ok(p) => Err(format!(
                                    "first argument to `or` must be a Boolean (given {p})"
                                )),
                                e @ _ => e,
                            }
                        }
                        K::Plus => {
                            check_arg_len!(exprs, "(+)", 2);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a, b);

                            primitive::add(a?, b?)
                        }
                        K::Minus => {
                            check_arg_len!(exprs, "(-)", 2);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a, b);

                            primitive::sub(a?, b?)
                        }
                        K::Times => {
                            check_arg_len!(exprs, "(*)", 2);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a, b);

                            primitive::mult(a?, b?)
                        }

                        K::Divide => {
                            check_arg_len!(exprs, "(/)", 2);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a, b);

                            primitive::divide(a?, b?)
                        }
                        K::Equals => {
                            check_arg_len!(exprs, "(/)", 2);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a, b);

                            Ok(Primitive::Boolean(a? == b?))
                        }
                        K::Not => {
                            check_arg_len!(exprs, "not", 1);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a);

                            primitive::not(a?)
                        }
                        K::Cons => {
                            check_arg_len!(exprs, "cons", 2);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a, b);

                            primitive::cons(a?, b?)
                        }
                        K::First => {
                            check_arg_len!(exprs, "first", 1);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a);

                            primitive::first(a?)
                        }
                        K::Rest => {
                            check_arg_len!(exprs, "rest", 1);
                            let x = exprs
                                .into_iter()
                                .map(|e| eval_nv_expression_with_env(e, env.clone()));
                            unpack_args!(x => a);

                            primitive::rest(a?)
                        }
                        K::Cond => todo!(),
                        K::Define => todo!(),
                        K::Local => todo!(),
                        K::List => todo!(),
                        K::CheckExpect => todo!(),
                    }
                }
                crate::FunctionName::Custom(name) => {
                    let args: Vec<Primitive> = exprs
                        .into_iter()
                        .map(|e| eval_nv_expression_with_env(e, env.clone()))
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
                    eval_nv_expression_with_env(body, Rc::new(inner_env))
                }
            }
        }
    }
}

pub fn eval_top_level_expression(
    expr: TopLevelExpression,
) -> Result<(Option<Primitive>, Rc<EvalEnv>), String> {
    eval_top_level_expression_with_env(expr, Rc::new(EvalEnv::new()))
}

pub fn eval_top_level_expression_with_env(
    expr: TopLevelExpression,
    env: Rc<EvalEnv>,
) -> Result<(Option<Primitive>, Rc<EvalEnv>), String> {
    match expr {
        TopLevelExpression::ConstantDefinition((name, value)) => {
            let mut env2 = (*env).clone();
            env2.entries
                .insert(name, eval_nv_expression_with_env(value, env)?);
            Ok((None, Rc::new(env2)))
        }
        TopLevelExpression::FunctionDefinition((FunctionName::Custom(name), args, body)) => {
            let mut env2 = (*env).clone();
            env2.entries
                .insert(name, Primitive::Lambda(Lambda((args, Box::new(body)))));
            Ok((None, Rc::new(env2)))
        }
        TopLevelExpression::FunctionDefinition((FunctionName::BuiltIn(n), _, _)) => Err(format!(
            "cannot define a function with name {:?}, that is a built in function",
            n
        )),
        TopLevelExpression::NonVoidExpression(expr) => {
            Ok((Some(eval_nv_expression_with_env(expr, env.clone())?), env))
        }
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
