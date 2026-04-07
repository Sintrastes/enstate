#![feature(coroutines, stmt_expr_attributes, coroutine_trait, trait_alias)]

use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::{Spacing, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::token::LArrow;
use syn::{Expr, Ident, Pat, Token};
use syn::{LocalInit, parse_macro_input, parse_quote};

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
            if let syn::Stmt::Local(expr) = stmt {
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
        enstate::coroutines::ReplayMachine::new(|| {
            #[coroutine]
            |_| {
                let mut #state_var = #initial_value;
                loop {
                    #transformed_body
                }
            }
        })
    }
    .into()
}

#[proc_macro]
pub fn machine_chain(input: TokenStream) -> TokenStream {
    let closure = parse_macro_input!(input as syn::ExprClosure);

    let body = closure.body;

    quote! {
        enstate::coroutines::ReplayChainMachine::new(|| {
            #[coroutine]
            |_| {
                #body
            }
        })
    }
    .into()
}

//////////////////////// Proc syntax //////////////////////////////

// Returns true if `expr` is exactly the identifier `ident` (and nothing else).
fn expr_is_ident(expr: &Expr, ident: &Ident) -> bool {
    if let Expr::Path(p) = expr {
        if p.attrs.is_empty() && p.qself.is_none() {
            let segs = &p.path.segments;
            return segs.len() == 1
                && segs[0].ident == *ident
                && matches!(segs[0].arguments, syn::PathArguments::None);
        }
    }
    false
}

// Collect all identifiers (as strings) appearing anywhere in `expr`.
// Used to detect forward references inside `rec` blocks.
fn expr_idents(expr: &Expr) -> HashSet<String> {
    struct Collector(HashSet<String>);
    impl<'ast> syn::visit::Visit<'ast> for Collector {
        fn visit_ident(&mut self, i: &'ast Ident) {
            self.0.insert(i.to_string());
        }
    }
    let mut c = Collector(HashSet::new());
    syn::visit::visit_expr(&mut c, expr);
    c.0
}

// Replace simple identifier references in `expr` according to `mappings`.
// Used inside `rec` inner machines to rewrite references to already-computed
// feedback vars so they resolve to the current-step output ident.
fn subst_idents(expr: &mut Expr, mappings: &[(Ident, Ident)]) {
    if mappings.is_empty() {
        return;
    }
    struct Subst<'a>(&'a [(Ident, Ident)]);
    impl<'a> syn::visit_mut::VisitMut for Subst<'a> {
        fn visit_expr_path_mut(&mut self, p: &mut syn::ExprPath) {
            if p.attrs.is_empty() && p.qself.is_none() {
                let segs = &mut p.path.segments;
                if segs.len() == 1 && matches!(segs[0].arguments, syn::PathArguments::None) {
                    for (orig, repl) in self.0 {
                        if segs[0].ident == *orig {
                            segs[0].ident = repl.clone();
                            return;
                        }
                    }
                }
            }
        }
    }
    syn::visit_mut::visit_expr_mut(&mut Subst(mappings), expr);
}

// Given the ordered bindings of a `rec` block and the set of all vars defined
// in that block, return the feedback vars: those that are referenced in some
// binding's *input* expression before their own binding appears.
fn detect_feedback_vars(stmts: &[Stmt], rec_vars: &[Ident]) -> Vec<Ident> {
    let rec_var_names: HashSet<String> = rec_vars.iter().map(|v| v.to_string()).collect();
    let mut computed: HashSet<String> = HashSet::new();
    let mut seen_as_feedback: HashSet<String> = HashSet::new();
    let mut result: Vec<Ident> = Vec::new();

    for stmt in stmts {
        let (var, refs) = match stmt {
            Stmt::Bind { var, input, .. } => (var, expr_idents(input)),
            Stmt::Let { var, expr, .. } => (var, expr_idents(expr)),
            _ => continue,
        };
        for ident_str in refs {
            if rec_var_names.contains(&ident_str)
                && !computed.contains(&ident_str)
                && !seen_as_feedback.contains(&ident_str)
            {
                seen_as_feedback.insert(ident_str.clone());
                let orig = rec_vars
                    .iter()
                    .find(|v| v.to_string() == ident_str)
                    .unwrap();
                result.push(orig.clone());
            }
        }
        computed.insert(var.to_string());
    }

    result
}

