use std::fs;
use std::path::Path;
use syn::{Item, Visibility, ImplItem};

fn main() {
    let path = "../lighthouse"; // Change this to your codebase path
    let mut public_types = Vec::new();
    let mut public_functions = Vec::new();

    process_directory(Path::new(path), &mut public_types, &mut public_functions);

    // Output the results
    println!("Public Types:");
    for ty in public_types {
        println!("- {}", ty);
    }

    println!("\nPublic Functions:");
    for func in public_functions {
        println!("- {}", func);
    }
}

fn process_directory(path: &Path, public_types: &mut Vec<String>, public_functions: &mut Vec<String>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();

            if path.is_dir() {
                process_directory(&path, public_types, public_functions);
            } else if path.extension().map(|s| s == "rs").unwrap_or(false) {
                let content = fs::read_to_string(&path).expect("Failed to read file");
                match syn::parse_file(&content) {
                    Ok(syntax_tree) => {
                        for item in syntax_tree.items {
                            match item {
                                Item::Struct(item_struct) => {
                                    if is_public(&item_struct.vis) {
                                        public_types.push(item_struct.ident.to_string());
                                    }
                                }
                                Item::Trait(item_trait) => {
                                    if is_public(&item_trait.vis) {
                                        public_types.push(item_trait.ident.to_string());
                                    }
                                }
                                Item::Impl(item_impl) => {
                                    // Check if the implementation is for a trait
                                    if let Some((_, trait_path, _)) = &item_impl.trait_ {
                                        for item in item_impl.items {
                                            if let ImplItem::Fn(method) = item {
                                                if is_public(&method.vis) {
                                                    public_functions.push(format!("{}::{}", trait_path.segments.last().unwrap().ident, method.sig.ident));
                                                }
                                            }
                                        }
                                    } else {
                                        // Handle inherent implementations
                                        for item in item_impl.items {
                                            if let ImplItem::Fn(method) = item {
                                                if is_public(&method.vis) {
                                                    public_functions.push(method.sig.ident.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                                Item::Type(item_type) => {
                                    if is_public(&item_type.vis) {
                                        public_types.push(item_type.ident.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
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

fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_valid_rust_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "pub struct TestStruct;").unwrap();

        let mut public_types = Vec::new();
        let mut public_functions = Vec::new();
        process_directory(dir.path(), &mut public_types, &mut public_functions);

        assert_eq!(public_types, vec!["TestStruct"]);
        assert!(public_functions.is_empty());
    }

    #[test]
    fn test_invalid_rust_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "invalid rust code").unwrap();

        let mut public_types = Vec::new();
        let mut public_functions = Vec::new();
        process_directory(dir.path(), &mut public_types, &mut public_functions);

        assert!(public_types.is_empty());
        assert!(public_functions.is_empty());
    }

    #[test]
    fn test_recursive_directory() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file_path = subdir.join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "pub struct TestStruct;").unwrap();

        let mut public_types = Vec::new();
        let mut public_functions = Vec::new();
        process_directory(dir.path(), &mut public_types, &mut public_functions);

        assert_eq!(public_types, vec!["TestStruct"]);
        assert!(public_functions.is_empty());
    }

    #[test]
    fn test_visibility_check() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "struct PrivateStruct;").unwrap();

        let mut public_types = Vec::new();
        let mut public_functions = Vec::new();
        process_directory(dir.path(), &mut public_types, &mut public_functions);

        assert_eq!(public_types, Vec::<String>::new(), "public_types is not empty: {:?}", public_types);
        assert!(public_functions.is_empty());
    }

    #[test]
    fn test_public_function() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "pub trait TestTrait {{ fn test_method(); }}").unwrap();
        writeln!(file, "impl TestTrait for TestStruct {{ pub fn test_method() {{}} }}").unwrap();

        let mut public_types = Vec::new();
        let mut public_functions = Vec::new();
        process_directory(dir.path(), &mut public_types, &mut public_functions);

        assert!(public_types.is_empty(), "public_types is not empty: {:?}", public_types);
        assert_eq!(public_functions, vec!["TestTrait::test_method"]);
    }
}