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

///
/// Macro to build a "standard" state machine without stopping
///  conditions.
///
/// This macro takes an identifier to use for the state of the
///  machine, an initial value for the state, and a closure taking
///  no arguments, which will be the main loop of the state machine.
///
/// Inside the closure, there should be at least one `choose!`
///  statement, which prompts the executor of the state machine
///  to choose between a finte number of action to perform, and
///  returns the action that was selected.
///
/// A `choose!` statement also yields the current value of the state
///  variable back to the caller.
///
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

    quote! {
        enstate::coroutines::AsMachine::new(
            #[coroutine]
            |_| {
                let mut #state_var = #initial_value;
                loop {
                    #transformed_body
                }
            }
        )
    }
    .into()
}

#[proc_macro]
pub fn machine_chain(input: TokenStream) -> TokenStream {
    let closure = parse_macro_input!(input as syn::ExprClosure);

    let body = closure.body;

    quote! {
        enstate::coroutines::AsChainMachine::new(
            #[coroutine]
            |_| {
                #body
            }
        )
    }
    .into()
}