// A pipeline is "linear" when every bind's input expression is exactly the
// immediately preceding variable, and `returnA` is the last statement.
// In that case the whole proc desugars to a plain compose chain with no
// fanout or environment tuples.
fn is_linear_pipeline(input_pat: &Pat, stmts: &[Stmt]) -> bool {
    let first_var = match input_pat {
        Pat::Ident(p) => &p.ident,
        _ => return false,
    };
    let mut prev: &Ident = first_var;
    for stmt in stmts {
        match stmt {
            Stmt::Bind { var, input, .. } => {
                if !expr_is_ident(input, prev) {
                    return false;
                }
                prev = var;
            }
            Stmt::RecBlock { .. } | Stmt::Let { .. } => return false,
            Stmt::Return { expr } => {
                if !expr_is_ident(expr, prev) {
                    return false;
                }
            }
        }
    }
    true
}

// Scan tokens until we find `-<` at the top level (not inside any group),
// returning the arrow expression.  The `-<` itself is left unconsumed.
//
// Key insight: `(…)`, `[…]`, `{…}` are each a *single* `TokenTree::Group`
// node, so any `-<` buried inside a function call or closure is invisible
// to the scan and we never stop prematurely.
//
// To distinguish the arrow operator from binary subtraction followed by
// less-than (`a - <b`), we require the `-` to have `Spacing::Joint`,
// meaning it is immediately followed (no whitespace) by the `<`.
fn parse_arrow_expr(input: ParseStream) -> Result<Expr> {
    let tokens: proc_macro2::TokenStream = input.step(|cursor| {
        let mut rest = *cursor;
        let mut collected: Vec<TokenTree> = Vec::new();

        loop {
            // Stop at `-<` — Joint minus immediately followed by `<`.
            if let Some((punct, next)) = rest.punct() {
                if punct.as_char() == '-' && punct.spacing() == Spacing::Joint {
                    if let Some((punct2, _)) = next.punct() {
                        if punct2.as_char() == '<' {
                            let ts: proc_macro2::TokenStream = collected.into_iter().collect();
                            return Ok((ts, rest));
                        }
                    }
                }
            }

            // No separator yet — consume one token tree and keep scanning.
            match rest.token_tree() {
                Some((tt, next)) => {
                    collected.push(tt);
                    rest = next;
                }
                None => return Err(syn::Error::new(rest.span(), "expected `-<`")),
            }
        }
    })?;

    syn::parse2(tokens)
}

// Parse a type annotation that appears before `<-` in a bind statement.
//
// Unlike `syn::Type::parse` (which would greedily consume `<-` as a generic
// argument list) and `syn::Path::parse_mod_style` (which refuses angle brackets
// entirely), this function handles types with generics like `Event<()>` by
// collecting tokens up to `<-` at the top level, then re-parsing them as a
// `syn::Type`.
//
// We detect `<-` the same way as `parse_arrow_expr`: a `-` punct with
// `Spacing::Joint` immediately followed by `<`.  Any `<` that is NOT
// immediately preceded by a joint `-` is treated as the start of a generic
// argument list rather than the arrow operator.
fn parse_type_before_larrow(input: ParseStream) -> Result<syn::Type> {
    let tokens: proc_macro2::TokenStream = input.step(|cursor| {
        let mut rest = *cursor;
        let mut collected: Vec<TokenTree> = Vec::new();

        loop {
            // Check for `<-` (LArrow) — a joint `<` immediately followed by `-`.
            if let Some((punct, next)) = rest.punct() {
                if punct.as_char() == '<' && punct.spacing() == Spacing::Joint {
                    if let Some((punct2, _)) = next.punct() {
                        if punct2.as_char() == '-' {
                            // Found `<-` — stop, don't consume it.
                            let ts: proc_macro2::TokenStream = collected.into_iter().collect();
                            return Ok((ts, rest));
                        }
                    }
                }
            }

            // Consume one token tree and keep scanning.
            match rest.token_tree() {
                Some((tt, next)) => {
                    collected.push(tt);
                    rest = next;
                }
                None => {
                    return Err(syn::Error::new(
                        rest.span(),
                        "expected `<-` after type annotation",
                    ));
                }
            }
        }
    })?;

    syn::parse2(tokens)
}

// AST for the `proc` block
struct ArrowProc {
    input_pat: Pat,
    input_type: Option<syn::Type>,
    body: Vec<Stmt>,
}

