use std::{io::BufRead, rc::Rc};

use bsl_parser::{
    eval::{eval_top_level_expression, eval_top_level_expression_with_env, EvalEnv},
    parse::Parser,
    *,
};
fn main() {
    let stdin = std::io::stdin();
    let mut env = Rc::new(EvalEnv::new());
    for line in stdin.lock().lines() {
        let content = line.unwrap();

        let parsed = parse_top_level_expression().parse(&content);

        if let Err(e) = parsed {
            println!("parsing error: {e}\n");
            continue;
        }

        let evaled = eval_top_level_expression_with_env(parsed.unwrap().1, env.clone());

        if let Err(e) = evaled {
            println!("evaluation error: {e}\n");
            continue;
        }

        let (res, new_env) = evaled.unwrap();

        if let Some(p) = res {
            println!("> {}\n", p);
        } else {
            println!("> ()\n")
        }

        env = new_env;
    }
    // bsl_parser::testing();
}
