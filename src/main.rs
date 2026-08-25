use bunlang::interpreter::Interpreter;
use bunlang::lexer::Lexer;
use bunlang::parser::{Item, ItemKind, Parser};
use std::collections::HashMap;
use std::process::exit;
use std::{env, fs};

fn load_program(
    file_path: &str,
    module_prefix: &str,
    resolved: &mut HashMap<String, Item>,
) -> Result<(), String> {
    let code = fs::read_to_string(file_path)
        .map_err(|e| format!("Could not read file '{}': {}", file_path, e))?;
    let tokens =
        Lexer::tokenize(&code).map_err(|e| format!("Lexer error in '{}':\n{}", file_path, e))?;
    let items = Parser::parse(tokens, &code)
        .map_err(|e| format!("Parser error in '{}':\n{}", file_path, e))?;

    resolve_items(items, module_prefix, resolved)
}

fn resolve_items(
    items: Vec<Item>,
    prefix: &str,
    resolved: &mut HashMap<String, Item>,
) -> Result<(), String> {
    let mut use_counter = 0;
    for item in items {
        match &item.value {
            ItemKind::ModImport { is_pub, name, path } => {
                let new_prefix = format!("{}{}::", prefix, name);
                load_program(path, &new_prefix, resolved)?;
            }
            ItemKind::Mod {
                is_pub,
                name,
                items: sub_items,
            } => {
                let new_prefix = format!("{}{}::", prefix, name);
                resolve_items(sub_items.clone(), &new_prefix, resolved)?;
            }
            ItemKind::FnDeclare { name, .. } => {
                let full_name = format!("{}{}", prefix, name);
                resolved.insert(full_name, item);
            }
            ItemKind::Use { .. } => {
                let key = format!("__use_{}_{}", prefix, use_counter);
                resolved.insert(key, item);
                use_counter += 1;
            }
            _ => {}
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: bunlang <file.bn>");
        exit(1);
    }
    let file = &args[1];
    let mut items = HashMap::new();
    if let Err(err) = load_program(file, "", &mut items) {
        println!("{}", err);
        exit(1);
    }
    let main_code = fs::read_to_string(file).unwrap();
    let mut interpreter = Interpreter::new(items, &main_code);
    if let Err(err) = interpreter.run() {
        println!("{}", err);
        exit(1);
    }
}
