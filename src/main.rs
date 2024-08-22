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
        for (item_type, names) in items {
            println!("  {}:", item_type);
            for name in names {
                println!("    - {}", name);
            }
        }
    }
}

fn process_directory(path: &Path, public_items: &mut HashMap<String, HashMap<String, Vec<String>>>) {
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

                        for item in syntax_tree.items {
                            match item {
                                Item::Const(item_const) => {
                                    if is_public(&item_const.vis) {
                                        file_items.entry("Const".to_string()).or_insert_with(Vec::new).push(item_const.ident.to_string());
                                    }
                                }
                                Item::Enum(item_enum) => {
                                    if is_public(&item_enum.vis) {
                                        file_items.entry("Enum".to_string()).or_insert_with(Vec::new).push(item_enum.ident.to_string());
                                    }
                                }
                                Item::ExternCrate(item_extern_crate) => {
                                    if is_public(&item_extern_crate.vis) {
                                        file_items.entry("ExternCrate".to_string()).or_insert_with(Vec::new).push(item_extern_crate.ident.to_string());
                                    }
                                }
                                Item::Fn(item_fn) => {
                                    if is_public(&item_fn.vis) {
                                        file_items.entry("Fn".to_string()).or_insert_with(Vec::new).push(item_fn.sig.ident.to_string());
                                    }
                                }
                                Item::ForeignMod(item_foreign_mod) => {
                                    for item in item_foreign_mod.items {
                                        if let syn::ForeignItem::Fn(item_fn) = item {
                                            if is_public(&item_fn.vis) {
                                                file_items.entry("ForeignFn".to_string()).or_insert_with(Vec::new).push(item_fn.sig.ident.to_string());
                                            }
                                        }
                                    }
                                }
                                Item::Impl(item_impl) => {
                                    if let Some((_, trait_path, _)) = &item_impl.trait_ {
                                        for item in item_impl.items {
                                            if let ImplItem::Fn(method) = item {
                                                if is_public(&method.vis) {
                                                    file_items.entry("TraitFn".to_string()).or_insert_with(Vec::new).push(format!("{}::{}", trait_path.segments.last().unwrap().ident, method.sig.ident));
                                                }
                                            }
                                        }
                                    } else {
                                        for item in item_impl.items {
                                            if let ImplItem::Fn(method) = item {
                                                if is_public(&method.vis) {
                                                    file_items.entry("ImplFn".to_string()).or_insert_with(Vec::new).push(method.sig.ident.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                                Item::Macro(item_macro) => {
									file_items.entry("Macro".to_string()).or_insert_with(Vec::new).push(item_macro.mac.path.segments.last().unwrap().ident.to_string());
                                }
                                Item::Mod(item_mod) => {
                                    if is_public(&item_mod.vis) {
                                        file_items.entry("Mod".to_string()).or_insert_with(Vec::new).push(item_mod.ident.to_string());
                                    }
                                }
                                Item::Static(item_static) => {
                                    if is_public(&item_static.vis) {
                                        file_items.entry("Static".to_string()).or_insert_with(Vec::new).push(item_static.ident.to_string());
                                    }
                                }
                                Item::Struct(item_struct) => {
                                    if is_public(&item_struct.vis) {
                                        file_items.entry("Struct".to_string()).or_insert_with(Vec::new).push(item_struct.ident.to_string());
                                    }
                                }
                                Item::Trait(item_trait) => {
                                    if is_public(&item_trait.vis) {
                                        file_items.entry("Trait".to_string()).or_insert_with(Vec::new).push(item_trait.ident.to_string());
                                    }
                                }
                                Item::TraitAlias(item_trait_alias) => {
                                    if is_public(&item_trait_alias.vis) {
                                        file_items.entry("TraitAlias".to_string()).or_insert_with(Vec::new).push(item_trait_alias.ident.to_string());
                                    }
                                }
                                Item::Type(item_type) => {
                                    if is_public(&item_type.vis) {
                                        file_items.entry("Type".to_string()).or_insert_with(Vec::new).push(item_type.ident.to_string());
                                    }
                                }
                                Item::Union(item_union) => {
                                    if is_public(&item_union.vis) {
                                        file_items.entry("Union".to_string()).or_insert_with(Vec::new).push(item_union.ident.to_string());
                                    }
                                }
                                Item::Use(item_use) => {
									if is_public(&item_use.vis) {
	                                    file_items.entry("Use".to_string()).or_insert_with(Vec::new).push(item_use.tree.to_token_stream().to_string().replace(" ", ""));
									}
                                }
                                Item::Verbatim(_) => {
                                    file_items.entry("Verbatim".to_string()).or_insert_with(Vec::new).push("Verbatim".to_string());
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
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
	fn test_is_public() {
		assert!(is_public(&Visibility::Public(syn::parse_quote! {
			pub
		})));
		assert!(!is_public(&Visibility::Inherited));
	}

    #[test]
    fn test_process_directory_with_empty_dir() {
        let dir = tempdir().unwrap();
        let mut public_items = HashMap::new();
        process_directory(dir.path(), &mut public_items);
        assert!(public_items.is_empty());
    }

    #[test]
    fn test_process_directory_with_public_items() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            r#"
            pub const TEST_CONST: i32 = 42;
            pub enum TestEnum {{ A, B, C }}
            pub extern crate test_crate;
            pub fn test_fn() {{}}
            extern "C" {{ pub fn foreign_fn(); }}
            pub struct TestStruct;
            pub trait TestTrait {{ fn trait_fn(&self); }}
            pub type TestType = i32;
            pub union TestUnion {{ a: i32, b: f32 }}
            pub use std::collections::HashMap;
            "#
        )
        .unwrap();

        let mut public_items = HashMap::new();
        process_directory(dir.path(), &mut public_items);

        let file_items = public_items.get(file_path.to_str().unwrap()).unwrap();
        assert_eq!(file_items["Const"], vec!["TEST_CONST"]);
        assert_eq!(file_items["Enum"], vec!["TestEnum"]);
        assert_eq!(file_items["ExternCrate"], vec!["test_crate"]);
        assert_eq!(file_items["Fn"], vec!["test_fn"]);
        assert_eq!(file_items["ForeignFn"], vec!["foreign_fn"]);
        assert_eq!(file_items["Struct"], vec!["TestStruct"]);
        assert_eq!(file_items["Trait"], vec!["TestTrait"]);
        assert_eq!(file_items["Type"], vec!["TestType"]);
        assert_eq!(file_items["Union"], vec!["TestUnion"]);
        assert_eq!(file_items["Use"], vec!["std::collections::HashMap"]);
    }

	#[test]
	fn test_process_directory_with_private_items() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test.rs");
		let mut file = File::create(&file_path).unwrap();
		writeln!(
			file,
			r#"
			const TEST_CONST: i32 = 42;
			enum TestEnum {{ A, B, C }}
			extern crate test_crate;
			fn test_fn() {{}}
			extern "C" {{ fn foreign_fn(); }}
			struct TestStruct;
			trait TestTrait {{ fn trait_fn(&self); }}
			type TestType = i32;
			union TestUnion {{ a: i32, b: f32 }}
			use std::collections::HashMap;
			"#
		)
		.unwrap();

		let mut public_items = HashMap::new();
		process_directory(dir.path(), &mut public_items);

		let file_items = public_items.get(file_path.to_str().unwrap());
		println!("{:?}", file_items);
		assert!(file_items.unwrap().is_empty());
	}

	#[test]
	fn test_process_directory_with_mixed_visibility_items() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test.rs");
		let mut file = File::create(&file_path).unwrap();
		writeln!(
			file,
			r#"
			pub const TEST_CONST: i32 = 42;
			enum TestEnum {{ A, B, C }}
			pub extern crate test_crate;
			fn test_fn() {{}}
			extern "C" {{ pub fn foreign_fn(); }}
			pub struct TestStruct;
			trait TestTrait {{ fn trait_fn(&self); }}
			pub type TestType = i32;
			union TestUnion {{ a: i32, b: f32 }}
			pub use std::collections::HashMap;
			"#
		)
		.unwrap();

		let mut public_items = HashMap::new();
		process_directory(dir.path(), &mut public_items);

		let file_items = public_items.get(file_path.to_str().unwrap()).unwrap();
		assert_eq!(file_items["Const"], vec!["TEST_CONST"]);
		assert_eq!(file_items["ExternCrate"], vec!["test_crate"]);
		assert_eq!(file_items["ForeignFn"], vec!["foreign_fn"]);
		assert_eq!(file_items["Struct"], vec!["TestStruct"]);
		assert_eq!(file_items["Type"], vec!["TestType"]);
		assert_eq!(file_items["Use"], vec!["std::collections::HashMap"]);
		assert!(file_items.get("Enum").is_none());
		assert!(file_items.get("Fn").is_none());
		assert!(file_items.get("Trait").is_none());
		assert!(file_items.get("Union").is_none());
	}

	#[test]
	fn test_process_directory_with_nested_modules() {
		let dir = tempdir().unwrap();
		let mod_dir = dir.path().join("nested");
		fs::create_dir(&mod_dir).unwrap();
		let file_path = mod_dir.join("mod.rs");
		let mut file = File::create(&file_path).unwrap();
		writeln!(
			file,
			r#"
			pub mod nested_mod {{
				pub const TEST_CONST: i32 = 42;
				pub fn test_fn() {{}}
			}}
			"#
		)
		.unwrap();

		let mut public_items = HashMap::new();
		process_directory(dir.path(), &mut public_items);

		let file_items = public_items.get(file_path.to_str().unwrap()).unwrap();
		assert_eq!(file_items["Mod"], vec!["nested_mod"]);
	}
}