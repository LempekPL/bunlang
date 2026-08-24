use crate::environment::Environment;
use crate::value::Value;

pub fn import_builtins(env: &mut Environment) {
    env.define("io::println", Value::Native(bun_println));
    env.define("io::print", Value::Native(bun_print));
    env.define("io::read", Value::Native(bun_read));
}

pub fn bun_println(args: Vec<Value>) -> Result<Value, String> {
    for arg in args {
        print!("{}", arg);
    }
    println!();
    use std::io::Write;
    std::io::stdout().flush().unwrap();
    Ok(Value::Unit)
}

pub fn bun_print(args: Vec<Value>) -> Result<Value, String> {
    for arg in args {
        print!("{}", arg);
    }
    use std::io::Write;
    std::io::stdout().flush().unwrap();
    Ok(Value::Unit)
}

pub fn bun_read(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!(
            "Function 'read' expects 0 arguments got {}",
            args.len()
        ));
    }
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(Value::String(
        input
            .trim_end_matches(|c| c == '\r' || c == '\n')
            .to_string(),
    ))
}