enum Stmt {
    Bind {
        var: Ident,
        /// Optional type annotation written as `var: Type <- machine -< input;`.
        /// When present, the macro generates a `let var: Type = var;` re-bind
        /// inside the flatten closure so that field access on `var` type-checks
        /// without requiring Rust to propagate the concrete type through `impl
        /// Machine` boundaries.
        var_type: Option<syn::Type>,
        arrow: Expr,
        input: Expr,
    },
    Let {
        var: Ident,
        var_type: Option<syn::Type>,
        expr: Expr,
    },
    RecBlock {
        init: Option<Expr>,
        body: Vec<Stmt>, // only Bind stmts allowed inside
    },
    Return {
        expr: Expr,
    },
}

/// Parses the `proc` syntax.
impl Parse for ArrowProc {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the pattern inside `| ... |`.
        // An optional type annotation (`|event: MyType|`) is stored and used
        // to type the initial `arr` closure, anchoring field access on the input.
        let _ = input.parse::<Token![|]>()?;
        let input_pat = Pat::parse_single(input)?;
        let input_type = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse::<syn::Type>()?)
        } else {
            None
        };
        let _ = input.parse::<Token![|]>()?;

        // Parse the body inside `{ ... }`
        let content;
        let _ = syn::braced!(content in input);
        let mut body = Vec::new();

        while !content.is_empty() {
            if content.peek(Token![let]) {
                // let binding: let x [: Type] = expr;
                content.parse::<Token![let]>()?;
                let var: Ident = content.parse()?;
                let var_type = if content.peek(Token![:]) {
                    content.parse::<Token![:]>()?;
                    Some(content.parse::<syn::Type>()?)
                } else {
                    None
                };
                content.parse::<Token![=]>()?;
                let expr: Expr = content.parse()?;
                content.parse::<Token![;]>()?;
                body.push(Stmt::Let {
                    var,
                    var_type,
                    expr,
                });
            } else if content.peek(Ident) && (content.peek2(LArrow) || content.peek2(Token![:])) {
                // Bind statement: ident [: Type] <- arrow_expr -< input_expr ;
                let var: Ident = content.parse()?;
                let var_type = if content.peek(Token![:]) {
                    content.parse::<Token![:]>()?;
                    Some(parse_type_before_larrow(&content)?)
                } else {
                    None
                };
                content.parse::<LArrow>()?;
                let arrow = parse_arrow_expr(&content)?;
                content.parse::<Token![-]>()?;
                content.parse::<Token![<]>()?;
                let input: Expr = content.parse()?;
                content.parse::<Token![;]>()?;
                body.push(Stmt::Bind {
                    var,
                    var_type,
                    arrow,
                    input,
                });
            } else if content.peek(Ident) {
                let kw: Ident = content.parse()?;
                if kw == "rec" {
                    // rec { ... };  or  rec(init_expr) { ... };
                    let init = if content.peek(syn::token::Paren) {
                        let inner;
                        syn::parenthesized!(inner in content);
                        Some(inner.parse::<Expr>()?)
                    } else {
                        None
                    };

                    // Parse the rec block body (Bind and Let stmts allowed)
                    let block_content;
                    let _ = syn::braced!(block_content in content);
                    let mut rec_body: Vec<Stmt> = Vec::new();
                    while !block_content.is_empty() {
                        if block_content.peek(Token![let]) {
                            block_content.parse::<Token![let]>()?;
                            let var: Ident = block_content.parse()?;
                            let var_type = if block_content.peek(Token![:]) {
                                block_content.parse::<Token![:]>()?;
                                Some(block_content.parse::<syn::Type>()?)
                            } else {
                                None
                            };
                            block_content.parse::<Token![=]>()?;
                            let expr: Expr = block_content.parse()?;
                            block_content.parse::<Token![;]>()?;
                            rec_body.push(Stmt::Let {
                                var,
                                var_type,
                                expr,
                            });
                        } else if block_content.peek(Ident)
                            && (block_content.peek2(LArrow) || block_content.peek2(Token![:]))
                        {
                            let var: Ident = block_content.parse()?;
                            let var_type = if block_content.peek(Token![:]) {
                                block_content.parse::<Token![:]>()?;
                                Some(parse_type_before_larrow(&block_content)?)
                            } else {
                                None
                            };
                            block_content.parse::<LArrow>()?;
                            let arrow = parse_arrow_expr(&block_content)?;
                            block_content.parse::<Token![-]>()?;
                            block_content.parse::<Token![<]>()?;
                            let input: Expr = block_content.parse()?;
                            block_content.parse::<Token![;]>()?;
                            rec_body.push(Stmt::Bind {
                                var,
                                var_type,
                                arrow,
                                input,
                            });
                        } else {
                            return Err(block_content
                                .error("expected a bind or let statement inside `rec` block"));
                        }
                    }

                    content.parse::<Token![;]>()?;
                    body.push(Stmt::RecBlock {
                        init,
                        body: rec_body,
                    });
                } else if kw == "returnA" {
                    // returnA -< expr ;
                    content.parse::<Token![-]>()?;
                    content.parse::<Token![<]>()?;
                    let expr: Expr = content.parse()?;
                    content.parse::<Token![;]>()?;
                    body.push(Stmt::Return { expr });
                } else {
                    return Err(syn::Error::new(kw.span(), "expected `returnA` or `rec`"));
                }
            } else {
                return Err(content.error("expected a bind or return statement"));
            }
        }

        Ok(ArrowProc {
            input_pat,
            input_type,
            body,
        })
    }
}

