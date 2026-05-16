use crate::config::ObfuscateConfig;
use anyhow::Result;
use quote::quote_spanned;
use rust_code_obfuscator_core::utils::generate_obf_suffix;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    parse_file, visit_mut::VisitMut, Expr, ExprForLoop, ExprIf, ExprLit, ExprLoop, ExprMatch,
    ExprWhile, Ident, ItemFn, Lit, LocalInit, Pat, PatIdent, Stmt, Type,
};

pub fn process_file(
    path: &Path,
    relative_path: &Path,
    config: &ObfuscateConfig,
    json_output: bool,
) -> Result<(String, bool, Option<String>)> {
    // (transformed, changed, before_for_diff)
    let source = fs::read_to_string(path)?;

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
        renamed_idents: HashMap::new(),
        obfuscated_vars: HashSet::new(),
        used_obfuscate_str: false,
        used_obfuscate_string: false,
        used_obfuscate_flow: false,
    };
    transformer.visit_file_mut(&mut syntax_tree);

    let mut has_use_str = false;
    let mut has_use_string = false;
    let mut has_use_flow = false;

    for item in &syntax_tree.items {
        if let syn::Item::Use(u) = item {
            if use_tree_contains_ident(&u.tree, "obfuscate_str") {
                has_use_str = true;
            }
            if use_tree_contains_ident(&u.tree, "obfuscate_string") {
                has_use_string = true;
            }
            if use_tree_contains_ident(&u.tree, "obfuscate_flow") {
                has_use_flow = true;
            }
        }
    }

    let mut new_use_items: Vec<syn::Item> = Vec::new();

    if transformer.used_obfuscate_string && !has_use_string {
        new_use_items.push(syn::parse_quote! {
            use rust_code_obfuscator::obfuscate_string;
        });
    }

    if transformer.used_obfuscate_str && !has_use_str {
        new_use_items.push(syn::parse_quote! {
            use rust_code_obfuscator::obfuscate_str;
        });
    }

    if transformer.used_obfuscate_flow && !has_use_flow {
        new_use_items.push(syn::parse_quote! {
            use rust_code_obfuscator::obfuscate_flow;
        });
    }

    if !new_use_items.is_empty() {
        for import in new_use_items.into_iter().rev() {
            syntax_tree.items.insert(0, import);
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
    renamed_idents: HashMap<String, Ident>,
    obfuscated_vars: HashSet<String>,
    used_obfuscate_str: bool,
    used_obfuscate_string: bool,
    used_obfuscate_flow: bool,
}

impl VisitMut for ObfuscationTransformer {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        // String obfuscation in let binding con type annotation ---
        if self.obfuscate_strings {
            if let Stmt::Local(local) = stmt {
                if let Some(LocalInit { expr, .. }) = &mut local.init {
                    // LocalInit { eq_token, expr, diverge }
                    let expr: &mut Expr = expr;

                    // only patterns with type annotation: let <pat>: <ty> = <expr>;
                    if let Pat::Type(pat_type) = &local.pat {
                        // Explicit type to the right
                        let ty: &Type = &pat_type.ty;

                        // A: let foo: &str = "literal";
                        if let Type::Reference(type_ref) = ty {
                            if let Type::Path(type_path) = &*type_ref.elem {
                                if type_path.path.is_ident("str") {
                                    if let Expr::Lit(ExprLit {
                                        lit: Lit::Str(ref lit_str),
                                        ..
                                    }) = expr
                                    {
                                        let value = lit_str.value();

                                        // min_string_length / ignore_strings
                                        if self
                                            .min_string_length
                                            .is_some_and(|min| value.len() < min)
                                        {
                                            // too short
                                        } else if self
                                            .ignore_strings
                                            .as_ref()
                                            .is_some_and(|list| list.contains(&value))
                                        {
                                            // in ignore list
                                        } else {
                                            let span = lit_str.span();
                                            let wrapped: Expr = syn::parse2(
                                                quote_spanned! {span=> obfuscate_str!(#value) },
                                            )
                                            .expect("failed to parse obfuscate_str! expression");
                                            *expr = wrapped;
                                            self.used_obfuscate_str = true;
                                        }
                                    }
                                }
                            }
                        }

                        // B: let foo: String = "literal";
                        if let Type::Path(type_path) = ty {
                            if type_path.path.is_ident("String") {
                                if let Expr::Lit(ExprLit {
                                    lit: Lit::Str(ref lit_str),
                                    ..
                                }) = expr
                                {
                                    let value = lit_str.value();

                                    if self.min_string_length.is_some_and(|min| value.len() < min) {
                                        // too short
                                    } else if self
                                        .ignore_strings
                                        .as_ref()
                                        .is_some_and(|list| list.contains(&value))
                                    {
                                        // ignore
                                    } else {
                                        let span = lit_str.span();
                                        let wrapped: Expr = syn::parse2(
                                            quote_spanned! {span=> obfuscate_string!(#value) },
                                        )
                                        .expect("failed to parse obfuscate_string! expression");
                                        *expr = wrapped;
                                        self.used_obfuscate_string = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Tracking of variables initialized via string macros
        if let Stmt::Local(local) = stmt {
            if let Some(LocalInit { expr, .. }) = &local.init {
                if let Expr::Macro(mac) = &**expr {
                    if mac.mac.path.is_ident("obfuscate_string")
                        || mac.mac.path.is_ident("obfuscate_str")
                    {
                        if let Some(pat_ident) = collect_idents_from_pat(&local.pat) {
                            self.obfuscated_vars.insert(pat_ident.to_string());
                        }
                    }
                }
            }
        }

        // Continue the standard visit
        syn::visit_mut::visit_stmt_mut(self, stmt);
    }

    fn visit_expr_method_call_mut(&mut self, node: &mut syn::ExprMethodCall) {
        if self.obfuscate_strings {
            let method_name = node.method.to_string();

            // For now we only handle push_str("literal") --> NEXT FUTURE
            if method_name == "push_str" {
                if let Some(first_arg) = node.args.first_mut() {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(ref lit_str),
                        ..
                    }) = first_arg
                    {
                        let value = lit_str.value();

                        if self.min_string_length.is_some_and(|min| value.len() < min) {
                            // too short
                        } else if self
                            .ignore_strings
                            .as_ref()
                            .is_some_and(|list| list.contains(&value))
                        {
                            // ignore
                        } else {
                            let span = lit_str.span();
                            let wrapped: Expr =
                                syn::parse2(quote_spanned! {span=> obfuscate_str!(#value) })
                                    .expect("failed to parse obfuscate_str! expression");

                            *first_arg = wrapped;
                            self.used_obfuscate_str = true;
                        }
                    }
                }
            }
        }

        syn::visit_mut::visit_expr_method_call_mut(self, node);
    }

    fn visit_attribute_mut(&mut self, attr: &mut syn::Attribute) {
        if self.skip_attributes {
            return;
        }
        syn::visit_mut::visit_attribute_mut(self, attr);
    }

    fn visit_expr_if_mut(&mut self, node: &mut ExprIf) {
        if self.obfuscate_flow {
            self.used_obfuscate_flow = true;
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
            self.used_obfuscate_flow = true;
            for arm in &mut node.arms {
                let original = &arm.body;
                *arm.body = syn::parse_quote!({ obfuscate_flow!(); #original });
            }
        }

        if let Expr::Path(ref expr_path) = *node.expr {
            if let Some(ident) = expr_path.path.get_ident() {
                if self.obfuscated_vars.contains(&ident.to_string()) {
                    *node.expr = syn::parse_quote!(&*#ident);
                }
            }
        }

        syn::visit_mut::visit_expr_match_mut(self, node);
    }

    fn visit_expr_loop_mut(&mut self, node: &mut ExprLoop) {
        if self.obfuscate_flow {
            self.used_obfuscate_flow = true;
            let inject: Stmt = syn::parse_quote! { obfuscate_flow!(); };
            node.body.stmts.insert(0, inject);
        }
        syn::visit_mut::visit_expr_loop_mut(self, node);
    }

    fn visit_expr_while_mut(&mut self, node: &mut ExprWhile) {
        if self.obfuscate_flow {
            self.used_obfuscate_flow = true;
            let inject: Stmt = syn::parse_quote! { obfuscate_flow!(); };
            node.body.stmts.insert(0, inject);
        }
        syn::visit_mut::visit_expr_while_mut(self, node);
    }

    fn visit_expr_for_loop_mut(&mut self, node: &mut ExprForLoop) {
        if self.obfuscate_flow {
            self.used_obfuscate_flow = true;
            let inject: Stmt = syn::parse_quote! { obfuscate_flow!(); };
            node.body.stmts.insert(0, inject);
        }
        syn::visit_mut::visit_expr_for_loop_mut(self, node);
    }

    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        if self.rename_identifiers {
            let original = func.sig.ident.clone();
            if let Some(new_ident) = self.rename_ident(&original, "_obf_") {
                func.sig.ident = new_ident;
            }
        }

        syn::visit_mut::visit_signature_mut(self, &mut func.sig);
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

        if let Some(new_ident) = self.rename_ident(&pat.ident, "_x") {
            pat.ident = new_ident;
        }

        syn::visit_mut::visit_pat_ident_mut(self, pat);
    }

    fn visit_expr_path_mut(&mut self, path: &mut syn::ExprPath) {
        if self.rename_identifiers && path.qself.is_none() && path.path.segments.len() == 1 {
            let ident = &path.path.segments[0].ident;
            if let Some(new_ident) = self.renamed_idents.get(&ident.to_string()) {
                path.path.segments[0].ident = new_ident.clone();
            }
        }

        syn::visit_mut::visit_expr_path_mut(self, path);
    }

    fn visit_item_const_mut(&mut self, item: &mut syn::ItemConst) {
        // Default: do NOT obfuscate const &str = "..."
        if self.obfuscate_strings {
            if let Type::Reference(type_ref) = &*item.ty {
                if let Type::Path(type_path) = &*type_ref.elem {
                    if type_path.path.is_ident("str")
                        && matches!(
                            &*item.expr,
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(_),
                                ..
                            })
                        )
                    {
                        // skip
                        return;
                    }
                }
            }
        }
        syn::visit_mut::visit_item_const_mut(self, item);
    }
}

impl ObfuscationTransformer {
    fn rename_ident(&mut self, ident: &Ident, separator: &str) -> Option<Ident> {
        let original = ident.to_string();
        if self.preserve_idents.contains(&original) {
            return None;
        }

        if let Some(existing) = self.renamed_idents.get(&original) {
            return Some(existing.clone());
        }

        let suffix = generate_obf_suffix();
        let new_ident = Ident::new(&format!("{original}{separator}{suffix}"), ident.span());
        self.renamed_idents.insert(original, new_ident.clone());
        Some(new_ident)
    }
}

fn collect_idents_from_pat(pat: &Pat) -> Option<Ident> {
    match pat {
        Pat::Ident(p) => Some(p.ident.clone()),
        Pat::Type(t) => collect_idents_from_pat(&t.pat),
        _ => None,
    }
}

fn use_tree_contains_ident(tree: &syn::UseTree, target: &str) -> bool {
    match tree {
        syn::UseTree::Path(path) => use_tree_contains_ident(&path.tree, target),
        syn::UseTree::Name(name) => name.ident == target,
        syn::UseTree::Rename(rename) => rename.rename == target,
        syn::UseTree::Group(group) => group
            .items
            .iter()
            .any(|item| use_tree_contains_ident(item, target)),
        syn::UseTree::Glob(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ObfuscationSection;
    use proc_macro2::Span;
    use quote::quote;
    use syn::token;
    use syn::{AttrStyle, Attribute, Block, ExprBlock, ExprIf, Meta, Stmt, StmtMacro, Token};
    use tempfile::TempDir;

    /// Input type for the `create_attribute` function, specifying the kind of `syn::Meta` to create.
    enum AttrInput {
        PathDsc(&'static str),
        ListDsc(ListDscInput),
        NameValueDsc(NameValueDscInput),
    }

    struct ListDscInput {
        path_dsc: &'static str,
        tokens: Vec<&'static str>,
    }

    struct NameValueDscInput {
        path_dsc: &'static str,
        value_dsc: &'static str,
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

    fn cfg_with_rename(rename: bool) -> ObfuscateConfig {
        ObfuscateConfig {
            obfuscation: ObfuscationSection {
                strings: false,
                min_string_length: None,
                ignore_strings: None,
                control_flow: false,
                skip_files: None,
                skip_attributes: None,
            },
            identifiers: Some(crate::config::IdentifiersSection {
                rename,
                preserve: None,
            }),
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
            renamed_idents: HashMap::new(),
            obfuscated_vars: HashSet::new(),
            used_obfuscate_str: false,
            used_obfuscate_string: false,
            used_obfuscate_flow: false,
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
            lit: Lit::Str(syn::LitStr::new(str, Span::call_site())),
        };
        Expr::Lit(literal)
    }

    /// Creates `syn::Attribute`.
    fn create_attribute(is_inner: bool, input: AttrInput) -> Attribute {
        let style = match is_inner {
            true => AttrStyle::Inner(Token![!](Span::call_site())),
            false => AttrStyle::Outer,
        };

        let meta: Meta = match input {
            AttrInput::PathDsc(p_name) => {
                Meta::Path(syn::Path::from(Ident::new(p_name, Span::call_site())))
            }
            AttrInput::ListDsc(input) => {
                let literals = input.tokens.iter().map(|s| quote! { #s });
                let tokens = quote! { #(#literals),* };
                Meta::List(syn::MetaList {
                    path: syn::Path::from(Ident::new(input.path_dsc, Span::call_site())),
                    delimiter: syn::MacroDelimiter::Paren(token::Paren(Span::call_site())),
                    tokens: tokens,
                })
            }
            AttrInput::NameValueDsc(input) => Meta::NameValue(syn::MetaNameValue {
                path: syn::Path::from(Ident::new(input.path_dsc, Span::call_site())),
                eq_token: Token![=](Span::call_site()),
                value: get_str_lit_expression(input.value_dsc),
            }),
        };
        let attr = Attribute {
            pound_token: Token![#](Span::call_site()),
            style: style,
            bracket_token: token::Bracket(Span::call_site()),
            meta: meta,
        };
        return attr;
    }

    fn verify_simple_stmt_after_flow_mut(stmt: Option<&Stmt>) {
        match stmt {
            Some(Stmt::Macro(StmtMacro {
                attrs: _,
                mac,
                semi_token: _,
            })) => {
                let ident = &mac.path.segments.last().unwrap().ident;
                assert_eq!(ident.to_string(), "obfuscate_flow");
            }
            Some(Stmt::Expr(
                Expr::Lit(ExprLit {
                    lit: Lit::Str(_), ..
                }),
                _,
            )) => {
                panic!("The first element must be the `obfuscate_flow` macro.\nIt cannot leave the original expression at the first index of block statements.");
            }
            Some(_) => {
                panic!("The first element must be the `obfuscate_flow` macro\nInstead, it creates unexpected expressions.");
            }
            None => {
                panic!("Unexpected behavior in `then_branch`");
            }
        }
    }

    fn _print_tokens<T: quote::ToTokens>(input: T) {
        let tokens = quote!(#input);
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

        assert_eq!(out.trim(), src.trim());
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

        while let Some(line) = lines.next() {
            if line.starts_with("pub const TEST_1") {
                assert_eq!(line, r#"pub const TEST_1: &str = "long enough test";"#);
                let line_3 = lines.next().unwrap();
                assert_eq!(line_3, r#"pub const TEST_2: &str = "test";"#);
                break;
            }
        }
    }

    #[test]
    fn process_file_does_not_apply_file_skip_rules() {
        let src = r#"pub fn message() {
    let text: &str = "secret";
}"#;

        let (_dir, path, relative_path) = create_rs_file(src);
        let skipped_files = Some(vec![String::from(relative_path.to_str().unwrap())]);

        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(true, None, None, false, skipped_files, None),
            false,
        )
        .unwrap();

        assert!(out.contains("obfuscate_str!(\"secret\")"));
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
    fn inner_docs_stay_before_inserted_macro_imports() {
        let src = r#"//! Crate docs
//! More docs

pub fn hello() {
    let message: &str = "hello";
}
"#;
        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(true, None, None, false, None, Some(false)),
            false,
        )
        .unwrap();

        let first_use = out
            .find("use rust_code_obfuscator::obfuscate_str;")
            .unwrap();
        let first_item = out.find("pub fn hello").unwrap();

        assert!(
            out.starts_with("//! Crate docs\n//! More docs"),
            "inner docs must remain first:\n{out}"
        );
        assert!(
            first_use < first_item,
            "macro import should be inserted before items but after inner docs:\n{out}"
        );
    }

    #[test]
    fn obfuscate_string_import_does_not_hide_missing_obfuscate_str_import() {
        let src = r#"use rust_code_obfuscator::obfuscate_string;

pub fn hello() {
    let message: &str = "hello";
}
"#;
        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg(true, None, None, false, None, None),
            false,
        )
        .unwrap();

        assert!(
            out.contains("use rust_code_obfuscator::obfuscate_str;"),
            "missing exact obfuscate_str import:\n{out}"
        );
    }

    #[test]
    fn renaming_updates_simple_function_and_local_references() {
        let src = r#"pub fn add(value: u32) -> u32 {
    let total = value + 1;
    total
}

pub fn call() -> u32 {
    add(41)
}
"#;
        let (_dir, path, relative_path) = create_rs_file(src);
        let (out, _changed, _before) = super::process_file(
            &path,
            relative_path.as_path(),
            &cfg_with_rename(true),
            false,
        )
        .unwrap();

        assert!(
            out.contains("pub fn add_obf_"),
            "function definition was not renamed:\n{out}"
        );
        assert!(
            out.contains("add_obf_"),
            "function call was not updated:\n{out}"
        );
        assert!(
            !out.contains("add(41)"),
            "old function call survived:\n{out}"
        );
        assert!(
            !out.contains("value + 1"),
            "old parameter reference survived:\n{out}"
        );
        assert!(
            !out.contains("\n    total\n"),
            "old local reference survived:\n{out}"
        );
    }

    #[test]
    fn obf_transformer_let_string_and_str() {
        let mut stmt: Stmt = syn::parse_quote! {
            let foo: &str = "hello";
        };
        let mut stmt2: Stmt = syn::parse_quote! {
            let bar: String = "world";
        };

        let mut transformer = obf_transformer(
            None, None, false, true, // obfuscate_strings
            false, false,
        );

        transformer.visit_stmt_mut(&mut stmt);
        transformer.visit_stmt_mut(&mut stmt2);

        // test foo: &str → obfuscate_str!
        if let Stmt::Local(local) = stmt {
            if let Some(LocalInit { expr, .. }) = local.init {
                match *expr {
                    Expr::Macro(mac) => {
                        assert_eq!(mac.mac.path.segments.last().unwrap().ident, "obfuscate_str");
                    }
                    _ => panic!("foo must use obfuscate_str!"),
                }
            } else {
                panic!("foo must have init");
            }
        } else {
            panic!("stmt must be Local");
        }

        // test bar: String → obfuscate_string!
        if let Stmt::Local(local) = stmt2 {
            if let Some(LocalInit { expr, .. }) = local.init {
                match *expr {
                    Expr::Macro(mac) => {
                        assert_eq!(
                            mac.mac.path.segments.last().unwrap().ident,
                            "obfuscate_string"
                        );
                    }
                    _ => panic!("bar must use obfuscate_string!"),
                }
            } else {
                panic!("bar must have init");
            }
        } else {
            panic!("stmt2 must be Local");
        }
    }

    #[test]
    fn obf_transformer_test_visit_stmt() {
        let ident_name = "foo";
        let number_of_idents = 1;
        let mut stmt: Stmt = syn::parse_quote! {
            let foo: &str = obfuscate_string!("test");
        };

        let mut transformer = obf_transformer(None, None, false, true, false, false);

        transformer.visit_stmt_mut(&mut stmt);
        let mut idents = transformer.obfuscated_vars.iter();

        assert_eq!(transformer.obfuscated_vars.len(), number_of_idents);
        assert_eq!(idents.next().unwrap(), ident_name);
    }

    #[test]
    fn obf_transformer_test_skip_attribute_mut() {
        let input = AttrInput::PathDsc("my_path");
        let mut attr_1: Attribute = create_attribute(true, input);

        let input = AttrInput::ListDsc(ListDscInput {
            path_dsc: "my_list_path",
            tokens: vec!["foo_1", "foo_2", "foo_3"],
        });
        let mut attr_2: Attribute = create_attribute(true, input);

        let name_value_3 = "my secret doc";
        let input = AttrInput::NameValueDsc(NameValueDscInput {
            path_dsc: "doc",
            value_dsc: name_value_3,
        });
        let mut attr_3: Attribute = create_attribute(true, input);

        let mut transformer = obf_transformer(None, None, false, true, false, true);

        transformer.visit_attribute_mut(&mut attr_1);
        transformer.visit_attribute_mut(&mut attr_2);
        transformer.visit_attribute_mut(&mut attr_3);

        match attr_3.meta {
            Meta::NameValue(meta_name_value) => match meta_name_value.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) => {
                    assert_eq!(lit_str.value(), name_value_3);
                }
                _ => panic!("attr_3.meta value must be a string literal"),
            },
            _ => panic!("attr_3.meta must remain of type Meta::NameValue"),
        }
    }

    #[test]
    fn obf_transformer_expr_if_mut() {
        let mut if_expr: ExprIf =
            syn::parse2(quote! {if foo_1 > foo_2 { "foo" ; } else { "foo" ; }}).unwrap();

        let mut transformer = obf_transformer(None, None, false, false, true, true);

        transformer.visit_expr_if_mut(&mut if_expr);

        let mut stmts_iter = if_expr.then_branch.stmts.iter();
        verify_simple_stmt_after_flow_mut(stmts_iter.next());

        let (_, else_branch) = if_expr.else_branch.unwrap();
        let stmts = if let Expr::Block(ExprBlock {
            attrs: _,
            label: _,
            block: Block {
                brace_token: _,
                stmts,
            },
        }) = *else_branch
        {
            stmts
        } else {
            panic!("Unexpected behavior - missing `else_branch`");
        };
        let mut stmts_iter = stmts.iter();

        verify_simple_stmt_after_flow_mut(stmts_iter.next());
    }

    #[test]
    fn obf_transformer_expr_match_mut() {
        let mut match_expr: ExprMatch =
            syn::parse2(quote! {match foo_1 {Some(foo_2)=>{"foo_1";},None=>{"foo_2";}}}).unwrap();

        let mut transformer = obf_transformer(None, None, false, false, true, true);

        transformer.visit_expr_match_mut(&mut match_expr);

        let mut arms = match_expr.arms.iter();

        let arms_next = arms.next().unwrap().body.clone();
        let inside_arm = if let Expr::Block(expr_blok) = *arms_next {
            expr_blok.block.stmts
        } else {
            panic!()
        };
        let mut stmts_iter = inside_arm.iter();
        verify_simple_stmt_after_flow_mut(stmts_iter.next());

        let arms_next = arms.next().unwrap().body.clone();
        let inside_arm = if let Expr::Block(expr_blok) = *arms_next {
            expr_blok.block.stmts
        } else {
            panic!()
        };
        let mut stmts_iter = inside_arm.iter();
        verify_simple_stmt_after_flow_mut(stmts_iter.next());
    }

    #[test]
    fn obf_transformer_expr_loop_mut() {
        let mut loop_expr: ExprLoop = syn::parse2(quote! {loop { "foo" ; }}).unwrap();

        let mut transformer = obf_transformer(None, None, false, false, true, true);

        transformer.visit_expr_loop_mut(&mut loop_expr);

        let mut stmts_iter = loop_expr.body.stmts.iter();
        verify_simple_stmt_after_flow_mut(stmts_iter.next());
    }

    #[test]
    fn obf_transformer_expr_while_mut() {
        let mut while_expr: ExprWhile = syn::parse2(quote! {while foo { "foo" ; }}).unwrap();

        let mut transformer = obf_transformer(None, None, false, false, true, true);

        transformer.visit_expr_while_mut(&mut while_expr);

        let mut stmts_iter = while_expr.body.stmts.iter();
        verify_simple_stmt_after_flow_mut(stmts_iter.next());
    }

    #[test]
    fn obf_transformer_expr_for_loop_mut() {
        let mut for_loop_expr: ExprForLoop = syn::parse2(quote! {for i in foo { i ; }}).unwrap();

        let mut transformer = obf_transformer(None, None, false, false, true, true);

        transformer.visit_expr_for_loop_mut(&mut for_loop_expr);

        let mut stmts_iter = for_loop_expr.body.stmts.iter();
        verify_simple_stmt_after_flow_mut(stmts_iter.next());
    }

    #[test]
    fn obf_transformer_pat_ident_mut() {
        let mut pat_ident_1: PatIdent = PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: Ident::new("foo", Span::call_site()),
            subpat: None,
        };
        let mut pat_ident_2: PatIdent = PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: Ident::new("foo", Span::call_site()),
            subpat: None,
        };

        let mut transformer = obf_transformer(None, None, true, false, false, true);

        transformer.visit_pat_ident_mut(&mut pat_ident_1);
        transformer.visit_pat_ident_mut(&mut pat_ident_2);

        let ident_1 = pat_ident_1.ident.to_string();
        let ident_first_5_1: String = ident_1.chars().take(5).collect();

        let ident_2 = pat_ident_2.ident.to_string();
        let ident_first_5_2: String = ident_2.chars().take(5).collect();
        // The same source identifier maps to the same obfuscated identifier so
        // references can be updated consistently.
        assert_eq!(ident_first_5_1, ident_first_5_2);
        assert_eq!(ident_1, ident_2);

        transformer.rename_identifiers = false;
        pat_ident_1.ident = Ident::new("foo", Span::call_site());
        pat_ident_2.ident = Ident::new("foo", Span::call_site());
        transformer.visit_pat_ident_mut(&mut pat_ident_1);
        transformer.visit_pat_ident_mut(&mut pat_ident_2);
        let ident_1 = pat_ident_1.ident.to_string();
        let ident_2 = pat_ident_2.ident.to_string();
        assert_eq!(ident_1, ident_2);

        transformer.rename_identifiers = true;
        transformer.preserve_idents = vec!["foo".to_string()];
        transformer.visit_pat_ident_mut(&mut pat_ident_1);
        transformer.visit_pat_ident_mut(&mut pat_ident_2);
        let ident_1 = pat_ident_1.ident.to_string();
        let ident_2 = pat_ident_2.ident.to_string();
        assert_eq!(ident_1, ident_2);
    }
}
