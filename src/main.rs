use std::collections::HashMap;
use serde_json::json;
use std::fs;
use std::path::Path;
use syn::{Item, Visibility, parse_file};
use quote::ToTokens;
use clap::{Arg, Command};
use colored::*;

fn main() {
    let matches = Command::new("Rust Codebase Scanner")
        .version("1.0")
        .author("Your Name")
        .about("Scans a Rust codebase and provides information about public items")
        .arg(Arg::new("path")
            .long("path")
            .value_name("PATH")
            .help("Sets the path to the Rust codebase")
            .required(true))
        .arg(Arg::new("query")
            .long("query")
            .value_name("QUERY")
            .help("Sets the query to filter results (e.g., fn.pub, struct.pub)")
            .required(false))
        .get_matches();

    if let Some(path) = matches.get_one::<String>("path") {
		let path = path.to_string();
		let query = matches.get_one::<String>("query").map(|s| s.as_str()).unwrap_or("all");

		let mut public_items = HashMap::new();
		process_directory(Path::new(&path), &mut public_items);

		display_results(&public_items, query);
	} else {
		eprintln!("Error: Path argument is required");
		std::process::exit(1);
	}
}

fn display_results(public_items: &HashMap<String, HashMap<String, Vec<HashMap<String, String>>>>, query: &str) {
    let queries: Vec<&str> = query.split(',').collect();

    for (file, items) in public_items {
        let mut file_printed = false;

        for (item_type, elements) in items {
            if queries.contains(&"all") || queries.iter().any(|q| q.starts_with(item_type.to_lowercase().as_str())) {
                if !file_printed {
                    println!("{}", file.bold());
                    file_printed = true;
                }

                println!("  {}:", item_type.green());
                for element in elements {
                    let name = element.get("name").unwrap();
					let default = "N/A".to_string();
                    let line = element.get("line").unwrap_or(&default);
                    let line_info = format!("(line: {})", line);
                    let dimmed_line_info = line_info.dimmed();
                    println!("    {:<30} {}", name, dimmed_line_info);

                    if let Some(fields) = element.get("fields") {
                        println!("      {}", "Fields:".italic());
                        let fields: serde_json::Value = serde_json::from_str(fields).unwrap();
                        for (field_name, field_value) in fields.as_object().unwrap() {
                            println!("        {:<20} : {}", field_name, field_value["type"].as_str().unwrap());
                        }
                    }
                }
                println!();
            }
        }
    }
}

fn process_directory(path: &Path, public_items: &mut HashMap<String, HashMap<String, Vec<HashMap<String, String>>>>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();

            if path.is_dir() {
                process_directory(&path, public_items);
            } else if path.extension().map(|s| s == "rs").unwrap_or(false) {
                let content = fs::read_to_string(&path).expect("Failed to read file");
                match parse_file(&content) {
                    Ok(syntax_tree) => {
                        let file_name = path.to_string_lossy().to_string();
                        let file_items = public_items.entry(file_name).or_insert_with(HashMap::new);

                        process_items(&syntax_tree.items, file_items);
                    }
                    Err(err) => {
                        eprintln!("Failed to parse file {}: {}", path.display(), err);
                        continue;
                    }
                }
            }
        }
    }
}

fn process_items(items: &[Item], file_items: &mut HashMap<String, Vec<HashMap<String, String>>>) {
    for item in items {
        let line = match item {
            Item::Fn(item_fn) => item_fn.sig.ident.span(),
            Item::Struct(item_struct) => item_struct.ident.span(),
            Item::Enum(item_enum) => item_enum.ident.span(),
            Item::Trait(item_trait) => item_trait.ident.span(),
            _ => continue,
        };
        let line_number = line.start().line.to_string();

        match item {
            Item::Fn(item_fn) => {
                if is_public(&item_fn.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Fn".to_string());
                    element.insert("name".to_string(), item_fn.sig.ident.to_string());
                    element.insert("line".to_string(), line_number);
                    file_items.entry("Fn".to_string()).or_insert_with(Vec::new).push(element);
                }
            },
            Item::Struct(item_struct) => {
                if is_public(&item_struct.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Struct".to_string());
                    element.insert("name".to_string(), item_struct.ident.to_string());
                    element.insert("line".to_string(), line_number);

                    let mut fields = HashMap::new();
                    for field in &item_struct.fields {
                        if let Some(ident) = &field.ident {
                            let field_type = field.ty.to_token_stream().to_string();
                            fields.insert(ident.to_string(), json!({"type": field_type}));
                        }
                    }
                    element.insert("fields".to_string(), json!(fields).to_string());
                    file_items.entry("Struct".to_string()).or_insert_with(Vec::new).push(element);
                }
            },
            // Add similar blocks for other item types (Enum, Trait, etc.)
            _ => {}
        }
    }
}

fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}