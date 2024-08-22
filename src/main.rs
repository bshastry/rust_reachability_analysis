use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{Item, Visibility, ImplItem};
use quote::ToTokens; // Correct import for ToTokens trait

fn main() {
    let path = "../lighthouse"; // Change this to your codebase path
    let mut public_items = HashMap::new();

    process_directory(Path::new(path), &mut public_items);

    // Output the results
    for (file, items) in &public_items {
        println!("File: {}", file);
        for (item_type, elements) in items {
            println!("  {}:", item_type);
            for element in elements {
                println!("    - {}: {}", element.get("name").unwrap(), element.get("type").unwrap());
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
                match syn::parse_file(&content) {
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
        match item {
            Item::Const(item_const) => {
                if is_public(&item_const.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Const".to_string());
                    element.insert("name".to_string(), item_const.ident.to_string());
                    file_items.entry("Const".to_string()).or_insert_with(Vec::new).push(element);
                }
            }
            Item::Enum(item_enum) => {
                if is_public(&item_enum.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Enum".to_string());
                    element.insert("name".to_string(), item_enum.ident.to_string());
                    file_items.entry("Enum".to_string()).or_insert_with(Vec::new).push(element);
                }
                for variant in &item_enum.variants {
                    process_items(&variant.fields.iter().map(|f| Item::Verbatim(f.to_token_stream())).collect::<Vec<_>>(), file_items);
                }
            }
            Item::ExternCrate(item_extern_crate) => {
                if is_public(&item_extern_crate.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "ExternCrate".to_string());
                    element.insert("name".to_string(), item_extern_crate.ident.to_string());
                    file_items.entry("ExternCrate".to_string()).or_insert_with(Vec::new).push(element);
                }
            }
            Item::Fn(item_fn) => {
                if is_public(&item_fn.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Fn".to_string());
                    element.insert("name".to_string(), item_fn.sig.ident.to_string());
                    file_items.entry("Fn".to_string()).or_insert_with(Vec::new).push(element);
                }
            }
            Item::ForeignMod(item_foreign_mod) => {
                for item in &item_foreign_mod.items {
                    if let syn::ForeignItem::Fn(item_fn) = item {
                        if is_public(&item_fn.vis) {
                            let mut element = HashMap::new();
                            element.insert("type".to_string(), "ForeignFn".to_string());
                            element.insert("name".to_string(), item_fn.sig.ident.to_string());
                            file_items.entry("ForeignFn".to_string()).or_insert_with(Vec::new).push(element);
                        }
                    }
                }
            }
            Item::Impl(item_impl) => {
                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    for item in &item_impl.items {
                        if let ImplItem::Fn(method) = item {
                            if is_public(&method.vis) {
                                let mut element = HashMap::new();
                                element.insert("type".to_string(), "TraitFn".to_string());
                                element.insert("name".to_string(), format!("{}::{}", trait_path.segments.last().unwrap().ident, method.sig.ident));
                                file_items.entry("TraitFn".to_string()).or_insert_with(Vec::new).push(element);
                            }
                        }
                    }
                } else {
                    for item in &item_impl.items {
                        if let ImplItem::Fn(method) = item {
                            if is_public(&method.vis) {
                                let mut element = HashMap::new();
                                element.insert("type".to_string(), "ImplFn".to_string());
                                element.insert("name".to_string(), method.sig.ident.to_string());
                                file_items.entry("ImplFn".to_string()).or_insert_with(Vec::new).push(element);
                            }
                        }
                    }
                }
            }
            Item::Macro(item_macro) => {
                let mut element = HashMap::new();
                element.insert("type".to_string(), "Macro".to_string());
                element.insert("name".to_string(), item_macro.mac.path.segments.last().unwrap().ident.to_string());
                file_items.entry("Macro".to_string()).or_insert_with(Vec::new).push(element);
            }
            Item::Mod(item_mod) => {
                if is_public(&item_mod.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Mod".to_string());
                    element.insert("name".to_string(), item_mod.ident.to_string());
                    file_items.entry("Mod".to_string()).or_insert_with(Vec::new).push(element);
                }
                if let Some((_, items)) = &item_mod.content {
                    process_items(items, file_items);
                }
            }
            Item::Static(item_static) => {
                if is_public(&item_static.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Static".to_string());
                    element.insert("name".to_string(), item_static.ident.to_string());
                    file_items.entry("Static".to_string()).or_insert_with(Vec::new).push(element);
                }
            }
            Item::Struct(item_struct) => {
                if is_public(&item_struct.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Struct".to_string());
                    element.insert("name".to_string(), item_struct.ident.to_string());
                    file_items.entry("Struct".to_string()).or_insert_with(Vec::new).push(element);
                }
                process_items(&item_struct.fields.iter().map(|f| Item::Verbatim(f.to_token_stream())).collect::<Vec<_>>(), file_items);
            }
            Item::Trait(item_trait) => {
                if is_public(&item_trait.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Trait".to_string());
                    element.insert("name".to_string(), item_trait.ident.to_string());
                    file_items.entry("Trait".to_string()).or_insert_with(Vec::new).push(element);
                }
                for item in &item_trait.items {
                    if let syn::TraitItem::Fn(method) = item {
                        let mut element = HashMap::new();
                        element.insert("type".to_string(), "TraitFn".to_string());
                        element.insert("name".to_string(), method.sig.ident.to_string());
                        file_items.entry("TraitFn".to_string()).or_insert_with(Vec::new).push(element);
                    }
                }
            }
            Item::TraitAlias(item_trait_alias) => {
                if is_public(&item_trait_alias.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "TraitAlias".to_string());
                    element.insert("name".to_string(), item_trait_alias.ident.to_string());
                    file_items.entry("TraitAlias".to_string()).or_insert_with(Vec::new).push(element);
                }
            }
            Item::Type(item_type) => {
                if is_public(&item_type.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Type".to_string());
                    element.insert("name".to_string(), item_type.ident.to_string());
                    file_items.entry("Type".to_string()).or_insert_with(Vec::new).push(element);
                }
            }
            Item::Union(item_union) => {
                if is_public(&item_union.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Union".to_string());
                    element.insert("name".to_string(), item_union.ident.to_string());
                    file_items.entry("Union".to_string()).or_insert_with(Vec::new).push(element);
                }
                process_items(&item_union.fields.named.iter().map(|f| Item::Verbatim(f.to_token_stream())).collect::<Vec<_>>(), file_items);
            }
            Item::Use(item_use) => {
                if is_public(&item_use.vis) {
                    let mut element = HashMap::new();
                    element.insert("type".to_string(), "Use".to_string());
                    element.insert("name".to_string(), item_use.tree.to_token_stream().to_string().replace(" ", ""));
                    file_items.entry("Use".to_string()).or_insert_with(Vec::new).push(element);
                }
            }
            Item::Verbatim(_) => {
                let mut element = HashMap::new();
                element.insert("type".to_string(), "Verbatim".to_string());
                element.insert("name".to_string(), "Verbatim".to_string());
                file_items.entry("Verbatim".to_string()).or_insert_with(Vec::new).push(element);
            }
            _ => {}
        }
    }
}

fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    fn setup_test_directory() -> TempDir {
        TempDir::new().unwrap()
    }

    fn create_test_file(dir: &Path, file_name: &str, content: &str) {
        let file_path = dir.join(file_name);
        let mut file = File::create(file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_process_directory_with_empty_dir() {
        let test_dir = setup_test_directory();
        let mut public_items = HashMap::new();

        process_directory(test_dir.path(), &mut public_items);

        assert!(public_items.is_empty());
    }

    #[test]
    fn test_process_directory_with_single_file() {
        let test_dir = setup_test_directory();
        create_test_file(test_dir.path(), "lib.rs", "pub fn test_fn() {}");

        let mut public_items = HashMap::new();
        process_directory(test_dir.path(), &mut public_items);

        assert_eq!(public_items.len(), 1);
        let file_items = public_items.get(&format!("{}/lib.rs", test_dir.path().display())).unwrap();
        assert_eq!(file_items.get("Fn").unwrap().len(), 1);
        assert_eq!(file_items.get("Fn").unwrap()[0].get("name").unwrap(), "test_fn");
    }

    #[test]
    fn test_process_directory_with_nested_mod() {
        let test_dir = setup_test_directory();
        create_test_file(test_dir.path(), "lib.rs", "pub mod nested { pub fn nested_fn() {} }");

        let mut public_items = HashMap::new();
        process_directory(test_dir.path(), &mut public_items);

        assert_eq!(public_items.len(), 1);
        let file_items = public_items.get(&format!("{}/lib.rs", test_dir.path().display())).unwrap();
        assert_eq!(file_items.get("Mod").unwrap().len(), 1);
        assert_eq!(file_items.get("Mod").unwrap()[0].get("name").unwrap(), "nested");
        assert_eq!(file_items.get("Fn").unwrap().len(), 1);
        assert_eq!(file_items.get("Fn").unwrap()[0].get("name").unwrap(), "nested_fn");
    }

    #[test]
    fn test_process_directory_with_enum() {
        let test_dir = setup_test_directory();
        create_test_file(test_dir.path(), "lib.rs", "pub enum TestEnum { Variant1, Variant2 }");

        let mut public_items = HashMap::new();
        process_directory(test_dir.path(), &mut public_items);

        assert_eq!(public_items.len(), 1);
        let file_items = public_items.get(&format!("{}/lib.rs", test_dir.path().display())).unwrap();
        assert_eq!(file_items.get("Enum").unwrap().len(), 1);
        assert_eq!(file_items.get("Enum").unwrap()[0].get("name").unwrap(), "TestEnum");
    }

    #[test]
    fn test_process_directory_with_struct() {
        let test_dir = setup_test_directory();
        create_test_file(test_dir.path(), "lib.rs", "pub struct TestStruct { pub field: i32 }");

        let mut public_items = HashMap::new();
        process_directory(test_dir.path(), &mut public_items);

        assert_eq!(public_items.len(), 1);
        let file_items = public_items.get(&format!("{}/lib.rs", test_dir.path().display())).unwrap();
        assert_eq!(file_items.get("Struct").unwrap().len(), 1);
        assert_eq!(file_items.get("Struct").unwrap()[0].get("name").unwrap(), "TestStruct");
    }

    #[test]
    fn test_process_directory_with_trait() {
        let test_dir = setup_test_directory();
        create_test_file(test_dir.path(), "lib.rs", "pub trait TestTrait { fn trait_fn(&self); }");

        let mut public_items = HashMap::new();
        process_directory(test_dir.path(), &mut public_items);

        assert_eq!(public_items.len(), 1);
        let file_items = public_items.get(&format!("{}/lib.rs", test_dir.path().display())).unwrap();
        assert_eq!(file_items.get("Trait").unwrap().len(), 1);
        assert_eq!(file_items.get("Trait").unwrap()[0].get("name").unwrap(), "TestTrait");
        assert_eq!(file_items.get("TraitFn").unwrap().len(), 1);
        assert_eq!(file_items.get("TraitFn").unwrap()[0].get("name").unwrap(), "trait_fn");
    }

    #[test]
    fn test_process_directory_with_impl() {
        let test_dir = setup_test_directory();
        create_test_file(test_dir.path(), "lib.rs", "pub struct TestStruct; impl TestStruct { pub fn impl_fn() {} }");

        let mut public_items = HashMap::new();
        process_directory(test_dir.path(), &mut public_items);

        assert_eq!(public_items.len(), 1);
        let file_items = public_items.get(&format!("{}/lib.rs", test_dir.path().display())).unwrap();
        assert_eq!(file_items.get("Struct").unwrap().len(), 1);
        assert_eq!(file_items.get("Struct").unwrap()[0].get("name").unwrap(), "TestStruct");
        assert_eq!(file_items.get("ImplFn").unwrap().len(), 1);
        assert_eq!(file_items.get("ImplFn").unwrap()[0].get("name").unwrap(), "impl_fn");
    }

    #[test]
    fn test_process_directory_with_use() {
        let test_dir = setup_test_directory();
        create_test_file(test_dir.path(), "lib.rs", "pub use std::collections::HashMap;");

        let mut public_items = HashMap::new();
        process_directory(test_dir.path(), &mut public_items);

        assert_eq!(public_items.len(), 1);
        let file_items = public_items.get(&format!("{}/lib.rs", test_dir.path().display())).unwrap();
        assert_eq!(file_items.get("Use").unwrap().len(), 1);
        assert_eq!(file_items.get("Use").unwrap()[0].get("name").unwrap(), "std::collections::HashMap");
    }

    #[test]
    fn test_process_directory_with_macro() {
        let test_dir = setup_test_directory();
        create_test_file(test_dir.path(), "lib.rs", "macro_rules! test_macro { () => { println!(\"Hello, world!\"); }; }");

        let mut public_items = HashMap::new();
        process_directory(test_dir.path(), &mut public_items);

        assert_eq!(public_items.len(), 1);
        let file_items = public_items.get(&format!("{}/lib.rs", test_dir.path().display())).unwrap();
        assert_eq!(file_items.get("Macro").unwrap().len(), 1);
        assert_eq!(file_items.get("Macro").unwrap()[0].get("name").unwrap(), "macro_rules");
    }
}