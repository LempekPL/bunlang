use bunlang::lexer::Lexer;
use bunlang::parser::{Item, ItemKind, Parser};
use std::{env, fs};
use std::collections::HashMap;
use std::process::exit;
use bunlang::interpreter::Interpreter;
use bunlang::value::Value;

fn load_program(
    file_path: &str,
    module_prefix: &str
) -> Result<HashMap<String, Item>, String> {
    let code = fs::read_to_string(file_path)
        .map_err(|e| format!("Could not read file '{}': {}", file_path, e))?;
    let tokens = Lexer::tokenize(&code)
        .map_err(|e| format!("Lexer error in '{}':\n{}", file_path, e))?;
    let items = Parser::parse(tokens, &code)
        .map_err(|e| format!("Parser error in '{}':\n{}", file_path, e))?;
    let mut resolved_items = HashMap::new();
    for (name, item) in items {
        match &item.value {
            ItemKind::ModImport { name: mod_name, path } => {
                let new_prefix = format!("{}{}::", module_prefix, mod_name);
                let sub_items = load_program(path, &new_prefix)?;
                resolved_items.extend(sub_items);
            }
            ItemKind::FnDeclare { name: fn_name, .. } => {
                let full_name = format!("{}{}", module_prefix, fn_name);
                resolved_items.insert(full_name, item);
            }
            ItemKind::Use { .. } => {
                resolved_items.insert(name, item);
            }
            _ => {}
        }
    }

    Ok(resolved_items)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: bunlang <file.bn>");
        exit(1);
    }
    let file = &args[1];
    let items = match load_program(file, "") {
        Ok(items) => items,
        Err(err) => {
            println!("{}", err);
            exit(1);
        }
    };
    let main_code = fs::read_to_string(file).unwrap();

    let mut interpreter = Interpreter::new(items, &main_code);
    if let Err(err) = interpreter.run() {
        println!("{}", err);
        exit(1);
    }
}
