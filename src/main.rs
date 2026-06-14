use std::{
    io::{BufRead, Read},
    rc::Rc,
};

use bsl_parser::{
    eval::{eval_top_level_expression_with_env, EvalEnv},
    parse::{com, par, Parser},
    *,
};
fn main() {
    // testing();

    let args: Vec<String> = std::env::args().collect();
    let mut env = Rc::new(EvalEnv::new());
    if args.len() > 1 {
        let mut file = if let Ok(f) = std::fs::File::open(&args[1]) {
            f
        } else {
            panic!("input file not found")
        };
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);

        let parsed = com::one_plus(par::maybe_space_then(parse_top_level_expression()))
            .parse((contents[..]).into());
        if let Err(e) = parsed {
            println!("parsing error: \n{e}");
        } else {
            dbg!(&parsed);
            for d in parsed.unwrap().1 {
                match eval_top_level_expression_with_env(d, env.clone()) {
                    Ok((output, new_env)) => {
                        env = new_env;
                        if let Some(p) = output {
                            println!("{p}")
                        }
                    }
                    Err(e) => {
                        println!("error while evaluating:\n{e}");
                    }
                }
            }
        }

        println!("done evaluating file, entering repl\n");
    }

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        // println!("> ");
        let content = line.unwrap();

        let parsed = parse_top_level_expression().parse((content[..]).into());

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
            println!("= {}\n", p);
        } else {
            println!("= ()\n")
        }

        env = new_env;
    }
}
