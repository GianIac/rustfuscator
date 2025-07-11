use std::path::Path;
use anyhow::Result;
use std::fs;
use quote::{quote, quote_spanned};
use syn::{
    parse_file,
    visit_mut::VisitMut,
    Expr, ExprLit, Lit,
    ExprIf, ExprMatch, ExprLoop, ExprWhile, ExprForLoop, Stmt,
    Ident, ItemFn, PatIdent,
};

use globset::{Glob, GlobSetBuilder};

use rust_code_obfuscator_core::utils::generate_obf_suffix;

use crate::config::ObfuscateConfig;

pub fn process_file(path: &Path, relative_path: &Path, config: &ObfuscateConfig) -> Result<String> {
    let source = fs::read_to_string(path)?;

    let file_name = relative_path.to_string_lossy();
    if let Some(skip_list) = &config.obfuscation.skip_files {
        if skip_list.iter().any(|entry| file_name.ends_with(entry)) {
            return Ok(source);
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
            return Ok(source);
        }

        if let Some(exclude_patterns) = &include.exclude {
            for pattern in exclude_patterns {
                let exclude_glob = Glob::new(pattern)?;
                if exclude_glob.compile_matcher().is_match(path) {
                    return Ok(source);
                }
            }
        }
    }

    let mut syntax_tree = parse_file(&source)?;

    let mut transformer = ObfuscationTransformer {
        min_string_length: config.obfuscation.min_string_length,
        ignore_strings: config.obfuscation.ignore_strings.clone(),
        rename_identifiers: config.identifiers.as_ref().map(|id| id.rename).unwrap_or(false),
        preserve_idents: config.identifiers.as_ref().and_then(|id| id.preserve.clone()).unwrap_or_default(),
        obfuscate_strings: config.obfuscation.strings,
        obfuscate_flow: config.obfuscation.control_flow,
        skip_docstrings: config.obfuscation.skip_docstrings.unwrap_or(false),
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

    let tokens = if has_use {
        quote!(#syntax_tree)
    } else {
        quote! {
            extern crate rust_code_obfuscator;
            use rust_code_obfuscator::{obfuscate_string, obfuscate_flow};
            #syntax_tree
        }
    };

    Ok(tokens.to_string())
}

struct ObfuscationTransformer {
    min_string_length: Option<usize>,
    ignore_strings: Option<Vec<String>>,
    rename_identifiers: bool,
    preserve_idents: Vec<String>,
    obfuscate_strings: bool,
    obfuscate_flow: bool,
    skip_docstrings: bool,
}

impl VisitMut for ObfuscationTransformer {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if !self.obfuscate_strings {
            return syn::visit_mut::visit_expr_mut(self, expr);
        }

        if let Expr::Lit(ExprLit { lit: Lit::Str(ref lit_str), .. }) = expr {
            let value = lit_str.value();

            if let Some(min_len) = self.min_string_length {
                if value.len() < min_len {
                    return;
                }
            }

            if let Some(ref ignores) = self.ignore_strings {
                if ignores.iter().any(|s| s == &value) {
                    return;
                }
            }

            let span = lit_str.span();
            let wrapped: Expr = syn::parse2(quote_spanned! {span=> obfuscate_string!(#value) }).unwrap();
            *expr = wrapped;
        }

        syn::visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        if !self.obfuscate_strings {
            return syn::visit_mut::visit_stmt_mut(self, stmt);
        }

        if let Stmt::Local(local) = stmt {
            if let Some(init) = &mut local.init {
                if let Expr::Lit(ExprLit { lit: Lit::Str(ref lit_str), .. }) = *init.expr {
                    let value = lit_str.value();

                    if let Some(min_len) = self.min_string_length {
                        if value.len() < min_len {
                            return;
                        }
                    }

                    if let Some(ref ignores) = self.ignore_strings {
                        if ignores.iter().any(|s| s == &value) {
                            return;
                        }
                    }

                    let span = lit_str.span();
                    let wrapped: Expr = syn::parse2(quote_spanned! {span=> obfuscate_string!(#value) }).unwrap();
                    init.expr = Box::new(wrapped);
                }
            }
        }

    fn visit_attribute_mut(&mut self, attr: &mut syn::Attribute) {
        if self.skip_docstrings {
            // Skip doc comments like this --> #[doc = "fratm"]
            if attr.path().is_ident("doc") {
                return;
            }
        }
        syn::visit_mut::visit_attribute_mut(self, attr);
    }
        

        syn::visit_mut::visit_stmt_mut(self, stmt);
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
