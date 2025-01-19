#![feature(coroutines, stmt_expr_attributes, coroutine_trait, trait_alias)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Result;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LocalInit, Stmt, Token, parse_macro_input, parse_quote};

// Structure to parse the macro input
struct MachineMacroInput {
    state_var: Ident,
    initial_value: Expr,
    body: Expr,
}

// Parser implementation
impl Parse for MachineMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let state_var = input.parse()?;
        input.parse::<Token![,]>()?;
        let initial_value = input.parse()?;
        input.parse::<Token![,]>()?;

        let closure = syn::ExprClosure::parse(input)?;

        let body = *closure.body;

        Ok(MachineMacroInput {
            state_var,
            initial_value,
            body,
        })
    }
}

#[proc_macro]
pub fn machine(input: TokenStream) -> TokenStream {
    let MachineMacroInput {
        state_var,
        initial_value,
        body,
    } = parse_macro_input!(input as MachineMacroInput);

    struct ChooseVisitor<'a> {
        state_var: &'a Ident,
    }

    impl<'a> syn::visit_mut::VisitMut for ChooseVisitor<'a> {
        fn visit_stmt_mut(&mut self, stmt: &mut syn::Stmt) {
            if let Stmt::Local(expr) = stmt {
                if let Some(ref init) = expr.init {
                    if let Expr::Macro(macro_exp) = &*init.expr {
                        if macro_exp.mac.path.is_ident("choose") {
                            let content = macro_exp.mac.tokens.clone();
                            let state_var = self.state_var;
                            let new_expr = parse_quote! {
                                yield (#state_var, [#content].as_slice())
                            };

                            expr.init = Some(LocalInit {
                                eq_token: init.eq_token,
                                expr: new_expr,
                                diverge: init.diverge.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    let mut transformed_body = body;
    let mut visitor = ChooseVisitor {
        state_var: &state_var,
    };
    syn::visit_mut::visit_expr_mut(&mut visitor, &mut transformed_body);

    let expanded = quote! {
        #[coroutine]
        |_| {
            let mut #state_var = #initial_value;
            loop {
                #transformed_body
            }
        }
    };

    expanded.into()
}
