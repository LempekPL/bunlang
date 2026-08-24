use bunlang::lexer::Lexer;
use bunlang::parser::Parser;
use std::fs;
use std::process::exit;
use bunlang::interpreter::Interpreter;
use bunlang::value::Value;

fn main() {
    let code = fs::read_to_string("main.bn").unwrap();
    let tokens = Lexer::tokenize(&code);
    if let Err(err) = tokens {
        println!("{}", err);
        exit(0);
    }
    // dbg!(&tokens.clone().unwrap());
    let expr = Parser::parse(tokens.unwrap(), &code);
    if let Err(err) = expr {
        println!("{}", err);
        exit(0);
    }
    // dbg!(&expr);
    let response = Interpreter::run(expr.unwrap(), &code);
    if let Err(err) = response {
        println!("{}", err);
        exit(0);
    }
    // if let Ok(res) = response && res != Value::Unit {
    //     println!("{:?}", res);
    // }
}
