use std::io;

mod error;
mod eval;
mod parser;
mod tokenizer;

use error::CalcError;
use eval::{evaluate, Context, Number};
use parser::parse;
use tokenizer::tokenize;

fn eval_str_ctx(s: &str, ctx: &mut Context) -> Result<Number, CalcError> {
    let tokens = tokenize(s)?;
    let ast = parse(&tokens)?;
    let result = evaluate(&ast, ctx)?;
    Ok(result)
}

fn eval_file(path: &str) -> Result<(), CalcError> {
    let mut ctx = Context::new();
    let contents = std::fs::read_to_string(path)?;
    let result = eval_str_ctx(&contents, &mut ctx)?;
    println!("{}", result);
    Ok(())
}

fn repl() {
    // TODO: Implement proper multi-line support
    let mut ctx = Context::new();
    let _stdout = io::stdout();
    let mut input = String::new();

    loop {
        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                input.push_str(line);
                match eval_str_ctx(&input, &mut ctx) {
                    Ok(result) => {
                        println!("{}", result);
                        input.clear();
                    }
                    // Artifact from previous band aid multi-line support
                    // input incomplete => {
                    //     input.push('\n');
                    //     print!("> ");
                    //     stdout.lock().flush().expect("Failed to flush stdout");
                    // }
                    Err(err) => {
                        eprintln!("{}", err);
                        input.clear();
                    }
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if let Err(err) = eval_file(&args[1]) {
            eprintln!("{}", err);
        }
        return;
    }

    repl();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_str(s: &str) -> Result<Number, CalcError> {
        eval_str_ctx(s, &mut Context::new())
    }

    #[test]
    fn test_eval_str() {
        assert!(eval_str("").is_ok());
        assert!(eval_str("-").is_err());
        assert!(eval_str("* 2").is_err());
        assert!(eval_str("2 +").is_err());
        assert_eq!(eval_str("2").unwrap(), 2.0);
        assert_eq!(eval_str("2 - 3").unwrap(), -1.0);
        assert_eq!(eval_str("2-3").unwrap(), -1.0);
        assert_eq!(eval_str("2 + 2 * 2").unwrap(), 6.0);
        assert_eq!(eval_str("3 * 2 * 5 + 10 / 5 - 8").unwrap(), 24.0);
    }

    #[test]
    fn test_number_parsing() {
        assert!(eval_str(".1").is_ok());
        assert!(eval_str("1.1").is_ok());
        assert!(eval_str("1.").is_ok());

        assert!(eval_str("2.3.4").is_err());
        assert!(eval_str("..").is_err());
        assert!(eval_str("..1").is_err());
        assert!(eval_str("1..").is_err());
        assert!(eval_str(".1.").is_err());
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(eval_str("-2").unwrap(), -2.0);
        assert_eq!(eval_str("2--2").unwrap(), 4.0);
        assert_eq!(eval_str("2+-2").unwrap(), 0.0);
        assert_eq!(eval_str("-2+-2").unwrap(), -4.0);
        assert_eq!(eval_str("2---2").unwrap(), 0.0);
        assert!(eval_str("2*+-2").is_err());
    }

    #[test]
    fn test_brackets() {
        assert_eq!(eval_str("4 * (5 - 1)").unwrap(), 16.0);
        assert_eq!(eval_str("(2 + 2) * (3 + 3)").unwrap(), 24.0);
        assert_eq!(eval_str("(2 + 2)").unwrap(), 4.0);
        assert_eq!(eval_str("-(2 + 2)").unwrap(), -4.0);
        assert_eq!(eval_str("-((2 + 3) * 4)").unwrap(), -20.0);
        assert_eq!(eval_str("-((2 + -4) * 5) / 2").unwrap(), 5.0);
        assert_eq!(eval_str("(1 + 2) + 3").unwrap(), 6.0);
        assert!(eval_str("-2 + 2)").is_err());
        assert!(eval_str("-(2 + 2").is_err());
        assert!(eval_str("()").is_err());
    }

    #[test]
    fn test_power() {
        assert!(eval_str("4 ^").is_err());
        assert!(eval_str("^ 3").is_err());
        assert_eq!(eval_str("1 ^ -3").unwrap(), 1.0);
        assert_eq!(eval_str("(-1) ^ -3").unwrap(), -1.0);
        assert_eq!(eval_str("(-1) ^ -4").unwrap(), 1.0);
        assert_eq!(eval_str("2 ^ -3").unwrap(), 0.125);
        assert_eq!(eval_str("2 ^ 0").unwrap(), 1.0);
        assert_eq!(eval_str("3 ^ 5").unwrap(), 243.0);
        assert_eq!(eval_str("-1 ^ 4").unwrap(), 1.0);
        assert_eq!(eval_str("-1 ^ 5").unwrap(), -1.0);
        assert_eq!(eval_str("-1 ^ -5").unwrap(), -1.0);
        assert_eq!(eval_str("(1 + 1) ^ (4 * 2)").unwrap(), 256.0);
    }

    #[test]
    fn test_mod() {
        assert!(eval_str("2 %").is_err());
        assert!(eval_str("% 3").is_err());
        assert!(eval_str("100 % 0").is_err());
        assert_eq!(eval_str("7 % 3").unwrap(), 1.0);
        assert_eq!(eval_str("7 % -3").unwrap(), 1.0);
        assert_eq!(eval_str("-7 % 3").unwrap(), -1.0);
        assert_eq!(eval_str("-9 % -3").unwrap(), 0.0);
        assert_eq!(eval_str("42 % 1337").unwrap(), 42.0);
        assert_eq!(eval_str("2 + 3 * 4 % 5").unwrap(), 4.0);
    }

    #[test]
    fn test_variables() {
        let mut ctx = Context::new();
        assert_eq!(eval_str_ctx("a = 2", &mut ctx).unwrap(), 2.0);
        assert_eq!(eval_str_ctx("b = a + 1", &mut ctx).unwrap(), 3.0);
        assert_eq!(eval_str_ctx("c = a + b", &mut ctx).unwrap(), 5.0);
        assert_eq!(ctx.get_var("a"), Some(2.0));
        assert_eq!(ctx.get_var("b"), Some(3.0));
        assert_eq!(ctx.get_var("c"), Some(5.0));

        assert!(eval_str("not_defined").is_err());

        let mut ctx = Context::new();
        assert_eq!(eval_str_ctx("some_longer_name = 2", &mut ctx).unwrap(), 2.0);
        assert_eq!(ctx.get_var("some_longer_name"), Some(2.0));

        assert!(eval_str("a b = 2").is_err());
        assert!(eval_str("2 = 2").is_err());
        assert!(eval_str("* = 2").is_err());
        assert!(eval_str("() = 2").is_err());
    }

    #[test]
    fn test_builtin_functions() {
        use std::f64::consts;

        let eps = 1e-10;
        assert!((eval_str("sin(pi/2)").unwrap() - 1.0).abs() < eps);
        assert!((eval_str("cos(pi/2)").unwrap() - 0.0).abs() < eps);
        assert!((eval_str("tan(pi/4)").unwrap() - 1.0).abs() < eps);
        assert!((eval_str("asin(1)").unwrap() - consts::FRAC_PI_2).abs() < eps);
        assert!((eval_str("acos(1)").unwrap() - 0.0).abs() < eps);
        assert!((eval_str("atan(1)").unwrap() - consts::FRAC_PI_4).abs() < eps);
        assert!((eval_str("sinh(1)").unwrap() - 1_f64.sinh()).abs() < eps);
        assert!((eval_str("cosh(1)").unwrap() - 1_f64.cosh()).abs() < eps);
        assert!((eval_str("tanh(1)").unwrap() - 1_f64.tanh()).abs() < eps);

        assert!((eval_str("ln(e)").unwrap() - 1.0).abs() < eps);
        assert!((eval_str("log2(1024)").unwrap() - 10.0).abs() < eps);
        assert!((eval_str("log10(1000)").unwrap() - 3.0).abs() < eps);
        assert!((eval_str("log(27, 3)").unwrap() - 3.0).abs() < eps);

        assert!((eval_str("abs(-1)").unwrap() - 1.0).abs() < eps);
        assert!((eval_str("abs(1)").unwrap() - 1.0).abs() < eps);
        assert!((eval_str("min(1, 5)").unwrap() - 1.0).abs() < eps);
        assert!((eval_str("max(1, 5)").unwrap() - 5.0).abs() < eps);
        assert!((eval_str("floor(1.5)").unwrap() - 1.0).abs() < eps);
        assert!((eval_str("ceil(1.5)").unwrap() - 2.0).abs() < eps);
        assert!((eval_str("round(1.5)").unwrap() - 2.0).abs() < eps);
        assert!((eval_str("round(1.4)").unwrap() - 1.0).abs() < eps);
        assert!((eval_str("round(1.6)").unwrap() - 2.0).abs() < eps);

        assert!(eval_str("sqrt(-1)").is_err());
        assert!((eval_str("sqrt(4)").unwrap() - 2.0).abs() < eps);
        assert!((eval_str("exp(2)").unwrap() - 7.389056099).abs() < eps);
    }

    #[test]
    fn test_functions() {
        use crate::eval::Function;

        let mut ctx = Context::new();
        ctx.add_function(
            "add",
            Function::new_builtin(2, |_ctx, args| args[0] + args[1]),
        )
        .unwrap();

        assert!(eval_str_ctx("add()", &mut ctx).is_err());
        assert!(eval_str_ctx("add(1)", &mut ctx).is_err());
        assert!(eval_str_ctx("add(1,)", &mut ctx).is_err());
        assert!(eval_str_ctx("add(,1)", &mut ctx).is_err());
        assert!(eval_str_ctx("add(1 1)", &mut ctx).is_err());
        assert_eq!(eval_str_ctx("add(1, 2)", &mut ctx).unwrap(), 3.0);
        assert!(eval_str_ctx("add(1, 2, 3)", &mut ctx).is_err());
        assert_eq!(eval_str_ctx("add(1, add(2, 3))", &mut ctx).unwrap(), 6.0);
    }

    #[test]
    fn test_multiple_lines() {
        let mut ctx = Context::new();

        let result = eval_str_ctx(
            r"a = 2
            b = 3
            c = a + b",
            &mut ctx,
        )
        .unwrap();

        assert_eq!(result, 5.0);
        assert_eq!(ctx.get_var("a"), Some(2.0));
        assert_eq!(ctx.get_var("b"), Some(3.0));
        assert_eq!(ctx.get_var("c"), Some(5.0));

        assert_eq!(eval_str("\n42\n").unwrap(), 42.0);
        assert_eq!(eval_str("42\n").unwrap(), 42.0);
        assert_eq!(eval_str("\n42").unwrap(), 42.0);
        assert_eq!(eval_str("\n\n\n").unwrap(), 0.0);
    }

    #[test]
    fn test_newlines_not_allowed() {
        assert!(eval_str("1 + \n 2").is_err());
        assert!(eval_str("sin(pi\n/2)").is_err());
        assert!(eval_str("sin(\npi/2)").is_err());
        assert!(eval_str("1 * (2 + \n 3)").is_err());
        assert!(eval_str("a = \n2").is_err());
    }

    #[test]
    fn test_user_functions() {
        let code = "\
            fn add(a, b, c) {\n\
                a + b + c\n\
            }\n\
            \n\
            fn sub(a, b) {\n\
                a - b\n\
            }\n\
            \n\
            sub(42, add(1, 2, 3))";
        assert_eq!(eval_str(code).unwrap(), 36.0);

        assert!(eval_str("fn add(a, {b) a + b }").is_err());
        assert!(eval_str("fn empty_body() {}").is_ok());
        assert!(eval_str("fn no_args() {\n inspect(1)\n }").is_ok());
        assert!(eval_str("fn one_liner(a, b) { a + b }").is_ok());
        assert!(eval_str("fn trailing_comma(a, b,) { a + b }").is_err());
        assert!(eval_str("fn leading_comma(, a, b) { a + b }").is_err());
        assert!(eval_str("fn no_comma(a b) { a + b }").is_err());
        assert!(eval_str("fn contains_expression(a, b, 1 + 1) { a + b }").is_err());
        assert!(eval_str("fn duplicate_arg_name(a, a) { a + a }").is_err());
    }

    #[test]
    fn test_if_statements() {
        let code = "\
            a = 0
            if (0) {
                a = 2
            }
            a";
        assert_eq!(eval_str(code).unwrap(), 0.0);

        let code = "\
            a = 0
            if (1) {
                a = 2
            }
            a";
        assert_eq!(eval_str(code).unwrap(), 2.0);

        let code = "\
            a = 0
            if (0) {
                a = 2
            } else {
                a = 3
            }
            a";
        assert_eq!(eval_str(code).unwrap(), 3.0);

        let code = "\
            a = 0
            if (1) {
                a = 2
            } else {
                a = 3
            }
            a";
        assert_eq!(eval_str(code).unwrap(), 2.0);
    }

    #[test]
    fn test_errors_on_missing_newline() {
        assert!(eval_str("1 + 1 2 + 2").is_err());
        assert!(eval_str("1 2").is_err());
        assert!(eval_str("(1 * 3) 2").is_err());

        assert!(eval_str("fn add(a, b) { a + b } fn sub(a, b) { a - b }").is_err());
        assert!(eval_str("if (1){ 1 } if (2){ 2 }").is_err());
    }
}
