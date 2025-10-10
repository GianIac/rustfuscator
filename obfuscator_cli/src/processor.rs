use crate::config::ObfuscateConfig;
use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use prettyplease;
use quote::{quote, quote_spanned};
use rust_code_obfuscator_core::utils::generate_obf_suffix;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    parse_file, visit_mut::VisitMut, Expr, ExprForLoop, ExprIf, ExprLit, ExprLoop, ExprMatch,
    ExprWhile, Ident, ItemFn, Lit, PatIdent, Stmt, Pat
};

pub fn process_file(
    path: &Path,
    relative_path: &Path,
    config: &ObfuscateConfig,
    json_output: bool,
) -> Result<(String, bool, Option<String>)> {
    // (transformed, changed, before_for_diff)
    let source = fs::read_to_string(path)?;

    let file_name = relative_path.to_string_lossy();
    if let Some(skip_list) = &config.obfuscation.skip_files {
        if skip_list.iter().any(|entry| file_name.ends_with(entry)) {
            println!("Skipping file (matched skip_files): {}", file_name);
            return Ok((source, false, None));
        }
    }

    if let Some(include) = &config.include {
        let mut builder = GlobSetBuilder::new();
        if let Some(files) = &include.files {
            for pattern in files {
                builder.add(Glob::new(pattern)?);
            }
        }
        let set = builder.build()?;

        if !set.is_match(path) {
            println!("Skipping file (not in include patterns): {}", file_name);
            return Ok((source, false, None));
        }

        if let Some(exclude_patterns) = &include.exclude {
            for pattern in exclude_patterns {
                let exclude_glob = Glob::new(pattern)?;
                if exclude_glob.compile_matcher().is_match(path) {
                    println!("Skipping file (excluded by pattern): {}", file_name);
                    return Ok((source, false, None));
                }
            }
        }
    }

    let mut syntax_tree = parse_file(&source)?;

    let mut transformer = ObfuscationTransformer {
        min_string_length: config.obfuscation.min_string_length,
        ignore_strings: config.obfuscation.ignore_strings.clone(),
        rename_identifiers: config
            .identifiers
            .as_ref()
            .map(|id| id.rename)
            .unwrap_or(false),
        preserve_idents: config
            .identifiers
            .as_ref()
            .and_then(|id| id.preserve.clone())
            .unwrap_or_default(),
        obfuscate_strings: config.obfuscation.strings,
        obfuscate_flow: config.obfuscation.control_flow,
        skip_attributes: config.obfuscation.skip_attributes.unwrap_or(false),
        obfuscated_vars: HashSet::new(),
    };
    transformer.visit_file_mut(&mut syntax_tree);

    let mut has_use = false;
    for item in &syntax_tree.items {
        if let syn::Item::Use(u) = item {
            let use_str = quote!(#u).to_string();
            if use_str.contains("obfuscate_string") || use_str.contains("obfuscate_flow") {
                has_use = true;
                break;
            }
        }
    }

    let mut new_use_items: Vec<syn::Item> = Vec::new();

    if transformer.obfuscate_strings {
        new_use_items.push(syn::parse_quote! { use rust_code_obfuscator::obfuscate_string; });
    }
    if transformer.obfuscate_flow {
        new_use_items.push(syn::parse_quote! { use rust_code_obfuscator::obfuscate_flow; });
    }

    if !has_use && !new_use_items.is_empty() {
        for it in new_use_items.into_iter().rev() {
            syntax_tree.items.insert(0, it);
        }
    }

    let transformed = prettyplease::unparse(&syntax_tree);
    let changed = transformed != source;

    if json_output {
        let json_payload = serde_json::json!({
            "file": relative_path.to_string_lossy(),
            "transformed": transformed,
            "changed": changed
        });

        let json_path = PathBuf::from("obf_json")
            .join(relative_path)
            .with_extension("json");
        fs::create_dir_all(json_path.parent().unwrap())?;
        fs::write(&json_path, serde_json::to_string_pretty(&json_payload)?)?;
        println!("✓ Saved transformed JSON to {}", json_path.display());

        return Ok((String::new(), changed, None));
    }

    Ok((transformed, changed, Some(source)))
}

struct ObfuscationTransformer {
    min_string_length: Option<usize>,
    ignore_strings: Option<Vec<String>>,
    rename_identifiers: bool,
    preserve_idents: Vec<String>,
    obfuscate_strings: bool,
    obfuscate_flow: bool,
    skip_attributes: bool,
    obfuscated_vars: HashSet<String>,
}

impl VisitMut for ObfuscationTransformer {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if !self.obfuscate_strings {
            return syn::visit_mut::visit_expr_mut(self, expr);
        }

        if let Expr::Lit(ExprLit {
            lit: Lit::Str(ref lit_str),
            ..
        }) = expr
        {
            let value = lit_str.value();

            if self
                .min_string_length
                .map_or(false, |min| value.len() < min)
            {
                return;
            }

            if self
                .ignore_strings
                .as_ref()
                .map_or(false, |list| list.contains(&value))
            {
                return;
            }

            let span = lit_str.span();
            let wrapped: Expr =
                syn::parse2(quote_spanned! {span=> obfuscate_string!(#value) }).unwrap();
            *expr = wrapped;
        }

        syn::visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        if let Stmt::Local(local) = stmt {
            if let Some(init) = &local.init {
                if let Expr::Macro(mac) = &*init.expr {
                    if mac.mac.path.is_ident("obfuscate_string") {
                        if let Some(pat_ident) = collect_idents_from_pat(&local.pat){
                            self.obfuscated_vars.insert(pat_ident.to_string());
                        }
                    }
                }
            }
        }

        syn::visit_mut::visit_stmt_mut(self, stmt);
    }

    fn visit_attribute_mut(&mut self, attr: &mut syn::Attribute) {
        if self.skip_attributes {
            return;
        }
        syn::visit_mut::visit_attribute_mut(self, attr);
    }

    fn visit_expr_if_mut(&mut self, node: &mut ExprIf) {
        if self.obfuscate_flow {
            let inject: Stmt = syn::parse_quote! { obfuscate_flow!(); };
            node.then_branch.stmts.insert(0, inject.clone());
            if let Some((_, else_branch)) = &mut node.else_branch {
                if let Expr::Block(block) = else_branch.as_mut() {
                    block.block.stmts.insert(0, inject.clone());
                }
            }
        }
        syn::visit_mut::visit_expr_if_mut(self, node);
    }

    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        if self.obfuscate_flow {
            for arm in &mut node.arms {
                let original = &arm.body;
                arm.body = Box::new(syn::parse_quote!({ obfuscate_flow!(); #original }));
            }
        }

        if let Expr::Path(ref expr_path) = *node.expr {
            if let Some(ident) = expr_path.path.get_ident() {
                if self.obfuscated_vars.contains(&ident.to_string()) {
                    node.expr = Box::new(syn::parse_quote!(&*#ident));
                }
            }
        }

        syn::visit_mut::visit_expr_match_mut(self, node);
    }

    fn visit_expr_loop_mut(&mut self, node: &mut ExprLoop) {
        if self.obfuscate_flow {
            let inject: Stmt = syn::parse_quote! { obfuscate_flow!(); };
            node.body.stmts.insert(0, inject);
        }
        syn::visit_mut::visit_expr_loop_mut(self, node);
    }

    fn visit_expr_while_mut(&mut self, node: &mut ExprWhile) {
        if self.obfuscate_flow {
            let inject: Stmt = syn::parse_quote! { obfuscate_flow!(); };
            node.body.stmts.insert(0, inject);
        }
        syn::visit_mut::visit_expr_while_mut(self, node);
    }

    fn visit_expr_for_loop_mut(&mut self, node: &mut ExprForLoop) {
        if self.obfuscate_flow {
            let inject: Stmt = syn::parse_quote! { obfuscate_flow!(); };
            node.body.stmts.insert(0, inject);
        }
        syn::visit_mut::visit_expr_for_loop_mut(self, node);
    }

    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        if self.rename_identifiers {
            let original = func.sig.ident.to_string();
            if !self.preserve_idents.contains(&original) {
                let suffix = generate_obf_suffix();
                let new_name = format!("{}_obf_{}", original, suffix);
                func.sig.ident = Ident::new(&new_name, func.sig.ident.span());
            }
        }

        self.visit_block_mut(&mut func.block);
    }

    fn visit_pat_ident_mut(&mut self, pat: &mut PatIdent) {
        if !self.rename_identifiers {
            return;
        }

        let name = pat.ident.to_string();
        if self.preserve_idents.contains(&name) {
            return;
        }

        let suffix = generate_obf_suffix();
        let new_name = format!("{}_x{}", name, suffix);
        pat.ident = Ident::new(&new_name, pat.ident.span());

        syn::visit_mut::visit_pat_ident_mut(self, pat);
    }
}

fn collect_idents_from_pat(pat: &Pat) -> Option<Ident> {
    match pat {
        Pat::Ident(p) => Some(p.ident.clone()),
        Pat::Type(t) => collect_idents_from_pat(&t.pat),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ObfuscationSection;
    use tempfile::TempDir;
    use syn::{AttrStyle, Attribute, Meta, Token};
    use syn::token;
    use proc_macro2::Span;

    /// Input type for the `create_attribute` function, specifying the kind of `syn::Meta` to create.
    enum AttrInput {
        PathDsc(&'static  str),
        ListDsc(ListDscInput),
        NameValueDsc(NameValueDscInput)
    }

    struct ListDscInput {
        path_dsc:&'static  str,
        tokens:Vec<&'static  str>
    }

    struct NameValueDscInput {
        path_dsc:&'static  str,
        value_dsc:&'static  str
    }

    fn cfg(
        strings: bool,
        min_str_len: Option<usize>,
        ignore_strings: Option<Vec<String>>,
        flow: bool,
        skip_files: Option<Vec<String>>,
        skip_attributes: Option<bool>,
    ) -> ObfuscateConfig {
        ObfuscateConfig {
            obfuscation: ObfuscationSection {
                strings,
                min_string_length: min_str_len,
                ignore_strings: ignore_strings,
                control_flow: flow,
                skip_files: skip_files,
                skip_attributes: skip_attributes,
            },
            identifiers: None,
            include: None,
        }
    }

    fn obf_transformer(
        min_str_len: Option<usize>,
        ignore_strings: Option<Vec<String>>,
        rename_identifiers: bool,
        obfuscate_strings: bool,
        obfuscate_flow: bool,
        skip_attributes: bool,
    ) -> ObfuscationTransformer {
        ObfuscationTransformer {
            min_string_length: min_str_len,
            ignore_strings: ignore_strings,
            rename_identifiers: rename_identifiers,
            preserve_idents: vec![],
            obfuscate_strings: obfuscate_strings,
            obfuscate_flow: obfuscate_flow,
            skip_attributes: skip_attributes,
            obfuscated_vars: HashSet::new(),
        }
    }

    /// Returns `path` and `relative_path` to a file created in a temporary directory.
    fn create_rs_file(src: &'static str) -> (TempDir, PathBuf, PathBuf) {
        let file_name = "lib.rs";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(file_name);
        let relative_path = std::path::Path::new(file_name).to_path_buf();
        std::fs::write(&path, src).unwrap();
        return (dir, path, relative_path);
    }

    fn get_str_lit_expression(str: &'static str) -> Expr {
        let literal = ExprLit {
            attrs: vec![],
            lit: Lit::Str(syn::LitStr::new(str, proc_macro2::Span::call_site())),
        };
        Expr::Lit(literal)
    }

    /// Creates `syn::Attribute`.
    fn create_attribute(is_inner:bool,input: AttrInput) -> Attribute {
        let style = match is_inner {
            true => AttrStyle::Inner(Token![!](Span::call_site())),
            false => AttrStyle::Outer,
        };

        let meta: Meta = match input {
            AttrInput::PathDsc(p_name) => 
                Meta::Path(syn::Path::from(Ident::new(p_name, Span::call_site()))),
            AttrInput::ListDsc(input) => {
                let literals = input.tokens.iter().map(|s| quote! { #s });
                let tokens = quote! { #(#literals),* };
                Meta::List(syn::MetaList {
                    path: syn::Path::from(Ident::new(input.path_dsc, Span::call_site())),
                    delimiter: syn::MacroDelimiter::Paren(token::Paren(Span::call_site())),
                    tokens: tokens,
                })
            },
            AttrInput::NameValueDsc(input) =>{
                Meta::NameValue(syn::MetaNameValue {
                    path: syn::Path::from(Ident::new(input.path_dsc, Span::call_site())),
                    eq_token: Token![=](Span::call_site()),
                    value: get_str_lit_expression(input.value_dsc),
                })
            },
        };
        let attr = Attribute {
            pound_token: Token![#](Span::call_site()),
            style: style,
            bracket_token: token::Bracket(Span::call_site()),
            meta: meta,
        };
        return attr;
    }

    fn _print_atribut_tokens(attr: syn::Attribute ) {
        let tokens = quote!(#attr);
        println!("Tokens: {}", tokens);
    }

    #[test]
    fn cfg_strings_on() {
        let src = r#"pub const TEST: &str = "test";"#;

        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(true, None, None, false, None, None),
            false,
        )
        .unwrap();

        let mut lines = out.lines();
        let line_1 = lines.next().unwrap();
        let line_2 = lines.next().unwrap();
        assert!(line_1 == r#"use rust_code_obfuscator::obfuscate_string;"#);
        assert!(line_2 == r#"pub const TEST: &str = obfuscate_string!("test");"#);
    }

    #[test]
    fn cfg_strings_off() {
        let src = r#"pub const TEST: &str = "test";"#;

        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(false, None, None, false, None, None),
            false,
        )
        .unwrap();

        let mut lines = out.lines();
        let line_1 = lines.next().unwrap();
        let line_2 = lines.next();
        assert!(line_1 == src);
        assert!(line_2 == None);
    }

    #[test]
    fn cfg_set_min_string_length() {
        let str_len_limit: Option<usize> = Some(5);
        let src = r#"pub const TEST_1: &str = "long enough test";
pub const TEST_2: &str = "test";
"#;

        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(true, str_len_limit, None, false, None, None),
            false,
        )
        .unwrap();

        let mut lines = out.lines();
        let _ = lines.next();
        let line_2 = lines.next().unwrap();
        let line_3 = lines.next().unwrap();
        assert!(line_2 == r#"pub const TEST_1: &str = obfuscate_string!("long enough test");"#);
        assert!(line_3 == r#"pub const TEST_2: &str = "test";"#);
    }

    #[test]
    fn cfg_skip_files() {
        let src = r#"pub const TEST: &str = "test";"#;

        let (_dir, path, relative_path) = create_rs_file(src);
        let skipped_files = Some(vec![String::from(relative_path.to_str().unwrap())]);

        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(true, None, None, false, skipped_files, None),
            false,
        )
        .unwrap();
        let mut lines = out.lines();
        let line_1 = lines.next().unwrap();
        assert!(line_1 == src);
    }

    #[test]
    fn cfg_skip_atributes() {
        let src = r#"//! Crate docs
pub const TEST: &str = "test";"#;

        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(true, None, None, false, None, Some(true)),
            false,
        )
        .unwrap();
        let mut lines = out.lines();
        let line_1 = lines.next().unwrap();
        assert!(line_1 == r#"//! Crate docs"#);
    }

    #[test]
    fn cfg_flow_of() {
        let src = r#"pub fn if_me() {
    if true {}
}"#;

        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(false, None, None, false, None, None),
            false,
        )
        .unwrap();
        assert_eq!(out.trim(), src.trim());
    }

    #[test]
    fn cfg_flow_on() {
        let src = r#"pub fn if_me() {
    if true {}
}"#;

        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(false, None, None, true, None, None),
            false,
        )
        .unwrap();
        let mut lines = out.lines();
        let line_1: &str = lines.next().unwrap();
        let _line_2: &str = lines.next().unwrap();
        let _line_3: &str = lines.next().unwrap();
        let line_4: &str = lines.next().unwrap();
        assert_eq!(line_1, r#"use rust_code_obfuscator::obfuscate_flow;"#);
        assert_eq!(line_4.trim(), r#"obfuscate_flow!();"#);
    }

    #[test]
    fn inner_docs_stay_first_and_compileable_transform() {
        let src = r#"//! Crate docs
//! More docs

pub fn hello() -> &'static str { "hi" }
"#;
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("lib.rs");
        std::fs::write(&p, src).unwrap();

        let (out, _changed, _before) = super::process_file(
            &p,
            std::path::Path::new("lib.rs"),
            &cfg(true, None, None, false, None, Some(false)),
            false,
        )
        .unwrap();

        let trimmed = out.trim_start();
        let first = trimmed.chars().next().unwrap();
        println!("out: {}", out);
        println!("trimmed: {}", out);
        assert!(
            first == '#' || first == '/',
            "inner attrs must be first:\n{out}"
        );
        assert!(
            !trimmed.starts_with("use "),
            "never `use` before inner attrs:\n{out}"
        );
    }

    #[test]
    fn obf_transformer_test_visit_expr() {
        let lit_too_short = "foo";
        let lit_long_enough = "foo test";
        let lit_to_ignore = "ignore me";

        let cfg_min_len = lit_too_short.len() + 1;
        let cfg_ignore = vec![String::from(lit_to_ignore)];

        let mut expr_1 = get_str_lit_expression(lit_too_short);
        let mut expr_2 = get_str_lit_expression(lit_long_enough);
        let mut expr_3 = get_str_lit_expression(lit_to_ignore);

        let mut transformer = obf_transformer(
            Some(cfg_min_len),
            Some(cfg_ignore),
            false,
            true,
            false,
            false,
        );

        transformer.visit_expr_mut(&mut expr_1);
        transformer.visit_expr_mut(&mut expr_2);
        transformer.visit_expr_mut(&mut expr_3);

        match &expr_1 {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) => {
                assert_eq!(lit_str.value(), lit_too_short);
            }
            _ => panic!("expr_1 must be a string literal"),
        }

        match &expr_2 {
            Expr::Macro(expr_macro) => {
                let mac = &expr_macro.mac;
                assert_eq!(mac.tokens.to_string().trim_matches('"'), lit_long_enough);
                assert_eq!(mac.path.segments.last().unwrap().ident, "obfuscate_string");
            }
            _ => panic!("expr_2 must be a macro"),
        }

        match &expr_3 {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) => {
                assert_eq!(lit_str.value(), lit_to_ignore);
            }
            _ => panic!("expr_3 must be a string literal"),
        }
    }

    #[test]
    fn obf_transformer_test_visit_stmt() {
        let ident_name = "foo";
        let number_of_idents = 1;
        let mut stmt: Stmt = syn::parse_quote! {
            let foo: &str = obfuscate_string!("test");
        };

        let mut transformer = obf_transformer(
            None,
            None,
            false,
            true,
            false,
            false,
        );

        transformer.visit_stmt_mut(&mut stmt);
        let mut idents = transformer.obfuscated_vars.iter();

        assert_eq!(transformer.obfuscated_vars.len(), number_of_idents);
        assert_eq!(idents.next().unwrap(), ident_name);
    }

    #[test]
    fn obf_transformer_test_skip_attribute_mut() {
        let input  = AttrInput::PathDsc("my_path");
        let mut attr_1: Attribute = create_attribute(true, input);

        let input  = AttrInput::ListDsc(ListDscInput { path_dsc: "my_list_path", tokens: vec!["foo_1","foo_2","foo_3"] });
        let mut attr_2: Attribute = create_attribute(true, input);

        let name_value_3 = "my secret doc";
        let input  = AttrInput::NameValueDsc(NameValueDscInput { path_dsc: "doc", value_dsc: name_value_3 });
        let mut attr_3: Attribute = create_attribute(true, input);

        let mut transformer = obf_transformer(
            None,
            None,
            false,
            true,
            false,
            true,
        );

        transformer.visit_attribute_mut(&mut attr_1);
        transformer.visit_attribute_mut(&mut attr_2);
        transformer.visit_attribute_mut(&mut attr_3);

        match attr_3.meta {
            Meta::NameValue(meta_name_value) => {
                match meta_name_value.value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) => {
                        assert_eq!(lit_str.value(), name_value_3);
                    }
                    _ => panic!("attr_3.meta value must be a string literal"),
                }
            },
            _ => panic!("attr_3.meta must remain of type Meta::NameValue"),
        }
    }



    #[test]
    fn obf_my_test_test(){
        use syn::{parse_quote, Stmt, Pat};
        let stmt: Stmt = parse_quote! {
            let foo: &str = "test";
        };
    
        if let Stmt::Local(local) = &stmt {
            match &local.pat {
                Pat::Ident(pat_ident) => {
                    println!("Variable name 1: {}", pat_ident.ident);
                }
                Pat::Type(pat_type) => {
                    // inside Pat::Type, the inner pattern may still be Pat::Ident
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        println!("Variable name 2: {}", pat_ident.ident);
                    } else {
                        println!("Type pattern, but not an identifier");
                    }
                }
                _ => {
                    println!("Other pattern:");
                }
            }
        } else {
            println!("Not a local statement");
        }
    }
}