// Shared bind-step code: given the current env vars, a user arrow expression,
// and the user input expression, produce the (step, new_vars) pair for the
// general non-linear path.
//
// `pending_lets` are `let x = expr` statements that immediately follow this
// bind in the source.  Folding them into the flatten step means their
// expressions run inside the pattern-destructuring closure where every
// variable's concrete type is already known — avoiding E0282 errors that
// occur when field-access expressions (`vel.value`) appear in a separate
// `arr` closure whose input type must be inferred through `impl Machine`.
fn make_bind_step(
    vars: &[Ident],
    arrow: &proc_macro2::TokenStream,
    input: &Expr,
    output_var: &Ident,
    pending_lets: &[(Ident, Option<syn::Type>, Expr)],
    // Extra `let var: Type = var;` re-bind statements inserted at the top of
    // the flatten body.  These anchor concrete types so that field-access
    // expressions in `pending_lets` can type-check even though Rust cannot
    // propagate types backward through `impl Machine` boundaries.
    rebind_stmts: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    let env_pat = quote! { (#(#vars,)*) };
    let env_ident = Ident::new("__arrow_env", proc_macro2::Span::mixed_site());
    let input_arrow = quote! {
        arr(|#env_ident| {
            #[allow(unused_variables)]
            let #env_pat = #env_ident;
            #input
        })
    };
    let composed_arrow = quote! { compose(#input_arrow, #arrow) };
    let keep_env = quote! { arr(|#env_pat| (#(#vars,)*)) };
    let fanouted = quote! { fanout(#keep_env, #composed_arrow) };
    // Flatten: env + new result → extended env, binding result as output_var.
    // When pending lets are present, evaluate them inside the flatten closure.
    // Re-bind statements anchor struct types for field access.
    let flatten = if pending_lets.is_empty() {
        quote! { arr(|(#env_pat, #output_var)| (#(#vars,)* #output_var)) }
    } else {
        let let_stmts: Vec<_> = pending_lets
            .iter()
            .map(|(v, vt, e)| {
                if let Some(t) = vt {
                    quote! { let #v: #t = #e; }
                } else {
                    quote! { let #v = #e; }
                }
            })
            .collect();
        let pl_vars: Vec<&Ident> = pending_lets.iter().map(|(v, _, _)| v).collect();
        quote! {
            arr(|(#env_pat, #output_var)| {
                #(#rebind_stmts)*
                #(#let_stmts)*
                (#(#vars,)* #output_var, #(#pl_vars,)*)
            })
        }
    };
    quote! { compose(#fanouted, #flatten) }
}

/// Pending bind tuple: (var, var_type, arrow_tokens, input_expr).
type PendingBind = (Ident, Option<syn::Type>, proc_macro2::TokenStream, Expr);

// Flush a buffered bind (and any trailing lets) into a composed machine step.
// Called whenever we encounter the next Bind, a RecBlock, or Return.
//
// When there are pending lets, type re-bind statements are generated from:
// - `input_type`: the input variable's declared type (from `|event: Type|`)
// - the bind's `var_type`: the user's annotation (from `vel: Vel <- ...`)
fn flush_pending_bind(
    pending_bind: &mut Option<PendingBind>,
    pending_lets: &mut Vec<(Ident, Option<syn::Type>, Expr)>,
    vars: &mut Vec<Ident>,
    composed: &mut proc_macro2::TokenStream,
    input_type: Option<&syn::Type>,
) {
    if let Some((pb_var, pb_var_type, pb_arrow, pb_input)) = pending_bind.take() {
        let pl = std::mem::take(pending_lets);

        // Build re-bind stmts to anchor types for field access in pending lets.
        let mut rebind_stmts = Vec::new();
        if !pl.is_empty() {
            // Re-bind the input variable with its declared type.
            if let Some(it) = input_type {
                if let Some(first_var) = vars.first() {
                    rebind_stmts.push(quote! {
                        #[allow(unused_variables)]
                        let #first_var: #it = #first_var;
                    });
                }
            }
            // Re-bind the bind output with the user-provided type annotation.
            if let Some(ref vt) = pb_var_type {
                rebind_stmts.push(quote! {
                    #[allow(unused_variables)]
                    let #pb_var: #vt = #pb_var;
                });
            }
        }

        let step = make_bind_step(&vars, &pb_arrow, &pb_input, &pb_var, &pl, &rebind_stmts);
        *composed = quote! { compose(#composed, #step) };
        vars.push(pb_var);
        for (pv, _, _) in pl {
            vars.push(pv);
        }
    }
}

#[proc_macro]
pub fn proc(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as ArrowProc);

    // Extract the first variable from the input pattern (assume single identifier)
    let first_var = match &ast.input_pat {
        Pat::Ident(pat_ident) => pat_ident.ident.clone(),
        _ => {
            return syn::Error::new(
                ast.input_pat.span(),
                "input pattern must be a single identifier (for now)",
            )
            .to_compile_error()
            .into();
        }
    };

    // Optimised path: linear pipeline  (arrowp "trivial" case)
    if is_linear_pipeline(&ast.input_pat, &ast.body) {
        let arrows: Vec<_> = ast
            .body
            .iter()
            .filter_map(|s| match s {
                Stmt::Bind { arrow, .. } => Some(quote! { #arrow }),
                Stmt::RecBlock { .. } | Stmt::Let { .. } | Stmt::Return { .. } => None,
            })
            .collect();

        let composed = if arrows.is_empty() {
            let ip = &ast.input_pat;
            quote! { arr(|#ip| #ip) }
        } else {
            arrows
                .into_iter()
                .reduce(|left, right| quote! { compose(#left, #right) })
                .unwrap()
        };

        return TokenStream::from(quote! { { #composed } });
    }

    let mut vars = vec![first_var];

    let input_pat = &ast.input_pat;

    // General path (non-linear): grow an environment tuple via fanout.
    //
    // Drain any leading `let` statements and fold them directly into the
    // initial `arr` closure, so that the input type annotation (if present)
    // is in scope for their expressions.  This is essential for field access
    // on structured inputs (`event.dt`, `game_event.direction`, etc.) where
    // Rust can't propagate the concrete type through a separate compose step.
    let mut body = ast.body.into_iter().peekable();
    let mut leading_lets: Vec<(Ident, Option<syn::Type>, Expr)> = Vec::new();
    while matches!(body.peek(), Some(Stmt::Let { .. })) {
        if let Some(Stmt::Let {
            var,
            var_type,
            expr,
        }) = body.next()
        {
            vars.push(var.clone());
            leading_lets.push((var, var_type, expr));
        }
    }

    let initial = {
        let ll: Vec<_> = leading_lets
            .iter()
            .map(|(v, vt, e)| {
                if let Some(t) = vt {
                    quote! { let #v: #t = #e; }
                } else {
                    quote! { let #v = #e; }
                }
            })
            .collect::<Vec<_>>();
        let all_vars = &vars;
        match &ast.input_type {
            Some(ty) => quote! { arr(|#input_pat: #ty| { #(#ll)* (#(#all_vars,)*) }) },
            None => quote! { arr(|#input_pat| { #(#ll)* (#(#all_vars,)*) }) },
        }
    };

    let mut composed = initial;

    // Buffer the most-recently-seen Bind so that any `let` stmts that
    // immediately follow it can be folded into its flatten step, giving
    // Rust concrete types for field-access expressions.
    let mut pending_bind: Option<PendingBind> = None;
    let mut pending_lets: Vec<(Ident, Option<syn::Type>, Expr)> = Vec::new();
    let input_type_ref = ast.input_type.as_ref();

    for stmt in body {
        match stmt {
            Stmt::Bind {
                var,
                var_type,
                arrow,
                input,
            } => {
                flush_pending_bind(
                    &mut pending_bind,
                    &mut pending_lets,
                    &mut vars,
                    &mut composed,
                    input_type_ref,
                );
                pending_bind = Some((var, var_type, quote! { #arrow }, input));
            }

            Stmt::Let {
                var,
                var_type,
                expr,
            } => {
                if pending_bind.is_some() {
                    // Accumulate: will be folded into the preceding bind's flatten step.
                    pending_lets.push((var, var_type, expr));
                } else {
                    // No preceding bind in scope (e.g. after a RecBlock).
                    // Fall back to a separate arr step.
                    let env_pat = quote! { (#(#vars,)*) };
                    let env_ident = Ident::new("__arrow_env", proc_macro2::Span::mixed_site());
                    let let_stmt = if let Some(ref t) = var_type {
                        quote! { let #var: #t = #expr; }
                    } else {
                        quote! { let #var = #expr; }
                    };
                    let step = quote! {
                        arr(|#env_ident| {
                            #[allow(unused_variables)]
                            let #env_pat = #env_ident;
                            #let_stmt
                            (#(#vars,)* #var)
                        })
                    };
                    composed = quote! { compose(#composed, #step) };
                    vars.push(var);
                }
            }

            Stmt::RecBlock {
                init,
                body: rec_body,
            } => {
                flush_pending_bind(
                    &mut pending_bind,
                    &mut pending_lets,
                    &mut vars,
                    &mut composed,
                    input_type_ref,
                );
                // ----------------------------------------------------------------
                // rec(init) { b1; b2; ... }
                //
                // Haskell-style ArrowLoop desugaring.  Forward-referenced
                // variables become the feedback loop, threaded via `cycle`.
                // ----------------------------------------------------------------

                // Step 1: all vars defined in the rec block (in declaration order)
                let rec_vars: Vec<Ident> = rec_body
                    .iter()
                    .filter_map(|s| match s {
                        Stmt::Bind { var, .. } | Stmt::Let { var, .. } => Some(var.clone()),
                        _ => None,
                    })
                    .collect();

                if rec_vars.is_empty() {
                    // Empty rec block — nothing to do.
                    continue;
                }

                // Step 2: detect feedback vars (forward-referenced in the block)
                let feedback_vars: Vec<Ident> = detect_feedback_vars(&rec_body, &rec_vars);

                if feedback_vars.is_empty() {
                    // No forward references — process bindings with the same
                    // pending-bind approach used in the outer loop.
                    let mut rec_pb: Option<PendingBind> = None;
                    let mut rec_pl: Vec<(Ident, Option<syn::Type>, Expr)> = Vec::new();
                    for stmt in rec_body {
                        match stmt {
                            Stmt::Bind {
                                var,
                                var_type,
                                arrow,
                                input,
                            } => {
                                flush_pending_bind(
                                    &mut rec_pb,
                                    &mut rec_pl,
                                    &mut vars,
                                    &mut composed,
                                    input_type_ref,
                                );
                                rec_pb = Some((var, var_type, quote! { #arrow }, input));
                            }
                            Stmt::Let {
                                var,
                                var_type,
                                expr,
                            } => {
                                if rec_pb.is_some() {
                                    rec_pl.push((var, var_type, expr));
                                } else {
                                    let env_pat = quote! { (#(#vars,)*) };
                                    let env_ident =
                                        Ident::new("__arrow_env", proc_macro2::Span::mixed_site());
                                    let let_stmt = if let Some(ref t) = var_type {
                                        quote! { let #var: #t = #expr; }
                                    } else {
                                        quote! { let #var = #expr; }
                                    };
                                    let step = quote! {
                                        arr(|#env_ident| {
                                            #[allow(unused_variables)]
                                            let #env_pat = #env_ident;
                                            #let_stmt
                                            (#(#vars,)* #var)
                                        })
                                    };
                                    composed = quote! { compose(#composed, #step) };
                                    vars.push(var);
                                }
                            }
                            _ => {}
                        }
                    }
                    flush_pending_bind(
                        &mut rec_pb,
                        &mut rec_pl,
                        &mut vars,
                        &mut composed,
                        input_type_ref,
                    );
                    continue;
                }

                // Step 3: build the inner machine over the flat augmented env
                //
                // Augmented env = outer_vars ++ feedback_vars (feedback vars carry
                // their previous-step values at the start of each traversal).
                //
                // For each feedback var `f`, the binding `f <- arrow -< input`
                // is processed normally but we rename the *output* to
                // `__rec_f_out` to avoid shadowing the feedback input `f` already
                // in the env.

                let fb_set: HashSet<String> = feedback_vars.iter().map(|v| v.to_string()).collect();

                // Map: original feedback var name → its fresh output ident
                let fb_out: Vec<(Ident, Ident)> = feedback_vars
                    .iter()
                    .map(|fv| {
                        let out = Ident::new(
                            &format!("__rec_{}_out", fv),
                            proc_macro2::Span::mixed_site(),
                        );
                        (fv.clone(), out)
                    })
                    .collect();

                let fb_out_name = |fv: &Ident| -> Ident {
                    fb_out
                        .iter()
                        .find(|(orig, _)| orig == fv)
                        .unwrap()
                        .1
                        .clone()
                };

                // Initial augmented vars for the inner machine
                let mut inner_vars: Vec<Ident> = vars
                    .iter()
                    .cloned()
                    .chain(feedback_vars.iter().cloned())
                    .collect();

                // Inner machine starts as identity on the flat augmented tuple
                let mut inner_composed = quote! { arr(|__x| __x) };

                // Track the most recent typed bind for re-bind injection
                let mut pending_rebind: Option<(Ident, syn::Type)> = None;

                for inner_stmt in rec_body {
                    let (out_var, step) = match inner_stmt {
                        Stmt::Bind {
                            var,
                            var_type,
                            arrow,
                            input,
                        } => {
                            let is_fb = fb_set.contains(&var.to_string());
                            let out_var = if is_fb {
                                fb_out_name(&var)
                            } else {
                                var.clone()
                            };
                            let step = make_bind_step(
                                &inner_vars,
                                &quote! { #arrow },
                                &input,
                                &out_var,
                                &[],
                                &[],
                            );
                            // Remember type annotation for subsequent let stmts.
                            // Use out_var (which is __rec_X_out for feedback vars)
                            // because fb_subst replaces the original name in exprs.
                            pending_rebind = var_type.map(|t| (out_var.clone(), t));
                            (out_var, step)
                        }
                        Stmt::Let {
                            var,
                            var_type,
                            expr,
                        } => {
                            // Substitute any already-computed feedback vars with their
                            // current-step output idents so the user's expression sees
                            // this step's value (not the previous-step feedback value).
                            let mut expr = expr;
                            let fb_subst: Vec<(Ident, Ident)> = fb_out
                                .iter()
                                .filter(|(_, out)| inner_vars.contains(out))
                                .map(|(orig, out)| (orig.clone(), out.clone()))
                                .collect();
                            subst_idents(&mut expr, &fb_subst);

                            let is_fb = fb_set.contains(&var.to_string());
                            let out_var = if is_fb {
                                fb_out_name(&var)
                            } else {
                                var.clone()
                            };
                            let env_pat = quote! { (#(#inner_vars,)*) };
                            let env_ident =
                                Ident::new("__arrow_env", proc_macro2::Span::mixed_site());

                            // Build re-bind stmts for type anchoring
                            let mut rebind_stmts = Vec::new();
                            if let Some(ref it) = input_type_ref {
                                if let Some(first_var) = vars.first() {
                                    rebind_stmts.push(quote! {
                                        #[allow(unused_variables)]
                                        let #first_var: #it = #first_var;
                                    });
                                }
                            }
                            if let Some((ref rb_var, ref rb_type)) = pending_rebind {
                                rebind_stmts.push(quote! {
                                    #[allow(unused_variables)]
                                    let #rb_var: #rb_type = #rb_var;
                                });
                            }

                            let let_stmt = if let Some(ref t) = var_type {
                                quote! { let #out_var: #t = #expr; }
                            } else {
                                quote! { let #out_var = #expr; }
                            };

                            let step = quote! {
                                arr(|#env_ident| {
                                    #[allow(unused_variables)]
                                    let #env_pat = #env_ident;
                                    #(#rebind_stmts)*
                                    #let_stmt
                                    (#(#inner_vars,)* #out_var)
                                })
                            };
                            (out_var, step)
                        }
                        _ => continue,
                    };
                    inner_composed = quote! { compose(#inner_composed, #step) };
                    inner_vars.push(out_var);
                }

                // Step 4: project inner machine output to (rec_vars_tuple, feedback)
                //
                // rec_vars_tuple = (non_fb_vars..., fb_out_names...) in rec_vars order.
                // feedback = the new values of the feedback vars (scalar if 1, tuple if N>1).

                let inner_env_pat = quote! { (#(#inner_vars,)*) };

                // rec_vars in declaration order, mapped to their output ident
                let rec_vars_out: Vec<proc_macro2::TokenStream> = rec_vars
                    .iter()
                    .map(|rv| {
                        if fb_set.contains(&rv.to_string()) {
                            let out = fb_out_name(rv);
                            quote! { #out }
                        } else {
                            quote! { #rv }
                        }
                    })
                    .collect();
                let rec_vars_out_tuple = quote! { (#(#rec_vars_out,)*) };

                let feedback_out: Vec<proc_macro2::TokenStream> = feedback_vars
                    .iter()
                    .map(|fv| {
                        let out = fb_out_name(fv);
                        quote! { #out }
                    })
                    .collect();
                let feedback_tuple = if feedback_vars.len() == 1 {
                    feedback_out[0].clone()
                } else {
                    quote! { (#(#feedback_out,)*) }
                };

                let inner_env_ident = Ident::new("__rec_env", proc_macro2::Span::mixed_site());
                let project = quote! {
                    arr(|#inner_env_ident| {
                        #[allow(unused_variables)]
                        let #inner_env_pat = #inner_env_ident;
                        (#rec_vars_out_tuple, #feedback_tuple)
                    })
                };
                let inner_flat = quote! { compose(#inner_composed, #project) };

                // Step 5: adapt inner_flat for cycle
                //
                // cycle expects Machine<(outer_env_tuple, feedback), (rec_vars_tuple, feedback)>
                // but inner_flat takes a flat augmented tuple.
                // Prepend an unpair step to convert the nested input.

                let outer_env_pat = quote! { (#(#vars,)*) };
                let feedback_input_pat = if feedback_vars.len() == 1 {
                    let fv = &feedback_vars[0];
                    quote! { #fv }
                } else {
                    quote! { (#(#feedback_vars,)*) }
                };

                let unpair = quote! {
                    arr(|(#outer_env_pat, #feedback_input_pat)| (#(#vars,)* #(#feedback_vars,)*))
                };
                let inner_for_cycle = quote! { compose(#unpair, #inner_flat) };

                // Step 6: apply cycle
                let init_expr = match init {
                    Some(e) => quote! { #e },
                    None => quote! { Default::default() },
                };
                let cycle_machine = quote! { cycle(#init_expr, #inner_for_cycle) };

                // Step 7: integrate into outer proc
                //
                // fanout outer env with cycle_machine, then flatten to add all
                // rec vars (using original names) to the outer environment.

                let keep_outer_env = quote! { arr(|#outer_env_pat| (#(#vars,)*)) };
                let fanouted_outer = quote! { fanout(#keep_outer_env, #cycle_machine) };

                // Destructure rec_vars_tuple using the ORIGINAL rec var names
                // (not the __rec_X_out renamed ones).  Rust's pattern matching
                // assigns the values to the new names regardless of the internal names.
                let flatten_outer = quote! {
                    arr(|(#outer_env_pat, (#(#rec_vars,)*))| (#(#vars,)* #(#rec_vars,)*))
                };

                let step = quote! { compose(#fanouted_outer, #flatten_outer) };
                composed = quote! { compose(#composed, #step) };

                // Add all rec vars to the outer environment (in declaration order)
                for v in rec_vars {
                    vars.push(v);
                }
            }

            Stmt::Return { expr } => {
                flush_pending_bind(
                    &mut pending_bind,
                    &mut pending_lets,
                    &mut vars,
                    &mut composed,
                    input_type_ref,
                );
                // Extract the return value from the environment.
                let env_pat = quote! { (#(#vars,)*) };
                let env_ident = Ident::new("__arrow_env", proc_macro2::Span::mixed_site());
                let output_arr = quote! {
                    arr(|#env_ident| {
                        #[allow(unused_variables)]
                        let #env_pat = #env_ident;
                        #expr
                    })
                };
                composed = quote! { compose(#composed, #output_arr) };
            }
        }
    }

    // Flush any bind that had no trailing lets (or wasn't flushed by a Return).
    flush_pending_bind(
        &mut pending_bind,
        &mut pending_lets,
        &mut vars,
        &mut composed,
        input_type_ref,
    );

    TokenStream::from(quote! {
        {
            #composed
        }
    })
}
