use anyhow::Result;
use std::fs;
use syn::{
    parse_file, visit_mut::VisitMut, Expr, ExprLit, Lit,
    ExprIf, ExprMatch, ExprLoop, ExprWhile, ExprForLoop, Stmt,
};
use quote::{quote, quote_spanned};

pub fn process_file(path: &std::path::Path) -> Result<String> {
    let source = fs::read_to_string(path)?;
    let mut syntax_tree = parse_file(&source)?;

    let mut transformer = ObfuscationTransformer;
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

struct ObfuscationTransformer;

impl VisitMut for ObfuscationTransformer {
    // STRING
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = expr {
            let value = lit_str.value();
            let span = lit_str.span();

            let wrapped: Expr = syn::parse2(quote_spanned! {span=>
                obfuscate_string!(#value)
            }).unwrap();

            *expr = wrapped;
        }

        syn::visit_mut::visit_expr_mut(self, expr);
    }

    // IF / ELSE
    fn visit_expr_if_mut(&mut self, node: &mut ExprIf) {
        let inject: Stmt = syn::parse_quote! {
            obfuscate_flow!();
        };
        node.then_branch.stmts.insert(0, inject.clone());
        if let Some((_, else_branch)) = &mut node.else_branch {
            if let Expr::Block(block) = else_branch.as_mut() {
                block.block.stmts.insert(0, inject.clone());
            }
        }
        syn::visit_mut::visit_expr_if_mut(self, node);
    }

    // MATCH
    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        for arm in &mut node.arms {
            let original = &arm.body;
    
            arm.body = Box::new(syn::parse_quote!({
                obfuscate_flow!();
                #original
            }));
        }
    
        syn::visit_mut::visit_expr_match_mut(self, node);
    }

    // LOOP
    fn visit_expr_loop_mut(&mut self, node: &mut ExprLoop) {
        let inject: Stmt = syn::parse_quote! {
            obfuscate_flow!();
        };
        node.body.stmts.insert(0, inject);
        syn::visit_mut::visit_expr_loop_mut(self, node);
    }

    // WHILE
    fn visit_expr_while_mut(&mut self, node: &mut ExprWhile) {
        let inject: Stmt = syn::parse_quote! {
            obfuscate_flow!();
        };
        node.body.stmts.insert(0, inject);
        syn::visit_mut::visit_expr_while_mut(self, node);
    }

    // FOR
    fn visit_expr_for_loop_mut(&mut self, node: &mut ExprForLoop) {
        let inject: Stmt = syn::parse_quote! {
            obfuscate_flow!();
        };
        node.body.stmts.insert(0, inject);
        syn::visit_mut::visit_expr_for_loop_mut(self, node);
    }
}
